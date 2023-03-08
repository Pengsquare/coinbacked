//! Program entrypoint

#![cfg(not(feature = "no-entrypoint"))]

use solana_program::
{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use solana_security_txt::security_txt;

security_txt! {
    // Required fields
    name: "Example",
    project_url: "http://example.com",
    contacts: "email:example@example.com,link:https://example.com/security,discord:example#1234",
    policy: "https://github.com/solana-labs/solana/blob/master/SECURITY.md"

}

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult 
{
    crate::processor::process_instruction(program_id, accounts, instruction_data)
}

// https://medium.com/coinmonks/understanding-arithmetic-overflow-underflows-in-rust-and-solana-smart-contracts-9f3c9802dc45
// more checks - https://solanacookbook.com/references/programs.html#how-to-read-accounts
// https://github.com/slowmist/solana-smart-contract-security-best-practices#Value-overflow


