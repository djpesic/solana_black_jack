extern crate rand;
use borsh::{BorshDeserialize, BorshSerialize};
use rand::seq::SliceRandom;
use rand::thread_rng;
use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::Message;
use solana_sdk::signature::Signer;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::transaction::Transaction;
use utils::{Error, Result};

/// Sends shuffled deck of cards as an instruction from PLAYER to PROGRAM via CONNECTION.
pub fn send_deck(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<()> {
    let deck = generate_deck();
    //serialize deck
    let mut encoded_deck: Vec<u8> = Vec::new();
    encoded_deck.push(utils::SEND_DECK);
    println!("Serialize deck");
    if let Err(_) = (utils::SendDeck { deck: deck }.serialize(&mut encoded_deck)) {
        return Err(utils::Error::Error(String::from(
            "Deck serialization error",
        )));
    }
    println!("Serialized deck len: {}", encoded_deck.len());
    println!("Serialized deck: {:?}", encoded_deck);
    send(player, program, connection, &encoded_deck)
}

fn send(player: &Keypair, program: &Keypair, connection: &RpcClient, data: &Vec<u8>) -> Result<()> {
    let black_jack_account_pub_key =
        utils::get_account_public_key(&player.pubkey(), &program.pubkey())?;

    // Submit an instruction to the chain which tells the program to
    // run. We pass the account that we want the results to be stored
    // in as one of the accounts arguments which the program will
    // handle. Instruction also contains serialized deck of cards, and solana program public key.
    let instruction = Instruction::new_with_bytes(
        program.pubkey(),
        &data,
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
    println!("Send transaction");
    connection.send_and_confirm_transaction(&transaction)?;
    Ok(())
}

/// Generate one classic deck of 52 cards and shuffle it.

fn generate_deck() -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();
    //four colours (spade, heart, diamond, club)
    for _j in 0..4 {
        for i in 1..15 {
            if i != 11 {
                result.push(i);
            }
        }
    }

    let mut rng = thread_rng();
    result.shuffle(&mut rng);
    println!("Generated deck: {:?}", result);
    result
}
/// Init deal operation. Dealing will be done inside the on-chain program.
pub fn deal(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<()> {
    let mut data: Vec<u8> = Vec::new();
    data.push(utils::DEAL);
    println!("Init dealing.");
    send(player, program, connection, &data)
}

/// Init clear operation. Clearing will be done inside the on-chain program.
pub fn clear_data(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<()> {
    let mut data: Vec<u8> = Vec::new();
    data.push(utils::CLEAR_DATA);
    println!("Init clearing data.");
    send(player, program, connection, &data)
}

/// Get dealer's faced-up card.
pub fn get_init_status(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<u8> {
    let bj_pubkey = utils::get_account_public_key(&player.pubkey(), &program.pubkey())?;
    let account = connection.get_account(&bj_pubkey)?;
    let account_data = utils::BlackJackAccountDataSchema::try_from_slice(&account.data)
        .map_err(|e| Error::SerializationError(e))?;
    println!("Dealer faced up card is {}", account_data.dealer_start2);
    println!("Sum of initial player hand is {}", account_data.player_hand);
    Ok(account_data.dealer_start2)
}

pub fn is_deck_dealt(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<bool> {
    println!("Check if deck is already dealt");
    let bj_pubkey = utils::get_account_public_key(&player.pubkey(), &program.pubkey())?;
    let account = connection.get_account(&bj_pubkey)?;
    let account_data = utils::BlackJackAccountDataSchema::try_from_slice(&account.data)
        .map_err(|e| Error::SerializationError(e))?;
    println!("Deck dealt: {}", account_data.last_operation == utils::DEAL);
    Ok(account_data.last_operation == utils::DEAL)
}

/// Init hit game action. Procedure will be done on the onchain program.
pub fn hit(
    player: &Keypair,
    program: &Keypair,
    connection: &RpcClient,
    operation: u8,
) -> Result<()> {
    let mut data: Vec<u8> = Vec::new();
    data.push(operation);
    println!("Init hit game action.");
    send(player, program, connection, &data)
}

/// Stand game action. Player ends game, and saves collected score.
pub fn stand(
    player: &Keypair,
    program: &Keypair,
    connection: &RpcClient,
    operation: u8,
) -> Result<()> {
    let mut data: Vec<u8> = Vec::new();
    data.push(operation);
    println!("Init hit game action.");
    send(player, program, connection, &data)
}

/// Player is busted.
pub fn busted(player: &Keypair, program: &Keypair, connection: &RpcClient) -> Result<()> {
    let mut data: Vec<u8> = Vec::new();
    data.push(utils::PLAYER_BUSTED);
    println!("Init busted game action.");
    send(player, program, connection, &data)
}
