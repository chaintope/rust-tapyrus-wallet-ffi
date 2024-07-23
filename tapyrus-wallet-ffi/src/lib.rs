use std::fmt::Debug;
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};
use tdk_wallet::{KeychainKind, tapyrus, Wallet};
use tdk_wallet::tapyrus::bip32::Xpriv;
use tdk_wallet::tapyrus::hex::FromHex;
use tdk_wallet::tapyrus::script::color_identifier::ColorIdentifier;
use tdk_wallet::tapyrus::{Address, secp256k1};
use tdk_wallet::tapyrus::secp256k1::rand::Rng;
use tdk_wallet::template::Bip44;

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
    pub esplora_host: String,
    pub esplora_port: u32,
    pub esplora_user: Option<String>,
    pub esplora_password: Option<String>,
}

pub(crate) struct HdWallet {
    network: tapyrus::network::Network,
    wallet: Mutex<Wallet>,
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

impl HdWallet {
    pub fn new(config: Config) -> Self {
        let network: tapyrus::network::Network = config.clone().network_mode.into();

        let seed: [u8; 32] = secp256k1::rand::thread_rng().gen();
        let root = Xpriv::new_master(network, &seed).unwrap();
        let wallet = Wallet::new_no_persist( // TODO: persist in SQLite3 database.
            Bip44(root, KeychainKind::External),
            Bip44(root, KeychainKind::Internal),
            network
        ).unwrap();

        HdWallet {
            network,
            wallet: Mutex::new(wallet),
        }
    }

    fn get_wallet(&self) -> MutexGuard<Wallet> {
        self.wallet.lock().expect("Failed to lock wallet")
    }

    pub fn get_new_address(&self, color_id: Option<String>) -> String {
        let address = self.get_wallet().reveal_next_address(KeychainKind::External).unwrap();

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
        "01000000011e86d7726322a1af403815466e44465bd6f119919a20680009b47b4ae00192a5210000006441f09130c3181d20273923f00544e398f4d51315bde28cd4a292d0acda92e9e7ba22c6767c7780828dbf0955add4615f9a2781672ed1afbb8b599a638b20b88ae60121039a77f4e4e45847e413617099b1b4e26d73f372d824432db3c005cabab28c4cccffffffff01d0070000000000001976a914c6e613b40de534b908a283c410f1847943eb629888ac00000000".to_string()
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

uniffi::include_scaffolding!("wallet");

#[cfg(test)]
mod test {
    use crate::*;

    fn get_wallet() -> HdWallet {
        let config = Config {
            network_mode: Network::Dev,
            network_id: 1,
            esplora_host: "localhost".to_string(),
            esplora_port: 50001,
            esplora_user: None,
            esplora_password: None,
        };
        HdWallet::new(config)
    }

    #[test]
    fn test_get_new_address() {
        let wallet = get_wallet();
        let address = wallet.get_new_address(None);
        assert_eq!(address.len(), 34, "Address should be 34 characters long");

        let color_id = ColorIdentifier::from_str("c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46").unwrap();
        let address = wallet.get_new_address(Some(color_id.to_string()));
        assert_eq!(address.len(), 80, "Address should be 80 characters long");
    }

    #[test]
    fn test_balance() {
        let wallet = get_wallet();
        let balance = wallet.balance(None);
        assert_eq!(balance, 0, "Balance should be 0");

        let color_id = ColorIdentifier::from_str("c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46").unwrap();
        let balance = wallet.balance(Some(color_id.to_string()));
        assert_eq!(balance, 0, "Balance should be 0");
    }
}