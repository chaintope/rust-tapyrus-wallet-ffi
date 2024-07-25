use std::fmt::Debug;
use std::fs::File;
use std::io::{Read, Write};
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};
use std::{fs, io};
use tdk_esplora::esplora_client;
use tdk_esplora::EsploraExt;
use tdk_sqlite::{rusqlite::Connection, Store};
use tdk_wallet::tapyrus::bip32::Xpriv;
use tdk_wallet::tapyrus::consensus::serialize;
use tdk_wallet::tapyrus::hex::{DisplayHex, FromHex};
use tdk_wallet::tapyrus::script::color_identifier::ColorIdentifier;
use tdk_wallet::tapyrus::secp256k1::rand::Rng;
use tdk_wallet::tapyrus::MalFixTxid;
use tdk_wallet::tapyrus::{secp256k1, Address, BlockHash};
use tdk_wallet::template::Bip44;
use tdk_wallet::{tapyrus, KeychainKind, Wallet};

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

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub network_mode: Network,
    pub network_id: u32,
    pub genesis_hash: String,
    pub esplora_host: String,
    pub esplora_port: u32,
    pub esplora_user: Option<String>,
    pub esplora_password: Option<String>,
    pub master_key_path: Option<String>,
    pub db_file_path: Option<String>,
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

struct TxOut {
    pub txid: String,
    pub index: u32,
    pub amount: u64,
    pub color_id: Option<String>,
    pub address: String,
    pub unspent: bool,
}

struct Contract {
    pub contract_id: String,
    pub contract: String,
    pub payment_base: String,
    pub payable: bool,
}

const SYNC_PARALLEL_REQUESTS: usize = 1;

impl HdWallet {
    pub fn new(config: Config) -> Self {
        let network: tapyrus::network::Network = config.clone().network_mode.into();

        let master_key_path = config
            .master_key_path
            .unwrap_or_else(|| "master_key".to_string());
        let master_key = initialize_or_load_master_key(&master_key_path, network).unwrap();

        let db_path = config
            .db_file_path
            .unwrap_or_else(|| "tapyrus-wallet.sqlite".to_string());
        let conn = Connection::open(&db_path).unwrap();
        let db = Store::new(conn).unwrap();

        let genesis_hash = BlockHash::from_str(&config.genesis_hash).unwrap();

        let wallet = Wallet::new_or_load_with_genesis_hash(
            Bip44(master_key, KeychainKind::External),
            Bip44(master_key, KeychainKind::Internal),
            db,
            network,
            genesis_hash,
        )
        .unwrap();

        HdWallet {
            network,
            wallet: Mutex::new(wallet),
            esplora_url: format!("http://{}:{}", config.esplora_host, config.esplora_port),
        }
    }

    pub fn sync(&self) -> () {
        let mut wallet = self.get_wallet();
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();

        let request = wallet.start_sync_with_revealed_spks();
        let update = client.sync(request, SYNC_PARALLEL_REQUESTS).unwrap();

        wallet.apply_update(update).expect("update failed");
    }

    fn get_wallet(&self) -> MutexGuard<Wallet> {
        self.wallet.lock().expect("Failed to lock wallet")
    }

    pub fn get_new_address(&self, color_id: Option<String>) -> String {
        let address = self
            .get_wallet()
            .reveal_next_address(KeychainKind::External)
            .unwrap();

        if let Some(color_id) = color_id {
            let color_id = ColorIdentifier::from_str(&color_id).unwrap();
            let script = address.script_pubkey().add_color(color_id).unwrap();
            let address = Address::from_script(&script, self.network).unwrap();
            return address.to_string();
        }

        return address.to_string();
    }

    pub fn balance(&self, color_id: Option<String>) -> u64 {
        let color_id = if let Some(color_id) = color_id {
            ColorIdentifier::from_str(&color_id).unwrap()
        } else {
            ColorIdentifier::default()
        };
        let balance = self.get_wallet().balance(color_id);
        balance.total().to_tap()
    }

    pub fn transfer(&self, params: Vec<TransferParams>, utxos: Vec<TxOut>) -> String {
        "2fa3170debe6bdcd98f2ef1fb0dc1368693b5ace4c8eabf549cb6c44616c2819".to_string()
    }

    pub fn get_transaction(&self, txid: String) -> String {
        let client = esplora_client::Builder::new(&self.esplora_url).build_blocking();
        let txid = txid.parse::<MalFixTxid>().unwrap();
        let tx = client.get_tx(&txid).unwrap();
        match tx {
            Some(tx) => serialize(&tx).to_lower_hex_string(),
            None => "".to_string(),
        }
    }

    pub fn get_tx_out_by_address(&self, tx: String, address: String) -> Vec<TxOut> {
        let mut r = Vec::new();
        r.push(TxOut {
            txid: "2fa3170debe6bdcd98f2ef1fb0dc1368693b5ace4c8eabf549cb6c44616c2819".to_string(),
            index: 0,
            amount: 10,
            color_id: Option::<String>::None,
            address: "15Q1z9LJGeaU6oHeEvT1SKoeCUJntZZ9Tg".to_string(),
            unspent: false,
        });
        r
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

    fn get_wallet() -> HdWallet {
        let config = Config {
            network_mode: Network::Prod,
            network_id: 1939510133,
            genesis_hash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a"
                .to_string(),
            esplora_host: "localhost".to_string(),
            esplora_port: 3001,
            esplora_user: None,
            esplora_password: None,
            master_key_path: None,
            db_file_path: None,
        };
        HdWallet::new(config)
    }

    #[test]
    fn test_get_new_address() {
        let wallet = get_wallet();
        let address = wallet.get_new_address(None);
        assert_eq!(address.len(), 34, "Address should be 34 characters long");

        let color_id = ColorIdentifier::from_str(
            "c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46",
        )
        .unwrap();
        let address = wallet.get_new_address(Some(color_id.to_string()));
        assert_eq!(address.len(), 78, "Address should be 78 characters long");
    }

    #[test]
    fn test_balance() {
        let wallet = get_wallet();
        let balance = wallet.balance(None);
        assert_eq!(balance, 0, "Balance should be 0");

        let color_id = ColorIdentifier::from_str(
            "c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46",
        )
        .unwrap();
        let balance = wallet.balance(Some(color_id.to_string()));
        assert_eq!(balance, 0, "Balance should be 0");
    }
}
