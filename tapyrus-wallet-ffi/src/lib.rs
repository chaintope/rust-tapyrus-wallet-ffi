use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::{fs, io};
use tdk_esplora::esplora_client;
use tdk_esplora::esplora_client::deserialize;
use tdk_esplora::EsploraExt;
use tdk_sqlite::{rusqlite::Connection, Store};
use tdk_wallet::descriptor::Descriptor;
use tdk_wallet::tapyrus::bip32::Xpriv;
use tdk_wallet::tapyrus::consensus::serialize;
use tdk_wallet::tapyrus::hex::{DisplayHex, FromHex};
use tdk_wallet::tapyrus::script::color_identifier::ColorIdentifier;
use tdk_wallet::tapyrus::secp256k1::rand::Rng;
use tdk_wallet::tapyrus::{secp256k1, Address, BlockHash, PublicKey, ScriptBuf};
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
            db_file_path,
        }
    }
}

pub(crate) struct HdWallet {
    network: tapyrus::network::Network,
    wallet: Mutex<Wallet>,
    esplora_url: String,
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

pub(crate) struct Contract {
    pub contract_id: String,
    pub contract: String,
    pub payment_base: String,
    pub payable: bool,
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
    LoadMasterKeyError,
    LoadWalletDBError {
        cause: String,
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
}

impl Display for NewError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NewError::LoadMasterKeyError => write!(f, "Failed to load master key"),
            NewError::LoadWalletDBError { cause: e } => {
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
        }
    }
}

impl std::error::Error for NewError {}

#[derive(Debug)]
pub(crate) enum SyncError {
    EsploraClientError { cause: String },
    UpdateWalletError { cause: String },
}

impl Display for SyncError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncError::EsploraClientError { cause: e } => write!(f, "Esplora client error: {}", e),
            SyncError::UpdateWalletError { cause: e } => {
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
    EsploraClient { cause: String },
    FailedToParseAddress { address: String },
    WrongNetworkAddress { address: String },
    FailedToParseTxid { txid: String },
    InvalidTransferAmount { cause: String },
    UnknownUtxo { utxo: TxOut },
    FailedToCreateTransaction { cause: String },
}

impl Display for TransferError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferError::InsufficientFund => write!(f, "Insufficient fund"),
            TransferError::EsploraClient { cause: e } => write!(f, "Esplora client error: {}", e),
            TransferError::FailedToParseAddress { address: e } => {
                write!(f, "Failed to parse address: {}", e)
            }
            TransferError::WrongNetworkAddress { address: e } => {
                write!(f, "Wrong network address: {}", e)
            }
            TransferError::FailedToParseTxid { txid: e } => {
                write!(f, "Failed to parse txid: {}", e)
            }
            TransferError::InvalidTransferAmount { cause: e } => {
                write!(f, "Invalid transfer amount: {}", e)
            }
            TransferError::UnknownUtxo { utxo: e } => write!(f, "Unknown utxo: {:?}", e),
            TransferError::FailedToCreateTransaction { cause: e } => {
                write!(f, "Failed to create transaction: {}", e)
            }
        }
    }
}

impl std::error::Error for TransferError {}

#[derive(Debug)]
pub(crate) enum GetTransactionError {
    FailedToParseTxid { txid: String },
    EsploraClientError { cause: String },
    UnknownTxid,
}

impl Display for GetTransactionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetTransactionError::FailedToParseTxid { txid: e } => {
                write!(f, "Failed to parse txid: {}", e)
            }
            GetTransactionError::EsploraClientError { cause: e } => {
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
        cause: String,
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
            GetTxOutByAddressError::EsploraClientError { cause: e } => {
                write!(f, "Esplora client error: {}", e)
            }
            GetTxOutByAddressError::UnknownTransaction => write!(f, "Unknown transaction"),
        }
    }
}

impl std::error::Error for GetTxOutByAddressError {}

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
            db_file_path,
        } = config.as_ref();

        let network: tapyrus::network::Network = network_mode.clone().into();

        let master_key_path = master_key_path
            .clone()
            .unwrap_or_else(|| "master_key".to_string());
        let master_key = initialize_or_load_master_key(&master_key_path, network)
            .map_err(|_| NewError::LoadMasterKeyError)?;

        let db_path = db_file_path
            .clone()
            .unwrap_or_else(|| "tapyrus-wallet.sqlite".to_string());
        let conn = Connection::open(&db_path).map_err(|e| NewError::LoadWalletDBError {
            cause: e.to_string(),
        })?;
        let db = Store::new(conn).map_err(|e| NewError::LoadWalletDBError {
            cause: e.to_string(),
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
                cause: e.to_string(),
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
            _ => {
                panic!("Unexpected error: {:?}", e)
            }
        })?;

        Ok(HdWallet {
            network,
            wallet: Mutex::new(wallet),
            esplora_url: esplora_url.clone(),
        })
    }

    pub fn sync(&self) -> Result<(), SyncError> {
        let mut wallet = self.get_wallet();
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();

        let request = wallet.start_sync_with_revealed_spks();
        let update = client.sync(request, SYNC_PARALLEL_REQUESTS).map_err(|e| {
            SyncError::EsploraClientError {
                cause: e.to_string(),
            }
        })?;

        wallet
            .apply_update(update)
            .map_err(|e| SyncError::UpdateWalletError {
                cause: e.to_string(),
            })?;
        Ok(())
    }

    pub fn full_sync(&self) -> Result<(), SyncError> {
        let mut wallet = self.get_wallet();
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();

        let request = wallet.start_full_scan();
        let update = client
            .full_scan(request, STOP_GAP, SYNC_PARALLEL_REQUESTS)
            .map_err(|e| SyncError::EsploraClientError {
                cause: e.to_string(),
            })?;

        wallet
            .apply_update(update)
            .map_err(|e| SyncError::UpdateWalletError {
                cause: e.to_string(),
            })?;
        Ok(())
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
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();

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
                    cause: e.to_string(),
                })?;
        wallet
            .sign(&mut psbt, SignOptions::default())
            .map_err(|e| TransferError::FailedToCreateTransaction {
                cause: e.to_string(),
            })?;
        let tx = psbt
            .extract_tx()
            .map_err(|e| TransferError::FailedToCreateTransaction {
                cause: e.to_string(),
            })?;
        client
            .broadcast(&tx)
            .map_err(|e| TransferError::EsploraClient {
                cause: e.to_string(),
            })?;

        Ok(tx.malfix_txid().to_string())
    }

    pub fn get_transaction(&self, txid: String) -> Result<String, GetTransactionError> {
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();
        let txid = txid
            .parse::<MalFixTxid>()
            .map_err(|_| GetTransactionError::FailedToParseTxid { txid })?;
        let tx = client
            .get_tx(&txid)
            .map_err(|e| GetTransactionError::EsploraClientError {
                cause: e.to_string(),
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
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();

        tx.output
            .iter()
            .enumerate()
            .try_fold(Vec::new(), |mut acc, (i, o)| {
                if o.script_pubkey == script_pubkey {
                    let status = client
                        .get_output_status(&tx.malfix_txid(), i as u64)
                        .map_err(|e| GetTxOutByAddressError::EsploraClientError {
                            cause: e.to_string(),
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
    ) -> String {
        "15Q1z9LJGeaU6oHeEvT1SKoeCUJntZZ9Tg".to_string()
    }

    pub fn store_contract(&self, contract: Contract) -> () {
        ()
    }

    pub fn update_contract(
        &self,
        contract_id: String,
        contract: Option<String>,
        payment_base: Option<String>,
        payable: Option<bool>,
    ) -> () {
        ()
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

uniffi::include_scaffolding!("wallet");

#[cfg(test)]
mod test {
    use crate::*;
    use std::hash::Hash;
    use std::thread;
    use tdk_wallet::tapyrus::PubkeyHash;

    fn get_wallet() -> HdWallet {
        let config = Config {
            network_mode: Network::Prod,
            network_id: 1939510133,
            genesis_hash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a"
                .to_string(),
            esplora_url: "http://localhost:3001".to_string(),
            esplora_user: None,
            esplora_password: None,
            master_key_path: Some("tests/master_key".to_string()),
            db_file_path: Some("tests/tapyrus-wallet.sqlite".to_string()),
        };
        HdWallet::new(Arc::new(config)).unwrap()
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
    #[ignore] // This test is for manual testing with esplora-tapyrus.
    fn test_with_esplora() {
        let wallet = get_wallet();
        wallet.full_sync().expect("Failed to sync");
        assert!(wallet.balance(None).unwrap() > 0, "{}",
                format!("TPC Balance should be greater than 0. Charge TPC from faucet (https://testnet-faucet.tapyrus.dev.chaintope.com/tapyrus/transactions) to Address: {}", wallet.get_new_address(None).unwrap().address)
        );

        println!("balance: {}", wallet.balance(None).unwrap());

        // transfer TPC to faucet
        let txid = wallet.transfer(
            vec![TransferParams {
                amount: 1000,
                to_address: "1LxWufmUothBSe78DYESKcoP8ppmPcSHZ6".to_string(),
            }],
            Vec::new(),
        );

        let color_id = ColorIdentifier::from_str(
            "c14ca2241021165f86cf706351de7e235d7f4b4895fcb4d9155a4e9245f95c2c9a",
        )
        .unwrap();
        let balance = wallet.balance(Some(color_id.to_string())).unwrap();
        assert_eq!(balance, 100, "Balance should be 100");
    }

    #[test]
    #[ignore] // This test is for manual testing with esplora-tapyrus.
    fn test_colored_coin_with_esplora() {
        let wallet = get_wallet();
        wallet.full_sync().expect("Failed to sync");

        let color_id = ColorIdentifier::from_str(
            "c14ca2241021165f86cf706351de7e235d7f4b4895fcb4d9155a4e9245f95c2c9a",
        )
        .unwrap();
        let balance = wallet.balance(Some(color_id.to_string())).unwrap();
        assert_eq!(balance, 100, "Balance should be 100");

        let to_address = wallet
            .get_new_address(Some(color_id.to_string()))
            .unwrap()
            .address;

        let txid = wallet
            .transfer(
                vec![TransferParams {
                    amount: 1,
                    to_address: to_address.clone(),
                }],
                Vec::new(),
            )
            .expect("Failed to transfer");

        // wait for transaction to be indexed
        let tx = loop {
            match wallet.get_transaction(txid.clone()) {
                Ok(tx) => break tx,
                Err(_) => thread::sleep(std::time::Duration::from_secs(1)),
            }
        };
        wallet.sync().expect("Failed to sync");
        let txout = wallet
            .get_tx_out_by_address(tx, to_address.clone())
            .unwrap();

        let another_address = wallet
            .get_new_address(Some(color_id.to_string()))
            .unwrap()
            .address;
        let txid = wallet
            .transfer(
                vec![TransferParams {
                    amount: 1,
                    to_address: another_address,
                }],
                txout,
            )
            .expect("Failed to transfer");
    }

    #[test]
    #[ignore] // This test is for manual testing with esplora-tapyrus.
    fn test_get_transaction() {
        let wallet = get_wallet();
        let txid = "97ca7f039b37444f22bea129a0454cf0c6677dd7176d238354f97a9ce10dc9af".to_string();
        let transaction = wallet.get_transaction(txid).unwrap();
        assert_eq!(transaction, "0100000001c0b8f338a48956d79dd8ed25673549bbc4d3e65e1f8ddb8edaff2dbf7daaf2c4000000006a47304402200e9d92b9009928deb8deceb88635df25e2162a689ec6be73bb81a846fa3667ed0220358077f7f5026bc49f77e1cca97e5b3e13a8697c75fbe12bdd276221f0a6d963012103d32aaa4e44a7b93ac517f697b901d4261581102d2a0c828935ce539b9f6574d1feffffff02b9b90000000000001976a914947424e58166cbb152df9216b8e6139c77655d1488ace8030000000000001976a914daea3bd9f5ca2d301b35db233cf79c49b65a4b9b88ac771b0700", "Transaction should be equal");
    }

    #[test]
    #[ignore] // This test is for manual testing with esplora-tapyrus.
    fn test_get_tx_out_by_address() {
        let wallet = get_wallet();
        let tx = "0100000001c0b8f338a48956d79dd8ed25673549bbc4d3e65e1f8ddb8edaff2dbf7daaf2c4000000006a47304402200e9d92b9009928deb8deceb88635df25e2162a689ec6be73bb81a846fa3667ed0220358077f7f5026bc49f77e1cca97e5b3e13a8697c75fbe12bdd276221f0a6d963012103d32aaa4e44a7b93ac517f697b901d4261581102d2a0c828935ce539b9f6574d1feffffff02b9b90000000000001976a914947424e58166cbb152df9216b8e6139c77655d1488ace8030000000000001976a914daea3bd9f5ca2d301b35db233cf79c49b65a4b9b88ac771b0700";

        let txouts = wallet.get_tx_out_by_address(
            tx.to_string(),
            "1LxWufmUothBSe78DYESKcoP8ppmPcSHZ6".to_string(),
        );
        assert_eq!(txouts.unwrap().len(), 1, "TxOut should be 1");
    }
}
