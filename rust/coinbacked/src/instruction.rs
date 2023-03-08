use solana_program::
{
    msg, program_error::ProgramError,
};

use arrayref::
{
    array_ref, array_refs
};

use std::str::from_utf8;

use crate::
{
    utils::*,
};

/// Program instructions
pub enum Instruction 
{
    /// creation of backing account for token
    CreateBackingAccount 
    { 
        /// initial coin amount to back token
        lamports: u64, 
        /// tos
        signed_tos: String
    },

    /// validate existing account
    ValidateBackingAccount,

    /// increase balance of backing account
    AddToBalanceOfBackingAccount 
    {  
        /// coin amount to add to backing account of token
        lamports: u64, 
        /// tos
        signed_tos: String
    },

    /// burn a token and receive (portion) of balance
    BurnTokenAndFreeBalanace 
    { 
        /// amount of token to burn
        amount: u64,
        /// tos
        signed_tos: String
    },

    /// clean up after burning and freeing
    CleanAccountsAfterBurning,

    /// admin transaction to create treasury account
    AdminCreateTreasuryAccount,

    /// admin transaction to transfer from treasury account
    AdminTransferFromTreasuryAccount
    {
        /// amount to transfer
        lamports: u64
    },
}

impl Instruction
{
    pub fn unpack(instruction_data: &[u8]) -> Result<Instruction, ProgramError>
    {
        if instruction_data.len() < 1
        {
            msg!("Incorrect data format, too short. Aborting.");
            return Err(ProgramError::InvalidInstructionData);
        }

        match instruction_data[0]
        {
            CO_OP_CREATE_BACKING_ACCOUNT => 
            {
                if instruction_data.len() != CO_OP_CREATE_BACKING_ACCOUNT_BYTE_LEN 
                {
                    msg!("Incorrect data format, wrong size for operation CREATE BACKING ACCOUNT. Aborting.");
                    return Err(ProgramError::InvalidInstructionData);
                }
                
                // slice into segments of data
                let (lamports_data, signed_tos_data) = array_refs![array_ref![instruction_data, 1, CO_OP_CREATE_BACKING_ACCOUNT_BYTE_LEN-1], CO_LAMPORTS_BYTE_LEN, CO_SIGNATURE_BASE58_BYTE_LEN];

                // create parameters
                let lamports = u64::from_le_bytes(*lamports_data);
                let signed_tos = from_utf8(signed_tos_data).map_err(|err| {
                    msg!("Invalid UTF-8, from byte {}. Aborting.", err.valid_up_to());
                    ProgramError::InvalidInstructionData
                })?;

                Ok(Instruction::CreateBackingAccount {lamports: lamports, signed_tos: signed_tos.to_string()})
            },

            CO_OP_VALIDATE_BACKING_ACCOUNT =>
            {
                if instruction_data.len() != CO_OP_VALIDATE_BACKING_ACCOUNT_BYTE_LEN
                {
                    msg!("Incorrect data format, wrong size for operation VALIDATE BACKING ACCOUNT. Aborting");
                    return Err(ProgramError::InvalidInstructionData);
                }

                Ok(Instruction::ValidateBackingAccount)
            },

            CO_OP_ADD_TO_BALANCE_OF_BACKING_ACCOUNT =>
            {
                if instruction_data.len() != CO_OP_ADD_TO_BALANCE_OF_BACKING_ACCOUNT_BYTE_LEN
                {
                    msg!("Incorrect data format, wrong size for operation ADD TO BALANCE OF BACKING ACCOUNT. Aborting.");
                    return Err(ProgramError::InvalidInstructionData);
                }
                
                // slice into segments of data
                let (lamports_data, signed_tos_data) = array_refs![array_ref![instruction_data, 1, CO_OP_ADD_TO_BALANCE_OF_BACKING_ACCOUNT_BYTE_LEN-1], CO_LAMPORTS_BYTE_LEN, CO_SIGNATURE_BASE58_BYTE_LEN];
                
                // create parameters
                let lamports = u64::from_le_bytes(*lamports_data);
                let signed_tos = from_utf8(signed_tos_data).map_err(|err| {
                    msg!("Invalid UTF-8, from byte {}. Aborting.", err.valid_up_to());
                    ProgramError::InvalidInstructionData
                })?;

                Ok(Instruction::AddToBalanceOfBackingAccount {lamports: lamports, signed_tos: signed_tos.to_string()})
            },

            CO_OP_BURN_AND_FREE_BALANCE =>
            {
                if instruction_data.len() != CO_OP_BURN_AND_FREE_BALANCE_BYTE_LEN
                {
                    msg!("Incorrect data format, wrong size for operation BURN AND FREE BALANCE. Aborting.");
                    return Err(ProgramError::InvalidInstructionData);
                }
                
                // slice into segments of data
                let (amount_data, signed_tos_data) = array_refs![array_ref![instruction_data, 1, CO_OP_ADD_TO_BALANCE_OF_BACKING_ACCOUNT_BYTE_LEN-1], CO_LAMPORTS_BYTE_LEN, CO_SIGNATURE_BASE58_BYTE_LEN];
                
                // create parameters
                let amount = u64::from_le_bytes(*amount_data);
                let signed_tos = from_utf8(signed_tos_data).map_err(|err| {
                    msg!("Invalid UTF-8, from byte {}. Aborting.", err.valid_up_to());
                    ProgramError::InvalidInstructionData
                })?;

                Ok(Instruction::BurnTokenAndFreeBalanace {amount: amount, signed_tos: signed_tos.to_string()})
            },

            CO_OP_CLEAN_ACCOUNTS_AFTER_BURNING =>
            {
                if instruction_data.len() != CO_OP_CLEAN_ACCOUNTS_AFTER_BURNING_BYTE_LEN
                {
                    msg!("Incorrect data format, wrong size for operation CLEAN ACCOUNTS AFTER BURNING. Aborting");
                    return Err(ProgramError::InvalidInstructionData);
                }

                Ok(Instruction::CleanAccountsAfterBurning)
            },

            CO_OP_ADMIN_CREATE_TREASURY_ACCOUNT =>
            {
                if instruction_data.len() != CO_OP_ADMIN_CREATE_TREASURY_ACCOUNT_BYTE_LEN
                {
                    msg!("Incorrect data format, wrong size for operation ADMIN CREATE TREASURY ACCOUNT. Aborting.");
                    return Err(ProgramError::InvalidInstructionData);
                }

                Ok(Instruction::AdminCreateTreasuryAccount)
            },

            CO_OP_ADMIN_TRANSFER_FROM_TREASURY_ACCOUNT =>
            {
                if instruction_data.len() != CO_OP_ADMIN_TRANSFER_FROM_TREASURY_ACCOUNT_BYTE_LEN
                {
                    msg!("Incorrect data format, wrong size for operation ADMIN TRANSFER FROM TREASURY ACCOUNT. Aborting.");
                    return Err(ProgramError::InvalidInstructionData);
                }
                let lamports_data = array_ref![instruction_data, 1, CO_OP_ADMIN_TRANSFER_FROM_TREASURY_ACCOUNT_BYTE_LEN-1];
                let lamports = u64::from_le_bytes(*lamports_data);

                Ok(Instruction::AdminTransferFromTreasuryAccount { lamports: lamports})
            }

            _ => Err(ProgramError::InvalidInstructionData)
        }

    }
}