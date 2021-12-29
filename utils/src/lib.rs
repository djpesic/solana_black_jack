use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use thiserror::Error;
use yaml_rust::YamlLoader;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read solana config file: ({0})")]
    ConfigReadError(std::io::Error),
    #[error("failed to parse solana config file: ({0})")]
    ConfigParseError(#[from] yaml_rust::ScanError),
    #[error("invalid config: ({0})")]
    InvalidConfig(String),

    #[error("serialization error: ({0})")]
    SerializationError(std::io::Error),
    #[error("serialization error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),
    #[error("error in public key derivation: ({0})")]
    KeyDerivationError(#[from] solana_sdk::pubkey::PubkeyError),

    #[error("error in fetching latest block hash: ({0})")]
    LatestBlockHashError(String),

    #[error("error in fetching fee calculator: ({0})")]
    FeeCaluclatorError(String),
    #[error("Custom error: ({0})")]
    Error(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// The schema for storage in blackjac accounts. This is what
/// is serialized into the account and later updated.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct BlackJackAccountSchema {
    cards: Vec<u8>,
}

impl BlackJackAccountSchema {
    pub fn new(cards: Vec<u8>) -> BlackJackAccountSchema {
        BlackJackAccountSchema { cards: cards }
    }

    pub fn get_cards(&self) -> &Vec<u8> {
        &self.cards
    }
}

/// Parses and returns the Solana yaml config on the system.
pub fn get_config() -> Result<yaml_rust::Yaml> {
    let path = match home::home_dir() {
        Some(mut path) => {
            path.push(".config/solana/cli/config.yml");
            path
        }
        None => {
            return Err(Error::ConfigReadError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "failed to locate homedir and thus can not locate solana config",
            )));
        }
    };
    let config = std::fs::read_to_string(path).map_err(|e| Error::ConfigReadError(e))?;
    let mut config = YamlLoader::load_from_str(&config)?;
    match config.len() {
        1 => Ok(config.remove(0)),
        l => Err(Error::InvalidConfig(format!(
            "expected one yaml document got ({})",
            l
        ))),
    }
}

/// Gets the RPC url for the cluster that this machine is configured
/// to communicate with.
pub fn get_rpc_url() -> Result<String> {
    let config = get_config()?;
    match config["json_rpc_url"].as_str() {
        Some(s) => Ok(s.to_string()),
        None => Err(Error::InvalidConfig(
            "missing `json_rpc_url` field".to_string(),
        )),
    }
}

/// Gets the "client wallet" or local solana wallet that has been configured
/// on the machine.
pub fn get_local_wallet() -> Result<Keypair> {
    let config = get_config()?;
    let path = match config["keypair_path"].as_str() {
        Some(s) => s,
        None => {
            return Err(Error::InvalidConfig(
                "missing `keypair_path` field".to_string(),
            ))
        }
    };
    read_keypair_file(path).map_err(|e| {
        Error::InvalidConfig(format!("failed to read keypair file ({}): ({})", path, e))
    })
}

/// Gets the seed used to generate accounts. If you'd like to
/// force this program to generate a new  account you can change this value.
pub fn get_account_seed() -> &'static str {
    "black_jack"
}

/// Derives and returns the account public key for a given
/// CLIENT, PROGRAM combination.
pub fn get_account_public_key(player: &Pubkey, program: &Pubkey) -> Result<Pubkey> {
    Ok(Pubkey::create_with_seed(
        player,
        get_account_seed(),
        program,
    )?)
}

/// Determines and reports the size of blackjack account data.
pub fn get_blackjack_data_size() -> Result<usize> {
    let mut vec = Vec::new();
    for _i in 1..52 {
        vec.push(0);
    }
    let encoded = BlackJackAccountSchema { cards: vec }
        .try_to_vec()
        .map_err(|e| Error::SerializationError(e))?;
    Ok(encoded.len())
}
