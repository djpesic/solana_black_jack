// use black_jack_client as bj_client;
// use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint, msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

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
        instructions::SEND_DECK => {
            instructions::unpack_send_deck(&instruction_data[1..], account);
        }
        instructions::DEAL => {
            instructions::unpack_deal(account);
        }
        instructions::CLEAR_DATA => {
            instructions::unpack_clear_data(account);
        }
        instructions::DEALER_HIT => {
            instructions::unpack_hit(account, instructions::DEALER_HIT);
        }
        instructions::PLAYER_HIT => {
            instructions::unpack_hit(account, instructions::PLAYER_HIT);
        }
        instructions::DEALER_STAND => {
            instructions::unpack_stand(account, instructions::DEALER_STAND);
        }
        instructions::PLAYER_STAND => {
            instructions::unpack_stand(account, instructions::PLAYER_STAND);
        }
        instructions::PLAYER_BUSTED => {
            instructions::unpack_busted(account, instructions::PLAYER_BUSTED);
        }
        instructions::DEALER_BUSTED => {
            instructions::unpack_busted(account, instructions::DEALER_BUSTED);
        }
        _ => (),
    }
    Ok(())
}
