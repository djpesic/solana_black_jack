use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::msg;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SendDeck {
    pub deck: Vec<u8>,
}

// The type of state managed by this program. The type defined here
// must match the `BlackJackAccountData` type defined by the client.

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct BlackJackAccountData {
    pub last_operation: u8, // last operation done on account
    //initial dealer cards, at the game's beginning.
    pub dealer_start1: u8,   //this card is not visible to players.
    pub dealer_start2: u8,   // this card is visible to players.
    pub player_hand: u8,     // contatins sum of the player's cards.
    pub current_card: usize, //current index inside the deck
    pub cards: Vec<u8>,      // deck of cards
}
//instruction codes. Used in program, for switching.
pub const SEND_DECK: u8 = 0;
pub const DEAL: u8 = 1;
pub const REQUEST_NEW_DECK: u8 = 2;
pub const CLEAR_DATA: u8 = 3;

//public constants
pub const CARD_NUMBER: u8 = 52;
/// Store  he received deck into the account.
pub fn unpack_send_deck(instruction_data: &[u8], account_info: &AccountInfo) {
    let send_deck_instruction = match SendDeck::try_from_slice(instruction_data) {
        Ok(sd) => sd,
        Err(_) => {
            msg!("Deserialization error");
            return;
        }
    };
    let mut account = BlackJackAccountData {
        last_operation: SEND_DECK,
        cards: send_deck_instruction.deck,
        dealer_start1: 0,
        dealer_start2: 0,
        player_hand: 0,
        current_card: 0,
    };
    account.current_card = account.cards.len() - 1;
    msg!("Received deck: {:?}", account.cards);

    match account.serialize(&mut &mut account_info.data.borrow_mut()[..]) {
        Ok(_) => {
            return;
        }
        Err(_) => {
            msg!("Account serialization error");
        }
    };
}
/// Deal the cards to the player and dealer. Game starts with this operation.
/// Cards are consumed from the deck's highest index.

pub fn unpack_deal(account_info: &AccountInfo) {
    msg!("Received deal command");
    let mut bj_account = match BlackJackAccountData::try_from_slice(&account_info.data.borrow()) {
        Ok(acc) => acc,
        Err(_) => {
            msg!("Account serialization error");
            return;
        }
    };
    match bj_account.cards.get_mut(bj_account.current_card) {
        Some(c) => {
            bj_account.dealer_start1 = *c;
            *c = 0;
            bj_account.current_card -= 1;
        }
        None => {
            msg!("No cards left to deal!");
            bj_account.last_operation = REQUEST_NEW_DECK;
            return;
        }
    };
    match bj_account.cards.get_mut(bj_account.current_card) {
        Some(c) => {
            bj_account.dealer_start2 = *c;
            *c = 0;
            bj_account.current_card -= 1;
        }
        None => {
            msg!("No cards left to deal!");
            bj_account.last_operation = REQUEST_NEW_DECK;
            return;
        }
    };
    match bj_account.cards.get_mut(bj_account.current_card) {
        Some(c) => {
            bj_account.player_hand = *c;
            *c = 0;
            bj_account.current_card -= 1;
        }
        None => {
            msg!("No cards left to deal!");
            bj_account.last_operation = REQUEST_NEW_DECK;
            return;
        }
    };
    match bj_account.cards.get_mut(bj_account.current_card) {
        Some(c) => {
            bj_account.player_hand += *c;
            *c = 0;
            bj_account.current_card -= 1;
        }
        None => {
            msg!("No cards left to deal!");
            bj_account.last_operation = REQUEST_NEW_DECK;
            return;
        }
    };
    bj_account.last_operation = DEAL;

    match bj_account.serialize(&mut &mut account_info.data.borrow_mut()[..]) {
        Ok(_) => {
            msg!("Deal finished, account: {:?}", bj_account);
            return;
        }
        Err(_) => {
            msg!("Account serialization error");
        }
    };
}

pub fn unpack_clear_data(account_info: &AccountInfo) {
    msg!("Clear account");
    let mut bj_account = match BlackJackAccountData::try_from_slice(&account_info.data.borrow()) {
        Ok(acc) => acc,
        Err(_) => {
            msg!("Account serialization error");
            return;
        }
    };
    bj_account.last_operation = CLEAR_DATA;
    bj_account.dealer_start1 = 0;
    bj_account.dealer_start1 = 2;
    bj_account.current_card = 0;
    bj_account.player_hand = 0;
    let len = bj_account.cards.len();
    bj_account.cards.clear();
    for _ in 0..len {
        bj_account.cards.push(0);
    }
    match bj_account.serialize(&mut &mut account_info.data.borrow_mut()[..]) {
        Ok(_) => {
            msg!("Clearing finished, account: {:?}", bj_account);
            return;
        }
        Err(_) => {
            msg!("Account serialization error");
        }
    };
}
