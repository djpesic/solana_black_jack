use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::msg;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SendDeck {
    pub deck: Vec<u8>,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct BlackJackAccount {
    pub cards: Vec<u8>,
    pub sum: u8,
}

pub const SEND_DECK: u8 = 0;
pub const CARD_NUMBER: u8 = 52;

pub fn unpack_send_deck(instruction_data: &[u8], account_info: &AccountInfo) {
    let send_deck_instruction = match SendDeck::try_from_slice(instruction_data) {
        Ok(sd) => sd,
        Err(_) => {
            msg!("Deserialization error");
            return;
        }
    };
    let account = BlackJackAccount {
        cards: send_deck_instruction.deck,
        sum: 0,
    };
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
