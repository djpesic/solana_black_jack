pub mod instructions;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint, msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::instructions::*;

// Declare the programs entrypoint. The entrypoint is the function
// that will get run when the program is executed.
#[cfg(not(feature = "exclude_entrypoint"))]
entrypoint!(process_instruction);

/// Logic that runs when the program is executed.
///
/// The account passed in ought to contain a `BlackJackAccountData`.
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> entrypoint::ProgramResult {
    msg!("instruction data: {:?}", instruction_data);

    // Get the account that stores greeting count information.
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order for the
    // program to write to it. If that is not the case then the
    // program has been invoked incorrectly and we report as much.
    if account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }
    msg!("account data len: {}", account.data_len());
    msg!("account data: {:?}", &account.data.borrow());

    match instruction_data[0] {
        SEND_DECK => {
            unpack_send_deck(&instruction_data[1..], account);
        }
        DEAL => {
            unpack_deal(account);
        }
        CLEAR_DATA => {
            unpack_clear_data(account);
        }
        DEALER_HIT => {
            unpack_hit(account, DEALER_HIT);
        }
        PLAYER_HIT => {
            unpack_hit(account, PLAYER_HIT);
        }
        DEALER_STAND => {
            unpack_stand(account, DEALER_STAND);
        }
        PLAYER_STAND => {
            unpack_stand(account, PLAYER_STAND);
        }
        PLAYER_BUSTED => {
            unpack_busted(account, PLAYER_BUSTED);
        }
        DEALER_BUSTED => {
            unpack_busted(account, DEALER_BUSTED);
        }
        _ => (),
    }
    Ok(())
}
