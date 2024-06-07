#[derive(PartialEq, Debug)]
pub(crate) enum Network {
    Prod,
    Dev,
}

pub(crate) struct Config {
    pub network_mode: Network,
    pub network_id: u32,
    pub esplora_host: String,
    pub esplora_port: u32,
    pub esplora_user: Option<String>,
    pub esplora_password: Option<String>,
}

pub(crate) struct HdWallet {}

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
        HdWallet {}
    }

    pub fn get_new_address(&self, color_id: Option<String>) -> String {
        "15Q1z9LJGeaU6oHeEvT1SKoeCUJntZZ9Tg".to_string()
    }

    pub fn balance(&self, color_id: Option<String>) -> u64 {
        100
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
