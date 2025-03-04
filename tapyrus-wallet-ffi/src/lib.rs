use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, io};
use tdk_esplora::esplora_client;
use tdk_esplora::esplora_client::{deserialize, OutputStatus};
use tdk_esplora::EsploraExt;
use tdk_sqlite::{rusqlite::Connection, Store};
use tdk_wallet::descriptor::{Descriptor, DescriptorPublicKey};
use tdk_wallet::miniscript::descriptor::DescriptorSecretKey;
use tdk_wallet::miniscript::ToPublicKey;
use tdk_wallet::signer::SignerId;
use tdk_wallet::tapyrus::bip32::{ChildNumber, Xpriv};
use tdk_wallet::tapyrus::consensus::serialize;
use tdk_wallet::tapyrus::hex::{DisplayHex, FromHex};
use tdk_wallet::tapyrus::script::color_identifier::ColorIdentifier;
use tdk_wallet::tapyrus::secp256k1::hashes::sha256;
use tdk_wallet::tapyrus::secp256k1::hashes::Hash;
use tdk_wallet::tapyrus::secp256k1::rand::Rng;
use tdk_wallet::tapyrus::secp256k1::Message;
use tdk_wallet::tapyrus::secp256k1::ThirtyTwoByteHash;
use tdk_wallet::tapyrus::{base64, secp256k1, Address, BlockHash, PublicKey, ScriptBuf};
use tdk_wallet::tapyrus::{Amount, MalFixTxid, OutPoint, Transaction};
use tdk_wallet::template::Bip44;
use tdk_wallet::wallet::tx_builder::AddUtxoError;
use tdk_wallet::wallet::NewOrLoadError;
use tdk_wallet::{tapyrus, KeychainKind, SignOptions, Wallet};

#[derive(PartialEq, Clone, Debug)]
pub(crate) enum Network {
    Prod,
    Dev,
}

impl From<Network> for tapyrus::network::Network {
    fn from(network: Network) -> Self {
        match network {
            Network::Prod => tapyrus::network::Network::Prod,
            Network::Dev => tapyrus::network::Network::Dev,
        }
    }
}

impl From<tapyrus::network::Network> for Network {
    fn from(network: tapyrus::network::Network) -> Self {
        match network {
            tapyrus::network::Network::Prod => Network::Prod,
            tapyrus::network::Network::Dev => Network::Dev,
            _ => panic!("Unsupported network"),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub network_mode: Network,
    pub network_id: u32,
    pub genesis_hash: String,
    pub esplora_url: String,
    pub esplora_user: Option<String>,
    pub esplora_password: Option<String>,
    pub master_key_path: Option<String>,
    pub master_key: Option<String>,
    pub db_file_path: Option<String>,
}

impl Config {
    /// Create a new Config instance.
    pub fn new(
        network_mode: Network,
        network_id: u32,
        genesis_hash: String,
        esplora_url: String,
        esplora_user: Option<String>,
        esplora_password: Option<String>,
        master_key_path: Option<String>,
        master_key: Option<String>,
        db_file_path: Option<String>,
    ) -> Self {
        Config {
            network_mode,
            network_id,
            genesis_hash,
            esplora_url,
            esplora_user,
            esplora_password,
            master_key_path,
            master_key,
            db_file_path,
        }
    }
}

pub(crate) struct HdWallet {
    network: tapyrus::network::Network,
    wallet: Mutex<Wallet>,
    esplora_url: String,
    esplora_user: Option<String>,
    esplora_password: Option<String>,
}

pub(crate) struct TransferParams {
    pub amount: u64,
    pub to_address: String,
}

#[derive(Debug, Clone)]
pub(crate) struct TxOut {
    pub txid: String,
    pub index: u32,
    pub amount: u64,
    pub color_id: Option<String>,
    pub address: String,
    pub unspent: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Contract {
    pub contract_id: String,
    pub contract: String,
    pub payment_base: String,
    pub payable: bool,
}

impl From<tdk_wallet::chain::Contract> for Contract {
    fn from(contract: tdk_wallet::chain::Contract) -> Self {
        Contract {
            contract_id: contract.contract_id,
            contract: String::from_utf8(contract.contract).unwrap(),
            payment_base: contract.payment_base.to_string(),
            payable: contract.spendable,
        }
    }
}

pub(crate) struct GetNewAddressResult {
    pub address: String,
    pub public_key: String,
}

const SYNC_PARALLEL_REQUESTS: usize = 1;
const STOP_GAP: usize = 25;

// Error type for the wallet
#[derive(Debug)]
pub(crate) enum NewError {
    LoadMasterKeyError {
        cause_description: String,
    },
    LoadWalletDBError {
        cause_description: String,
    },
    ParseGenesisHashError,
    LoadedGenesisDoesNotMatch {
        /// The expected genesis block hash.
        expected: String,
        /// The block hash loaded from persistence.
        got: Option<String>,
    },
    LoadedNetworkDoesNotMatch {
        /// The expected network type.
        expected: Network,
        /// The network type loaded from persistence.
        got: Option<Network>,
    },
    NotInitialized,
    MasterKeyDoesNotMatch {
        /// The descriptor loaded from persistence.
        got: Option<String>,
        /// The keychain of the descriptor not matching
        keychain: String,
    },
}

impl Display for NewError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NewError::LoadMasterKeyError {
                cause_description: e,
            } => {
                write!(f, "Failed to load master key: {}", e)
            }
            NewError::LoadWalletDBError {
                cause_description: e,
            } => {
                write!(f, "Failed to load wallet db: {}", e)
            }
            NewError::ParseGenesisHashError => write!(f, "Failed to parse genesis hash"),
            NewError::LoadedGenesisDoesNotMatch { expected, got } => write!(
                f,
                "Loaded genesis block hash does not match. Expected: {:?}, Got: {:?}",
                expected, got
            ),
            NewError::LoadedNetworkDoesNotMatch { expected, got } => write!(
                f,
                "Loaded network does not match. Expected: {:?}, Got: {:?}",
                expected, got
            ),
            NewError::NotInitialized => {
                write!(f, "Wallet is not initialized")
            }
            NewError::MasterKeyDoesNotMatch { got, keychain } => {
                write!(
                    f,
                    "Master key does not match with persisted. got: {:?}, keychain: {:?}",
                    got, keychain
                )
            }
        }
    }
}

impl std::error::Error for NewError {}

#[derive(Debug)]
pub(crate) enum SyncError {
    EsploraClientError { cause_description: String },
    UpdateWalletError { cause_description: String },
}

impl Display for SyncError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::EsploraClientError {
                cause_description: e,
            } => write!(f, "Esplora client error: {}", e),
            SyncError::UpdateWalletError {
                cause_description: e,
            } => {
                write!(f, "Failed to update wallet: {}", e)
            }
        }
    }
}

impl std::error::Error for SyncError {}

#[derive(Debug)]
pub(crate) enum GetNewAddressError {
    InvalidColorId,
}

impl Display for GetNewAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetNewAddressError::InvalidColorId => write!(f, "Invalid color id"),
        }
    }
}

impl std::error::Error for GetNewAddressError {}

#[derive(Debug)]
pub(crate) enum BalanceError {
    InvalidColorId,
}

impl Display for BalanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BalanceError::InvalidColorId => write!(f, "Invalid color id"),
        }
    }
}

impl std::error::Error for BalanceError {}

#[derive(Debug)]
pub(crate) enum TransferError {
    InsufficientFund,
    EsploraClient { cause_description: String },
    FailedToParseAddress { address: String },
    WrongNetworkAddress { address: String },
    FailedToParseTxid { txid: String },
    InvalidTransferAmount { cause_description: String },
    UnknownUtxo { utxo: TxOut },
    FailedToCreateTransaction { cause_description: String },
}

impl Display for TransferError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::InsufficientFund => write!(f, "Insufficient fund"),
            TransferError::EsploraClient {
                cause_description: e,
            } => write!(f, "Esplora client error: {}", e),
            TransferError::FailedToParseAddress { address: e } => {
                write!(f, "Failed to parse address: {}", e)
            }
            TransferError::WrongNetworkAddress { address: e } => {
                write!(f, "Wrong network address: {}", e)
            }
            TransferError::FailedToParseTxid { txid: e } => {
                write!(f, "Failed to parse txid: {}", e)
            }
            TransferError::InvalidTransferAmount {
                cause_description: e,
            } => {
                write!(f, "Invalid transfer amount: {}", e)
            }
            TransferError::UnknownUtxo { utxo: e } => write!(f, "Unknown utxo: {:?}", e),
            TransferError::FailedToCreateTransaction {
                cause_description: e,
            } => {
                write!(f, "Failed to create transaction: {}", e)
            }
        }
    }
}

impl std::error::Error for TransferError {}

#[derive(Debug)]
pub(crate) enum GetTransactionError {
    FailedToParseTxid { txid: String },
    EsploraClientError { cause_description: String },
    UnknownTxid,
}

impl Display for GetTransactionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTransactionError::FailedToParseTxid { txid: e } => {
                write!(f, "Failed to parse txid: {}", e)
            }
            GetTransactionError::EsploraClientError {
                cause_description: e,
            } => {
                write!(f, "Esplora client error: {}", e)
            }
            GetTransactionError::UnknownTxid => write!(f, "Unknown txid"),
        }
    }
}

impl std::error::Error for GetTransactionError {}

#[derive(Debug)]
pub(crate) enum GetTxOutByAddressError {
    FailedToParseTxHex,
    FailedToParseAddress {
        address: String,
    },
    EsploraClientError {
        cause_description: String,
    },
    /// The transaction is not found in Esplora.
    UnknownTransaction,
}

impl Display for GetTxOutByAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTxOutByAddressError::FailedToParseTxHex => write!(f, "Failed to parse tx hex"),
            GetTxOutByAddressError::FailedToParseAddress { address: e } => {
                write!(f, "Failed to parse address: {}", e)
            }
            GetTxOutByAddressError::EsploraClientError {
                cause_description: e,
            } => {
                write!(f, "Esplora client error: {}", e)
            }
            GetTxOutByAddressError::UnknownTransaction => write!(f, "Unknown transaction"),
        }
    }
}

impl std::error::Error for GetTxOutByAddressError {}

#[derive(Debug)]
pub(crate) enum CalcPayToContractAddressError {
    FailedToParsePublicKey,
    InvalidColorId,
    ContractError { cause_description: String },
}

impl Display for CalcPayToContractAddressError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcPayToContractAddressError::FailedToParsePublicKey => {
                write!(f, "Failed to parse public key")
            }
            CalcPayToContractAddressError::InvalidColorId => write!(f, "Invalid color id"),
            CalcPayToContractAddressError::ContractError {
                cause_description: e,
            } => {
                write!(f, "Contract error: {}", e)
            }
        }
    }
}

impl std::error::Error for CalcPayToContractAddressError {}

#[derive(Debug)]
pub(crate) enum StoreContractError {
    ContractError { cause_description: String },
    FailedToParsePublicKey,
}

impl Display for StoreContractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreContractError::ContractError {
                cause_description: e,
            } => write!(f, "Contract error: {}", e),
            StoreContractError::FailedToParsePublicKey => {
                write!(f, "Failed to parse public key")
            }
        }
    }
}

impl std::error::Error for StoreContractError {}

#[derive(Debug)]
pub(crate) enum UpdateContractError {
    ContractError { cause_description: String },
}

impl Display for UpdateContractError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UpdateContractError::ContractError {
                cause_description: e,
            } => write!(f, "Contract error: {}", e),
        }
    }
}

impl std::error::Error for UpdateContractError {}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum SignMessageError {
    FailedToParsePublicKey,
    PublicKeyNotFoundInWallet,
}

impl Display for SignMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SignMessageError::FailedToParsePublicKey => {
                write!(f, "Failed to parse public key")
            }
            SignMessageError::PublicKeyNotFoundInWallet => {
                write!(f, "Public key not found in wallet")
            }
        }
    }
}

impl std::error::Error for SignMessageError {}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum VerifySignError {
    FailedToParsePublicKey,
    FailedToParseSignature,
}

impl Display for VerifySignError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifySignError::FailedToParsePublicKey => {
                write!(f, "Failed to parse public key")
            }
            VerifySignError::FailedToParseSignature => {
                write!(f, "Failed to parse signature")
            }
        }
    }
}

impl std::error::Error for VerifySignError {}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum CheckTrustLayerRefundError {
    FailedToParseTxid { txid: String },
    EsploraClientError { cause_description: String },
    UnknownTxid,
    CannotFoundRefundTransaction { txid: String },
    InvalidColorId,
}

impl Display for CheckTrustLayerRefundError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckTrustLayerRefundError::FailedToParseTxid { txid: e } => {
                write!(f, "Failed to parse txid: {}", e)
            }
            CheckTrustLayerRefundError::EsploraClientError {
                cause_description: e,
            } => {
                write!(f, "Esplora client error: {}", e)
            }
            CheckTrustLayerRefundError::UnknownTxid => write!(f, "Unknown txid"),
            CheckTrustLayerRefundError::CannotFoundRefundTransaction { txid: e } => {
                write!(f, "Cannot found refund transaction: {}", e)
            }
            CheckTrustLayerRefundError::InvalidColorId => write!(f, "Invalid color id"),
        }
    }
}

impl std::error::Error for CheckTrustLayerRefundError {}

impl HdWallet {
    pub fn new(config: Arc<Config>) -> Result<Self, NewError> {
        let Config {
            network_mode,
            network_id,
            genesis_hash,
            esplora_url,
            esplora_user,
            esplora_password,
            master_key_path,
            master_key,
            db_file_path,
        } = config.as_ref();

        let network: tapyrus::network::Network = network_mode.clone().into();

        let master_key = if master_key.is_some() && master_key_path.is_some() {
            return Err(NewError::LoadMasterKeyError {
                cause_description:
                    "master_key_path and master_key cannot be specified at the same time"
                        .to_string(),
            });
        } else if master_key.is_none() {
            let master_key_path = master_key_path
                .clone()
                .unwrap_or_else(|| "master_key".to_string());
            initialize_or_load_master_key(&master_key_path, network).map_err(|_| {
                NewError::LoadMasterKeyError {
                    cause_description: format!(
                        "Failed to read or crate file at {}",
                        master_key_path
                    )
                    .to_string(),
                }
            })?
        } else {
            Xpriv::from_str(&(master_key.clone().unwrap())).map_err(|_| {
                NewError::LoadMasterKeyError {
                    cause_description: "Failed to parse master_key.".to_string(),
                }
            })?
        };

        let db_path = db_file_path
            .clone()
            .unwrap_or_else(|| "tapyrus-wallet.sqlite".to_string());
        let conn = Connection::open(&db_path).map_err(|e| NewError::LoadWalletDBError {
            cause_description: e.to_string(),
        })?;
        let db = Store::new(conn).map_err(|e| NewError::LoadWalletDBError {
            cause_description: e.to_string(),
        })?;

        let genesis_hash =
            BlockHash::from_str(genesis_hash).map_err(|_| NewError::ParseGenesisHashError)?;

        let wallet = Wallet::new_or_load_with_genesis_hash(
            Bip44(master_key, KeychainKind::External),
            Bip44(master_key, KeychainKind::Internal),
            db,
            network,
            genesis_hash,
        )
        .map_err(|e| match e {
            NewOrLoadError::Persist(e) => NewError::LoadWalletDBError {
                cause_description: e.to_string(),
            },
            NewOrLoadError::NotInitialized => NewError::NotInitialized,
            NewOrLoadError::LoadedGenesisDoesNotMatch { expected, got } => {
                NewError::LoadedGenesisDoesNotMatch {
                    expected: expected.to_string(),
                    got: got.map(|h| h.to_string()),
                }
            }
            NewOrLoadError::LoadedNetworkDoesNotMatch { expected, got } => {
                NewError::LoadedNetworkDoesNotMatch {
                    expected: expected.into(),
                    got: got.map(|n| n.into()),
                }
            }
            NewOrLoadError::Descriptor(e) => NewError::LoadMasterKeyError {
                cause_description: e.to_string(),
            },
            NewOrLoadError::LoadedDescriptorDoesNotMatch { got, keychain } => {
                NewError::MasterKeyDoesNotMatch {
                    got: got.map(|h| h.to_string()),
                    keychain: keychain.as_byte().to_string(),
                }
            }
        })?;

        Ok(HdWallet {
            network,
            wallet: Mutex::new(wallet),
            esplora_url: esplora_url.clone(),
            esplora_user: esplora_user.clone(),
            esplora_password: esplora_password.clone(),
        })
    }

    pub fn sync(&self) -> Result<(), SyncError> {
        let mut wallet = self.get_wallet();
        let client = self.esplora_client();

        let request = wallet.start_sync_with_revealed_spks();
        let update = client.sync(request, SYNC_PARALLEL_REQUESTS).map_err(|e| {
            SyncError::EsploraClientError {
                cause_description: e.to_string(),
            }
        })?;

        wallet
            .apply_update(update)
            .map_err(|e| SyncError::UpdateWalletError {
                cause_description: e.to_string(),
            })?;
        Ok(())
    }

    pub fn full_sync(&self) -> Result<(), SyncError> {
        let mut wallet = self.get_wallet();
        let client = self.esplora_client();

        let request = wallet.start_full_scan();
        let update = client
            .full_scan(request, STOP_GAP, SYNC_PARALLEL_REQUESTS)
            .map_err(|e| SyncError::EsploraClientError {
                cause_description: e.to_string(),
            })?;

        wallet
            .apply_update(update)
            .map_err(|e| SyncError::UpdateWalletError {
                cause_description: e.to_string(),
            })?;
        Ok(())
    }

    fn esplora_client(&self) -> esplora_client::BlockingClient {
        let mut builder = esplora_client::Builder::new(&self.esplora_url);

        // Set basic authentication if user and password are provided
        if let (Some(user), Some(password)) = (&self.esplora_user, &self.esplora_password) {
            use base64::prelude::*;

            let credentials = format!("{}:{}", user, password);
            let encoded = BASE64_STANDARD.encode(credentials.as_bytes());
            let auth_haeder_value = format!("Basic {}", encoded);
            builder = builder.header("Authorization", auth_haeder_value.as_str());
        }

        builder.build_blocking()
    }

    fn get_wallet(&self) -> MutexGuard<Wallet> {
        self.wallet.lock().expect("Failed to lock wallet")
    }

    pub fn get_new_address(
        &self,
        color_id: Option<String>,
    ) -> Result<GetNewAddressResult, GetNewAddressError> {
        let mut wallet = self.get_wallet();
        let keychain = KeychainKind::External;
        let address_info = wallet.reveal_next_address(keychain).unwrap();

        let descriptor = wallet.get_descriptor_for_keychain(keychain);
        let secp = secp256k1::Secp256k1::verification_only();
        let derived_descriptor = descriptor
            .derived_descriptor(&secp, address_info.index)
            .unwrap();
        let public_key = match derived_descriptor {
            Descriptor::Pkh(a) => a.into_inner(),
            _ => {
                panic!("get_new_address() doesn't support Bare and Sh descriptor")
            }
        };

        let address = if let Some(color_id) = color_id {
            let color_id = ColorIdentifier::from_str(&color_id)
                .map_err(|_| GetNewAddressError::InvalidColorId)?;
            let script = address_info.script_pubkey().add_color(color_id).unwrap();
            Address::from_script(&script, self.network).unwrap()
        } else {
            address_info.address
        };

        Ok(GetNewAddressResult {
            address: address.to_string(),
            public_key: public_key.to_string(),
        })
    }

    pub fn balance(&self, color_id: Option<String>) -> Result<u64, BalanceError> {
        let color_id = if let Some(color_id) = color_id {
            ColorIdentifier::from_str(&color_id).map_err(|_| BalanceError::InvalidColorId)?
        } else {
            ColorIdentifier::default()
        };
        let balance = self.get_wallet().balance(color_id);
        Ok(balance.total().to_tap())
    }

    pub fn transfer(
        &self,
        params: Vec<TransferParams>,
        utxos: Vec<TxOut>,
    ) -> Result<String, TransferError> {
        let mut wallet = self.get_wallet();
        let client = self.esplora_client();

        let mut tx_builder = wallet.build_tx();
        params.iter().try_for_each(|param| {
            let address = Address::from_str(&param.to_address).map_err(|_| {
                TransferError::FailedToParseAddress {
                    address: (&param.to_address).clone(),
                }
            })?;
            let address = address.require_network(self.network).map_err(|_| {
                TransferError::WrongNetworkAddress {
                    address: (&param.to_address).clone(),
                }
            })?;

            let script = address.script_pubkey();
            if script.is_colored() {
                let color_id = script.color_id().unwrap();
                let non_colored_script = script.remove_color();
                tx_builder.add_recipient_with_color(
                    non_colored_script,
                    Amount::from_tap(param.amount),
                    color_id,
                );
            } else {
                tx_builder.add_recipient(script, Amount::from_tap(param.amount));
            }
            Ok(())
        })?;

        tx_builder
            .add_utxos(
                &utxos
                    .iter()
                    .map(|utxo| {
                        let txid = MalFixTxid::from_str(&utxo.txid).map_err(|_| {
                            TransferError::FailedToParseTxid {
                                txid: (&utxo.txid).clone(),
                            }
                        })?;
                        Ok(OutPoint::new(txid, utxo.index))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|e| match e {
                AddUtxoError::UnknownUtxo(outpoint) => {
                    let utxo = utxos
                        .iter()
                        .find(|utxo| {
                            utxo.txid == outpoint.txid.to_string() && utxo.index == outpoint.vout
                        })
                        .unwrap();
                    TransferError::UnknownUtxo { utxo: utxo.clone() }
                }
                AddUtxoError::ContractError => {
                    panic!("ContractError")
                }
            })?;

        let mut psbt =
            tx_builder
                .finish()
                .map_err(|e| TransferError::FailedToCreateTransaction {
                    cause_description: e.to_string(),
                })?;
        wallet
            .sign(&mut psbt, SignOptions::default())
            .map_err(|e| TransferError::FailedToCreateTransaction {
                cause_description: e.to_string(),
            })?;
        let tx = psbt
            .extract_tx()
            .map_err(|e| TransferError::FailedToCreateTransaction {
                cause_description: e.to_string(),
            })?;
        client
            .broadcast(&tx)
            .map_err(|e| TransferError::EsploraClient {
                cause_description: e.to_string(),
            })?;

        Ok(tx.malfix_txid().to_string())
    }

    pub fn get_transaction(&self, txid: String) -> Result<String, GetTransactionError> {
        let client = self.esplora_client();
        let txid = txid
            .parse::<MalFixTxid>()
            .map_err(|_| GetTransactionError::FailedToParseTxid { txid })?;
        let tx = client
            .get_tx(&txid)
            .map_err(|e| GetTransactionError::EsploraClientError {
                cause_description: e.to_string(),
            })?;
        match tx {
            Some(tx) => Ok(serialize(&tx).to_lower_hex_string()),
            None => Err(GetTransactionError::UnknownTxid),
        }
    }

    pub fn get_tx_out_by_address(
        &self,
        tx: String,
        address: String,
    ) -> Result<Vec<TxOut>, GetTxOutByAddressError> {
        let raw = Vec::from_hex(&tx).map_err(|_| GetTxOutByAddressError::FailedToParseTxHex)?;
        let tx: Transaction =
            deserialize(raw.as_slice()).map_err(|_| GetTxOutByAddressError::FailedToParseTxHex)?;
        let script_pubkey = Address::from_str(&address)
            .map_err(|_| GetTxOutByAddressError::FailedToParseAddress {
                address: address.clone(),
            })?
            .require_network(self.network)
            .map_err(|_| GetTxOutByAddressError::FailedToParseAddress {
                address: address.clone(),
            })?
            .script_pubkey();
        let client = self.esplora_client();

        tx.output
            .iter()
            .enumerate()
            .try_fold(Vec::new(), |mut acc, (i, o)| {
                if o.script_pubkey == script_pubkey {
                    let status = client
                        .get_output_status(&tx.malfix_txid(), i as u64)
                        .map_err(|e| GetTxOutByAddressError::EsploraClientError {
                            cause_description: e.to_string(),
                        })?;

                    let status = match status {
                        Some(status) => status,
                        None => return Err(GetTxOutByAddressError::UnknownTransaction),
                    };

                    let txout = TxOut {
                        txid: tx.malfix_txid().to_string(),
                        index: i as u32,
                        amount: o.value.to_tap(),
                        color_id: o.script_pubkey.color_id().map(|id| id.to_string()),
                        address: Address::from_script(&o.script_pubkey, self.network)
                            .unwrap()
                            .to_string(),
                        unspent: !status.spent,
                    };
                    acc.push(txout);
                }

                Ok(acc)
            })
    }

    pub fn calc_p2c_address(
        &self,
        public_key: String,
        contract: String,
        color_id: Option<String>,
    ) -> Result<String, CalcPayToContractAddressError> {
        let wallet = self.get_wallet();
        let payment_base = PublicKey::from_str(&public_key)
            .map_err(|_| CalcPayToContractAddressError::FailedToParsePublicKey)?;
        let contract = contract.as_bytes().to_vec();
        let color_id = match color_id {
            Some(id) => Some(
                ColorIdentifier::from_str(&id)
                    .map_err(|_| CalcPayToContractAddressError::InvalidColorId)?,
            ),
            None => None,
        };
        let address = wallet
            .create_pay_to_contract_address(&payment_base, contract, color_id)
            .map_err(|e| CalcPayToContractAddressError::ContractError {
                cause_description: e.to_string(),
            })?;
        Ok(address.to_string())
    }

    pub fn store_contract(&self, contract: Contract) -> Result<Contract, StoreContractError> {
        let mut wallet = self.get_wallet();
        let payment_base = PublicKey::from_str(&contract.payment_base)
            .map_err(|_| StoreContractError::FailedToParsePublicKey)?;
        let contract = wallet
            .store_contract(
                contract.contract_id,
                contract.contract.as_bytes().to_vec(),
                payment_base,
                contract.payable,
            )
            .map_err(|e| StoreContractError::ContractError {
                cause_description: e.to_string(),
            })?;
        Ok(contract.into())
    }

    pub fn update_contract(
        &self,
        contract_id: String,
        payable: bool,
    ) -> Result<(), UpdateContractError> {
        let mut wallet = self.get_wallet();
        wallet.update_contract(contract_id, payable).map_err(|e| {
            UpdateContractError::ContractError {
                cause_description: e.to_string(),
            }
        })?;
        Ok(())
    }

    pub fn check_trust_layer_refund(
        &self,
        txid: String,
        color_id: String,
    ) -> Result<u64, CheckTrustLayerRefundError> {
        let wallet = self.get_wallet();
        let client = self.esplora_client();
        let txid = txid
            .parse::<MalFixTxid>()
            .map_err(|_| CheckTrustLayerRefundError::FailedToParseTxid { txid: txid.clone() })?;
        let color_id = ColorIdentifier::from_str(&color_id)
            .map_err(|_| CheckTrustLayerRefundError::InvalidColorId)?;

        // get transactions that uses the txid as input
        let opt_tx =
            client
                .get_tx(&txid)
                .map_err(|e| CheckTrustLayerRefundError::EsploraClientError {
                    cause_description: e.to_string(),
                })?;
        let tx = match opt_tx {
            Some(tx) => tx,
            None => return Err(CheckTrustLayerRefundError::UnknownTxid),
        };

        // filter outputs that send the color_id token to other wallet
        let mut transfer_txouts = tx.output.iter().enumerate().filter(|(_, txout)| {
            // filter outputs that send the color_id token to other wallet
            let output_color_id = txout.script_pubkey.color_id();
            let script_pubkey = txout.script_pubkey.remove_color();
            output_color_id.is_some()
                && output_color_id.unwrap() == color_id
                && !wallet.is_mine(script_pubkey.as_script()) // exclude change outputs
        });

        // fold the amount of refund txout value that is sent back to the wallet
        transfer_txouts.try_fold(
            0u64,
            |acc, (index, _)| -> Result<u64, CheckTrustLayerRefundError> {
                let output_status = client.get_output_status(&txid, index as u64).map_err(|e| {
                    CheckTrustLayerRefundError::EsploraClientError {
                        cause_description: e.to_string(),
                    }
                })?;
                match output_status {
                    Some(OutputStatus {
                        txid: Some(txid), ..
                    }) => {
                        let opt_tx = client.get_tx(&txid).map_err(|e| {
                            CheckTrustLayerRefundError::EsploraClientError {
                                cause_description: e.to_string(),
                            }
                        })?;
                        let tx = match opt_tx {
                            Some(tx) => tx,
                            None => {
                                return Err(
                                    CheckTrustLayerRefundError::CannotFoundRefundTransaction {
                                        txid: txid.to_string(),
                                    },
                                )
                            }
                        };
                        let refund_txout = tx.output.iter().find(|txout| {
                            if txout.script_pubkey.color_id().is_some()
                                && txout.script_pubkey.color_id().unwrap() == color_id
                            {
                                let script_pubkey = txout.script_pubkey.remove_color();
                                wallet.is_mine(script_pubkey.as_script())
                            } else {
                                false
                            }
                        });
                        match refund_txout {
                            Some(refund_txout) => Ok(acc + refund_txout.value.to_tap()),
                            None => Ok(acc),
                        }
                    }
                    Some(_) => Ok(acc),
                    None => Ok(acc),
                }
            },
        )
    }

    pub fn sign_message(
        &self,
        public_key: String,
        message: String,
    ) -> Result<String, SignMessageError> {
        let wallet = self.get_wallet();
        let public_key = PublicKey::from_str(&public_key)
            .map_err(|_| SignMessageError::FailedToParsePublicKey)?;
        let message_bytes = message.as_bytes();
        let message_hash: sha256::Hash = Hash::hash(message_bytes);
        let message = Message::from(message_hash);
        let keychains: BTreeMap<_, _> = wallet.keychains().collect();
        let descriptor = keychains.get(&KeychainKind::External).unwrap();
        let script_buf = ScriptBuf::new_p2pkh(&public_key.pubkey_hash());
        let spk = script_buf.as_script();
        let next_index = wallet
            .spk_index()
            .next_index(&KeychainKind::External)
            .unwrap()
            .0;
        match descriptor
            .find_derivation_index_for_spk(wallet.secp_ctx(), &spk, 0..next_index)
            .unwrap()
        {
            Some((index, _)) => {
                let signers = wallet.get_signers(KeychainKind::External);
                let key_map = signers.as_key_map(wallet.secp_ctx());

                let (_, secret) = key_map.iter().next().unwrap();
                match secret {
                    DescriptorSecretKey::XPrv(xprv) => {
                        let path = xprv
                            .derivation_path
                            .extend(&[ChildNumber::from_normal_idx(index).unwrap()]);
                        let derived_xprv = xprv.xkey.derive_priv(wallet.secp_ctx(), &path).unwrap();
                        let secp = wallet.secp_ctx();
                        let sig = secp.sign_ecdsa(&message, &derived_xprv.private_key);
                        Ok(sig.serialize_der().to_lower_hex_string())
                    }
                    _ => {
                        unreachable!("Invalid private key type");
                    }
                }
            }
            None => Err(SignMessageError::PublicKeyNotFoundInWallet),
        }
    }

    pub fn verify_sign(
        &self,
        public_key: String,
        message: String,
        sign: String,
    ) -> Result<bool, VerifySignError> {
        let public_key = PublicKey::from_str(&public_key)
            .map_err(|_| VerifySignError::FailedToParsePublicKey)?;
        let message_bytes = message.as_bytes();
        let message_hash: sha256::Hash = Hash::hash(message_bytes);
        let message = Message::from(message_hash);

        let sign = Vec::from_hex(&sign).map_err(|_| VerifySignError::FailedToParseSignature)?;
        let secp = secp256k1::Secp256k1::verification_only();
        let signature = secp256k1::ecdsa::Signature::from_der(&sign)
            .map_err(|_| VerifySignError::FailedToParseSignature)?;
        match secp.verify_ecdsa(&message, &signature, &public_key.inner) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

fn initialize_or_load_master_key(file_path: &str, network: tapyrus::Network) -> io::Result<Xpriv> {
    if fs::metadata(file_path).is_ok() {
        // File exists, read the private key
        let mut file = File::open(file_path)?;
        let mut xpriv_str = String::new();
        file.read_to_string(&mut xpriv_str)?;
        let xpriv = Xpriv::from_str(&xpriv_str).expect("Failed to parse Xpriv from file");
        Ok(xpriv)
    } else {
        // File doesn't exist, generate Xpriv and persist
        let seed: [u8; 32] = secp256k1::rand::thread_rng().gen();
        let xpriv = Xpriv::new_master(network, &seed).unwrap();
        let xpriv_str = xpriv.to_string();
        let mut file = File::create(file_path)?;
        file.write_all(xpriv_str.as_bytes())?;
        Ok(xpriv)
    }
}

fn generate_master_key(network: Network) -> String {
    let seed: [u8; 32] = secp256k1::rand::rngs::OsRng.gen();
    Xpriv::new_master(network.into(), &seed)
        .unwrap()
        .to_string()
}

uniffi::include_scaffolding!("wallet");

#[cfg(test)]
mod test {
    use crate::*;
    use esplora_client::{self, BlockingClient, ScriptBuf};
    use rand::random;
    use std::hash::Hash;
    use std::thread::sleep;
    use std::time::Duration;
    use std::{env, thread};
    use tdk_chain::serde::{Deserialize, Serialize};
    use tdk_esplora::esplora_client::Builder;
    use tdk_testenv::{anyhow, tapyruscore_rpc::RpcApi, TestEnv};
    use tdk_wallet::tapyrus::PubkeyHash;

    fn db_file_path() -> String {
        let mut temp_path = env::temp_dir();
        let file_name = format!("tapyrus-wallet-{}.sqlite", random::<u32>());
        temp_path.push(file_name);
        temp_path.to_str().unwrap().to_string()
    }

    fn get_wallet() -> HdWallet {
        let db_file_path = db_file_path();

        // testnet setting
        let config = Config {
            network_mode: Network::Prod,
            network_id: 1939510133,
            genesis_hash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a"
                .to_string(),
            esplora_url: "http://localhost:3001".to_string(),
            esplora_user: None,
            esplora_password: None,
            master_key_path: None,
            master_key: Some("xprv9s21ZrQH143K3fYtYJZ5aLANmuode1z8g2AoQdwcxSrAwo6LzzGMSyNMLNw9d1q7TGPEc9d3bd2DjPaCJXR7pbWh1xuSFSRYsy1HHDeivek".to_string()),
            db_file_path: Some(db_file_path),
        };
        HdWallet::new(Arc::new(config)).unwrap()
    }

    fn get_wallet_testenv(
        env: &TestEnv,
        client: &BlockingClient,
        master_key: Option<String>,
    ) -> HdWallet {
        let db_file_path = db_file_path();

        // connect to testenv
        let config = Config {
            network_mode: Network::Dev,
            network_id: 1905960821,
            genesis_hash: "aa71d030ac96eafa5cd4cb6dcbd8e8845c03b1a60641bf816c85e97bcf6bb8ea"
                .to_string(),
            esplora_url: format!("http://{}", &env.electrsd.esplora_url.clone().unwrap()),
            esplora_user: None,
            esplora_password: None,
            master_key_path: None,
            master_key: Some(master_key.unwrap_or("tprv8ZgxMBicQKsPeDdk6yMbK91PfeqepaeaKj1yGLRAGAac3yZEYS5Z6vMKu8rmybsyHWiEQ1JAZihfUC3DmGXq6H8279NVL7F8poWjVtVdFU9".to_string())),
            db_file_path: Some(db_file_path),
        };
        let wallet = HdWallet::new(Arc::new(config)).unwrap();

        wallet.full_sync().expect("Failed to sync");
        let balance = wallet.balance(None).unwrap();
        assert_eq!(balance, 0);

        // Send TPC to the wallet for paying fee
        let GetNewAddressResult {
            address: address, ..
        } = wallet.get_new_address(None).unwrap();
        let address = Address::from_str(&address).unwrap().assume_checked();
        let _ = env.tapyrusd.client.send_to_address(
            &address,
            Amount::from_tap(20000),
            None,
            None,
            None,
            None,
            Some(1),
            None,
        );
        wait_for_confirmation(&env, &client, 1);
        wallet.sync().expect("Failed to sync");
        let balance = wallet.balance(None).unwrap();
        assert_eq!(balance, 20000);

        wallet
    }

    fn wait_for_confirmation(
        env: &TestEnv,
        client: &BlockingClient,
        count: usize,
    ) -> anyhow::Result<()> {
        let height = client.get_height().unwrap();
        let _block_hashes = env.mine_blocks(count, None)?;
        while client.get_height().unwrap() < height + (count as u32) {
            sleep(Duration::from_millis(100))
        }
        Ok(())
    }

    #[test]
    fn test_get_new_address() {
        let wallet = get_wallet();
        let GetNewAddressResult {
            address,
            public_key,
        } = wallet.get_new_address(None).unwrap();
        assert_eq!(address.len(), 34, "Address should be 34 characters long");
        let public_key = PublicKey::from_str(&public_key).unwrap();
        assert_eq!(
            address,
            Address::p2pkh(&public_key, wallet.network).to_string()
        );

        let color_id = ColorIdentifier::from_str(
            "c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46",
        )
        .unwrap();
        let GetNewAddressResult {
            address,
            public_key,
        } = wallet.get_new_address(Some(color_id.to_string())).unwrap();
        assert_eq!(address.len(), 78, "Address should be 78 characters long");
        let public_key = PublicKey::from_str(&public_key).unwrap();
        let spk = ScriptBuf::new_cp2pkh(&color_id, &PubkeyHash::from(public_key));
        let expected = Address::from_script(&spk, wallet.network)
            .unwrap()
            .to_string();
        assert_eq!(address, expected);
    }

    #[test]
    fn test_generate_master_key() {
        let db_file_path = db_file_path();
        let master_key = generate_master_key(Network::Prod);

        // testnet setting
        let config = Config {
            network_mode: Network::Prod,
            network_id: 1939510133,
            genesis_hash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a"
                .to_string(),
            esplora_url: "http://localhost:3001".to_string(),
            esplora_user: None,
            esplora_password: None,
            master_key_path: None,
            master_key: Some(master_key),
            db_file_path: Some(db_file_path),
        };
        HdWallet::new(Arc::new(config)).unwrap();
    }

    #[test]
    fn test_balance() {
        let wallet = get_wallet();
        let balance = wallet.balance(None).unwrap();
        assert_eq!(balance, 0, "Balance should be 0");

        let color_id = ColorIdentifier::from_str(
            "c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46",
        )
        .unwrap();
        let balance = wallet.balance(Some(color_id.to_string())).unwrap();
        assert_eq!(balance, 0, "Balance should be 0");
    }

    #[test]
    fn test_calc_p2c_address() {
        let wallet = get_wallet();
        let public_key =
            "039be0d2b0c3b6f7fad77f142257aee12b2a34047aa3191edc0424cd15e0fa15da".to_string();
        let address = wallet
            .calc_p2c_address(public_key, "content".to_string(), None)
            .expect("Failed to calculate P2C address");
        assert_eq!(
            address, "1NUKT87AxtsJ74EiZ6esDz8kjppHS4cKz2",
            "Address should be equal"
        );
    }

    #[test]
    fn test_store_contract() {
        let wallet = get_wallet();
        let GetNewAddressResult {
            address,
            public_key,
        } = wallet.get_new_address(None).unwrap();
        let contract = Contract {
            contract_id: "contract_id".to_string(),
            contract: "contract".to_string(),
            payment_base: public_key,
            payable: true,
        };
        let stored_contract = wallet
            .store_contract(contract.clone())
            .expect("Failed to store contract");

        // Update contract
        let updated_contract = wallet
            .update_contract(contract.contract_id.clone(), false)
            .expect("Failed to update contract");
    }

    #[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
    #[cfg_attr(
        feature = "serde",
        derive(serde::Deserialize, serde::Serialize),
        serde(crate = "serde_crate")
    )]
    pub struct IssueResponse {
        color: String,
        txids: Vec<MalFixTxid>,
    }

    fn test_env() -> TestEnv {
        std::env::set_var("NETWORK_ID", "1905960821");
        std::env::set_var(
            "PRIVATE_KEY",
            "cUJN5RVzYWFoeY8rUztd47jzXCu1p57Ay8V7pqCzsBD3PEXN7Dd4",
        );
        std::env::set_var("GENESIS_BLOCK", "0100000000000000000000000000000000000000000000000000000000000000000000002b5331139c6bc8646bb4e5737c51378133f70b9712b75548cb3c05f9188670e7440d295e7300c5640730c4634402a3e66fb5d921f76b48d8972a484cc0361e66ef74f45e012103af80b90d25145da28c583359beb47b21796b2fe1a23c1511e443e7a64dfdb27d40e05f064662d6b9acf65ae416379d82e11a9b78cdeb3a316d1057cd2780e3727f70a61f901d10acbe349cd11e04aa6b4351e782c44670aefbe138e99a5ce75ace01010000000100000000000000000000000000000000000000000000000000000000000000000000000000ffffffff0100f2052a010000001976a91445d405b9ed450fec89044f9b7a99a4ef6fe2cd3f88ac00000000");

        TestEnv::new().unwrap()
    }

    fn prepare_token() -> (TestEnv, ColorIdentifier, BlockingClient) {
        let env = test_env();
        let esplora_url = format!("http://{}", &env.electrsd.esplora_url.clone().unwrap());
        let client = Builder::new(esplora_url.as_str()).build_blocking();

        let address: String = env.tapyrusd.client.call("getnewaddress", &[]).unwrap();
        let address: Address = Address::from_str(&address).unwrap().assume_checked();
        let _ = env
            .tapyrusd
            .client
            .send_to_address(
                &address,
                Amount::from_tap(30000),
                None,
                None,
                None,
                None,
                Some(1),
                None,
            )
            .unwrap();
        wait_for_confirmation(&env, &client, 101);

        // Issue to core wallet
        let issue_spk = address.script_pubkey();
        let ret: IssueResponse = env
            .tapyrusd
            .client
            .call(
                "issuetoken",
                &[1.into(), 1000.into(), issue_spk.to_hex_string().into()],
            )
            .unwrap();
        let color_id = ColorIdentifier::from_str(&ret.color).unwrap();

        wait_for_confirmation(&env, &client, 1);
        (env, color_id, client)
    }

    fn distribute_token(
        wallet: &HdWallet,
        env: &TestEnv,
        color_id: &ColorIdentifier,
        client: &BlockingClient,
    ) {
        let GetNewAddressResult {
            address: address, ..
        } = wallet.get_new_address(Some(color_id.to_string())).unwrap();

        let txid: MalFixTxid = env
            .tapyrusd
            .client
            .call("transfertoken", &[address.to_string().into(), 100.into()])
            .unwrap();

        wait_for_confirmation(&env, &client, 1);
        wallet.sync().expect("Failed to sync");

        let balance = wallet.balance(Some(color_id.clone().to_string())).unwrap();
        assert_eq!(balance, 100);
    }

    #[test]
    fn test_receive_pay_to_contract_transfer_and_transfer_pay_to_contract_utxo() {
        let (env, color_id, client) = prepare_token();
        let wallet = get_wallet_testenv(&env, &client, None);

        // create cp2pkh address
        let GetNewAddressResult { public_key, .. } =
            wallet.get_new_address(Some(color_id.to_string())).unwrap();
        let p2c_address = wallet
            .calc_p2c_address(
                public_key.clone(),
                "content".to_string(),
                Some(color_id.clone().to_string()),
            )
            .unwrap();

        //send p2c token from tapyrus core wallet.
        let txid: MalFixTxid = env
            .tapyrusd
            .client
            .call(
                "transfertoken",
                &[p2c_address.to_string().into(), 400.into()],
            )
            .unwrap();

        let contract = Contract {
            contract_id: "contract_id".to_string(),
            contract: "content".to_string(),
            payment_base: public_key,
            payable: false,
        };

        wallet
            .store_contract(contract.clone())
            .expect("Failed to store contract");

        wait_for_confirmation(&env, &client, 1);
        wallet.sync().expect("Failed to sync");

        let balance = wallet.balance(Some(color_id.clone().to_string())).unwrap();
        assert_eq!(balance, 400);

        let transaction = wallet.get_transaction(txid.to_string()).unwrap();
        let tx_outs = wallet
            .get_tx_out_by_address(transaction.to_string(), p2c_address.to_string())
            .unwrap();

        let another_address: String = env
            .tapyrusd
            .client
            .call("getnewaddress", &["".into(), color_id.to_string().into()])
            .unwrap();

        let ret = wallet.transfer(
            vec![TransferParams {
                amount: 300,
                to_address: another_address.clone(),
            }],
            tx_outs,
        );
        assert!(ret.is_ok());

        wait_for_confirmation(&env, &client, 1);
        wallet.sync().expect("Failed to sync");

        assert_eq!(
            wallet.balance(Some(color_id.clone().to_string())).unwrap(),
            100
        );
    }

    #[test]
    fn test_refund() {
        let (env, color_id, client) = prepare_token();
        let sender_wallet = get_wallet_testenv(&env, &client, None);
        distribute_token(&sender_wallet, &env, &color_id, &client);
        let receiver_wallet = get_wallet_testenv(&env, &client, Some("tprv8ZgxMBicQKsPfKH3fHRJGBs9Vt2hMHfroZuZ5yYLYZgwvC3Hc8Wksn1HDinon77ZvDNEo25BEefQ6Ldgi4Nw29o1gP7pY8QzAyn1WQimrdc".to_string()));
        distribute_token(&receiver_wallet, &env, &color_id, &client);

        // Receiver generate new public key and notify to the sender
        let GetNewAddressResult {
            public_key: receiver_public_key,
            ..
        } = receiver_wallet
            .get_new_address(Some(color_id.to_string()))
            .unwrap();

        // Sender creates P2C address and transfers to the P2C address
        let p2c_address = sender_wallet
            .calc_p2c_address(
                receiver_public_key.clone(),
                "content".to_string(),
                Some(color_id.clone().to_string()),
            )
            .unwrap();
        let transfer_txid = sender_wallet
            .transfer(
                vec![TransferParams {
                    amount: 10,
                    to_address: p2c_address.clone(),
                }],
                vec![],
            )
            .expect("Failed to transfer");

        wait_for_confirmation(&env, &client, 1);
        sender_wallet.sync().expect("Failed to sync");
        assert_eq!(
            sender_wallet
                .balance(Some(color_id.clone().to_string()))
                .unwrap(),
            90
        );

        // Receiver confirms the transfer
        let contract = Contract {
            contract_id: "contract_id".to_string(),
            contract: "content".to_string(),
            payment_base: receiver_public_key,
            payable: false,
        };
        receiver_wallet
            .store_contract(contract.clone())
            .expect("Failed to store contract");
        assert_eq!(
            receiver_wallet
                .balance(Some(color_id.clone().to_string()))
                .unwrap(),
            100
        );
        receiver_wallet.sync().expect("Failed to sync");
        assert_eq!(
            receiver_wallet
                .balance(Some(color_id.clone().to_string()))
                .unwrap(),
            110
        );

        // Sender check refund but it will not have any refund
        assert_eq!(
            sender_wallet
                .check_trust_layer_refund(transfer_txid.clone(), color_id.clone().to_string())
                .unwrap(),
            0
        );

        // Receiver refund the token
        let GetNewAddressResult {
            address: refund_address,
            ..
        } = sender_wallet
            .get_new_address(Some(color_id.to_string()))
            .unwrap();
        let tx = receiver_wallet
            .get_transaction(transfer_txid.clone())
            .unwrap();
        let utxo = receiver_wallet
            .get_tx_out_by_address(tx, p2c_address)
            .unwrap();
        let refund_txid = receiver_wallet
            .transfer(
                vec![TransferParams {
                    amount: 10,
                    to_address: refund_address,
                }],
                utxo,
            )
            .expect("Failed to refund");

        wait_for_confirmation(&env, &client, 1);
        sender_wallet.sync().expect("Failed to sync");
        receiver_wallet.sync().expect("Failed to sync");
        assert_eq!(
            sender_wallet
                .balance(Some(color_id.clone().to_string()))
                .unwrap(),
            100
        );
        assert_eq!(
            receiver_wallet
                .balance(Some(color_id.clone().to_string()))
                .unwrap(),
            100
        );

        // Sender check refund and it will have refund
        println!("check_trust_layer_refund");
        assert_eq!(
            sender_wallet
                .check_trust_layer_refund(transfer_txid.clone(), color_id.clone().to_string())
                .unwrap(),
            10
        );
    }

    #[test]
    fn test_sign_message() {
        let wallet = get_wallet();
        let message = "message".to_string();
        let GetNewAddressResult { public_key, .. } = wallet.get_new_address(None).unwrap();
        let sig = wallet
            .sign_message(public_key.clone(), message.clone())
            .unwrap();

        assert!(wallet
            .verify_sign(public_key.clone(), message.clone(), sig.clone())
            .unwrap());

        let message = "another message".to_string();
        assert!(!wallet
            .verify_sign(public_key.clone(), message.clone(), sig.clone())
            .unwrap());
    }

    #[test]
    fn test_sign_message_error() {
        let wallet = get_wallet();
        let message = "message".to_string();
        let public_key =
            "039be0d2b0c3b6f7fad77f142257aee12b2a34047aa3191edc0424cd15e0fa15da".to_string();
        assert_eq!(
            Err(SignMessageError::PublicKeyNotFoundInWallet),
            wallet.sign_message(public_key, message)
        );
    }

    #[test]
    fn test_verify_sign_error() {
        let wallet = get_wallet();
        let message = "message".to_string();
        let public_key =
            "039be0d2b0c3b6f7fad77f142257aee12b2a34047aa3191edc0424cd15e0fa15da".to_string();
        let invalid_sign = "invalid".to_string();

        assert_eq!(
            Err(VerifySignError::FailedToParseSignature),
            wallet.verify_sign(public_key, message, invalid_sign)
        );
    }
}
