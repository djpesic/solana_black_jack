use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct SendDeck {
    pub deck: Vec<u8>,
}

pub const SEND_DECK: u8 = 0;
