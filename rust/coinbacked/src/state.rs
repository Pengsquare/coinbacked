use solana_program::
{
    msg, pubkey::Pubkey, program_error::ProgramError,
};

use arrayref::
{
    array_ref, array_refs, mut_array_refs, array_mut_ref
};

use crate::
{
    utils::*,
};

/// Data for the backing account
#[derive(Debug)]
pub struct BackingAccount 
{
    /// pub key of the token this account is backing
    pub token_key: Pubkey,

    /// initial rent excemption, will just be used to figure out if/when account is empty in cases of rent excemption chanhing over time
    pub rent_excemption: u64,

    /// account seed bump for validation
    pub bump: u8,
}

impl BackingAccount
{
    pub fn pack(&self, dst: &mut [u8])
    {
        let dst = array_mut_ref![dst, 0, CO_ACCOUNT_BACKING_BYTE_LEN];
        let (token_key_dest, rent_excemption_dst, bump_dst) = mut_array_refs![dst, 32, CO_LAMPORTS_BYTE_LEN, 1];
        token_key_dest.copy_from_slice(self.token_key.as_ref());
        *rent_excemption_dst = self.rent_excemption.to_le_bytes();
        *bump_dst = self.bump.to_le_bytes();
    }

    pub fn unpack(source: &[u8]) -> Result<BackingAccount, ProgramError>
    {
        if source.len() < CO_ACCOUNT_BACKING_BYTE_LEN
        {
            msg!("No backing account data found. Aborting");
            return Err(ProgramError::InvalidAccountData);
        }

        let (token_key_data, rent_excemption_data, bump_data) = array_refs![array_ref![source, 0, CO_ACCOUNT_BACKING_BYTE_LEN], CO_PUBKEY_BYTE_LEN, CO_LAMPORTS_BYTE_LEN, CO_BUMP_BYTE_LEN];
        
        Ok(
            BackingAccount
            {
                token_key: Pubkey::new(token_key_data), 
                rent_excemption: u64::from_le_bytes(*rent_excemption_data), 
                bump: u8::from_le_bytes(*bump_data)
            }
        )
    }
}

#[derive(Debug)]
pub struct TreasuryAccount
{
    /// initial rent excemption, will just be used to figure out if/when account is empty in cases of rent excemption chanhing over time
    pub rent_excemption: u64,

    /// account seed bump for validation
    pub bump: u8,
}

impl TreasuryAccount
{
    pub fn pack(&self, dst: &mut [u8])
    {
        let dst = array_mut_ref![dst, 0, CO_ACCOUNT_PROTOCOL_TREASURY_BYTE_LEN];
        let (rent_excemption_dst, bump_dst) = mut_array_refs![dst, CO_LAMPORTS_BYTE_LEN, 1];
        *rent_excemption_dst = self.rent_excemption.to_le_bytes();
        *bump_dst = self.bump.to_le_bytes();
    }

    pub fn unpack(source: &[u8]) -> Result<TreasuryAccount, ProgramError>
    {
        if source.len() < CO_ACCOUNT_PROTOCOL_TREASURY_BYTE_LEN
        {
            msg!("No or invalid treasure account data found. Aborting");
            return Err(ProgramError::InvalidAccountData);
        }

        let (rent_excemption_data, bump_data) = array_refs![array_ref![source, 0, CO_ACCOUNT_PROTOCOL_TREASURY_BYTE_LEN], CO_LAMPORTS_BYTE_LEN, CO_BUMP_BYTE_LEN];

        Ok(
            TreasuryAccount
            {
                rent_excemption: u64::from_le_bytes(*rent_excemption_data),
                bump: u8::from_le_bytes(*bump_data)
            }
        )
    }
}