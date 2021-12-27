extern crate rand;
use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::utils;
use crate::{Error, Result};
use borsh::BorshSerialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::Message;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::{read_keypair_file, Keypair};
use solana_sdk::transaction::Transaction;

/// Establishes a RPC connection with the solana cluster configured by
/// `solana config set --url <URL>`. Information about what cluster
/// has been configured is gleened from the solana config file
/// `~/.config/solana/cli/config.yml`.
pub fn establish_connection() -> Result<RpcClient> {
    let rpc_url = utils::get_rpc_url()?;
    Ok(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ))
}

/// Determines the amount of lamports that will be required to execute
/// this smart contract. The minimum balance is calculated assuming
/// that the user would like to make their account rent exempt.
///
/// For more information about rent see the Solana documentation
/// [here](https://docs.solana.com/implemented-proposals/rent#two-tiered-rent-regime)
pub fn get_balance_requirement(connection: &RpcClient) -> Result<u64> {
    let account_fee =
        connection.get_minimum_balance_for_rent_exemption(utils::get_blackjack_data_size()?)?;

    let latest_hash = match connection.get_latest_blockhash() {
        Ok(hash) => hash,
        Err(_) => {
            return Err(Error::LatestBlockHashError(String::from(
                "Can't fetch latest block hash",
            )));
        }
    };
    let fee_calculator = match connection.get_fee_calculator_for_blockhash(&latest_hash) {
        Ok(calc) => match calc {
            Some(calc) => calc,
            None => {
                return Err(Error::FeeCaluclatorError(String::from(
                    "Can't fetch fee caluclator for blockhash",
                )));
            }
        },
        Err(_) => {
            return Err(Error::FeeCaluclatorError(String::from(
                "Can't fetch fee caluclator for blockhash",
            )));
        }
    };
    let transaction_fee = fee_calculator.lamports_per_signature * 100;

    Ok(transaction_fee + account_fee)
}

/// Gets the balance of PLAYER in lamports via a RPC call over
/// CONNECTION.
pub fn get_player_balance(player: &Keypair, connection: &RpcClient) -> Result<u64> {
    Ok(connection.get_balance(&player.pubkey())?)
}

/// Requests that AMOUNT lamports are transfered to PLAYER via a RPC
/// call over CONNECTION.
///
/// Airdrops are only avaliable on test networks.
pub fn request_airdrop(player: &Keypair, connection: &RpcClient, amount: u64) -> Result<()> {
    let sig = connection.request_airdrop(&player.pubkey(), amount)?;
    loop {
        let confirmed = connection.confirm_transaction(&sig)?;
        if confirmed {
            break;
        }
    }
    Ok(())
}

/// Loads keypair information from the file located at KEYPAIR_PATH
/// and then verifies that the loaded keypair information corresponds
/// to an executable account via CONNECTION. Failure to read the
/// keypair or the loaded keypair corresponding to an executable
/// account will result in an error being returned.
pub fn get_program(keypair_path: &str, connection: &RpcClient) -> Result<Keypair> {
    let program_keypair = read_keypair_file(keypair_path).map_err(|e| {
        Error::InvalidConfig(format!(
            "failed to read program keypair file ({}): ({})",
            keypair_path, e
        ))
    })?;

    let program_info = connection.get_account(&program_keypair.pubkey())?;
    if !program_info.executable {
        return Err(Error::InvalidConfig(format!(
            "program with keypair ({}) is not executable",
            keypair_path
        )));
    }

    Ok(program_keypair)
}

///
/// The  account has a [derived
/// address](https://docs.solana.com/developing/programming-model/calling-between-programs#program-derived-addresses)
/// which allows it to own and manage the account. Additionally the
/// address being derived means that we can regenerate it when we'd
/// like to find the  account again later.
pub fn create_blackjack_account(
    player: &Keypair,
    program: &Keypair,
    connection: &RpcClient,
) -> Result<()> {
    let account_pubkey = utils::get_account_public_key(&player.pubkey(), &program.pubkey())?;

    if let Err(_) = connection.get_account(&account_pubkey) {
        println!("creating blackjack account");
        let lamport_requirement =
            connection.get_minimum_balance_for_rent_exemption(utils::get_blackjack_data_size()?)?;

        // This instruction creates an account with the key
        // "account_pubkey". The created account is owned by the
        // program. The account is loaded with enough lamports to stop
        // it from needing to pay rent. The lamports to fund this are
        // paid by the player.
        //
        // It is important that the program owns the created account
        // because it needs to be able to modify its contents.
        //
        // The address of the account created by
        // create_account_with_seed is the same as the address
        // generated by utils::get_account_public_key. We do this as
        // opposed to create_account because create account doesn't
        // derive that address like that.
        let instruction = solana_sdk::system_instruction::create_account_with_seed(
            &player.pubkey(),
            &account_pubkey,
            &player.pubkey(),
            utils::get_account_seed(),
            lamport_requirement,
            utils::get_blackjack_data_size()? as u64,
            &program.pubkey(),
        );
        let message = Message::new(&[instruction], Some(&player.pubkey()));
        let latest_hash = match connection.get_latest_blockhash() {
            Ok(hash) => hash,
            Err(_) => {
                return Err(Error::LatestBlockHashError(String::from(
                    "Can't fetch latest block hash",
                )));
            }
        };
        let transaction = Transaction::new(&[player], message, latest_hash);

        connection.send_and_confirm_transaction(&transaction)?;
    }

    Ok(())
}

/// Sends shuffled deck of cards as an instruction from PLAYER to PROGRAM via CONNECTION.
pub fn send_deck(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<()> {
    let deck = generate_deck();
    //serialize deck
    let encoded = utils::BlackJackAccountSchema::new(deck)
        .try_to_vec()
        .map_err(|e| Error::SerializationError(e))?;

    let black_jack_account_pub_key =
        utils::get_account_public_key(&player.pubkey(), &program.pubkey())?;

    // Submit an instruction to the chain which tells the program to
    // run. We pass the account that we want the results to be stored
    // in as one of the accounts arguments which the program will
    // handle. Instruction also contains serialized deck of cards, and solana program public key.

    let instruction = Instruction::new_with_bytes(
        program.pubkey(),
        &encoded,
        vec![AccountMeta::new(black_jack_account_pub_key, false)],
    );
    let message = Message::new(&[instruction], Some(&player.pubkey()));
    let latest_hash = match connection.get_latest_blockhash() {
        Ok(hash) => hash,
        Err(_) => {
            return Err(Error::LatestBlockHashError(String::from(
                "Can't fetch latest block hash",
            )));
        }
    };
    let transaction = Transaction::new(&[player], message, latest_hash);

    connection.send_and_confirm_transaction(&transaction)?;

    Ok(())
}
/// Generate one classic deck of 52 cards and shuffle it.

pub fn generate_deck() -> Vec<u8> {
    let mut range: Vec<u8> = (1..11).collect();
    let mut result: Vec<u8> = Vec::new();
    //four colours (spade, heart, diamond, club)
    for _i in 0..4 {
        result.append(&mut range);
        result.push(12);
        result.push(13);
        result.push(14);
    }

    let mut rng = thread_rng();
    result.shuffle(&mut rng);
    println!("Generated deck: {:?}", result);
    result
}
