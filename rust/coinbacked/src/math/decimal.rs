//! Math for preserving precision of token amounts which are limited
//! based on https://github.com/solana-labs/solana-program-library/tree/master/token-lending/program/src/math

#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::manual_range_contains)]

use crate::
{
    math::{common::*},
    error::CoinbackedError,
};
use solana_program::program_error::ProgramError;
use std::{convert::TryFrom, fmt};
use uint::construct_uint;

// U192 with 192 bits consisting of 3 x 64-bit words
construct_uint! 
{
    pub struct U192(3);
}

/// Large decimal values, precise to 18 digits
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Eq, Ord)]
pub struct Decimal(pub U192);

impl Decimal 
{
    // OPTIMIZE: use const slice when fixed in BPF toolchain
    fn wad() -> U192 
    {
        U192::from(WAD)
    }

    /// Floor scaled decimal to u64
    pub fn try_floor_u64(&self) -> Result<u64, ProgramError> 
    {
        let ceil_val = self
            .0
            .checked_div(Self::wad())
            .ok_or(CoinbackedError::MathError)?;
        Ok(u64::try_from(ceil_val).map_err(|_| CoinbackedError::MathError)?)
    }
}

impl fmt::Display for Decimal 
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut scaled_val = self.0.to_string();
        if scaled_val.len() <= SCALE {
            scaled_val.insert_str(0, &vec!["0"; SCALE - scaled_val.len()].join(""));
            scaled_val.insert_str(0, "0.");
        } else {
            scaled_val.insert(scaled_val.len() - SCALE, '.');
        }
        f.write_str(&scaled_val)
    }
}

impl From<u64> for Decimal 
{
    fn from(val: u64) -> Self {
        Self(Self::wad() * U192::from(val))
    }
}

impl From<u128> for Decimal 
{
    fn from(val: u128) -> Self {
        Self(Self::wad() * U192::from(val))
    }
}

impl TryAdd for Decimal 
{
    fn try_add(self, rhs: Self) -> Result<Self, ProgramError> {
        Ok(Self(
            self.0
                .checked_add(rhs.0)
                .ok_or(CoinbackedError::MathError)?,
        ))
    }
}

impl TrySub for Decimal 
{
    fn try_sub(self, rhs: Self) -> Result<Self, ProgramError> {
        Ok(Self(
            self.0
                .checked_sub(rhs.0)
                .ok_or(CoinbackedError::MathError)?,
        ))
    }
}

impl TryDiv<u64> for Decimal 
{
    fn try_div(self, rhs: u64) -> Result<Self, ProgramError> {
        Ok(Self(
            self.0
                .checked_div(U192::from(rhs))
                .ok_or(CoinbackedError::MathError)?,
        ))
    }
}

impl TryDiv<Decimal> for Decimal 
{
    fn try_div(self, rhs: Self) -> Result<Self, ProgramError> {
        Ok(Self(
            self.0
                .checked_mul(Self::wad())
                .ok_or(CoinbackedError::MathError)?
                .checked_div(rhs.0)
                .ok_or(CoinbackedError::MathError)?,
        ))
    }
}

impl TryMul<u64> for Decimal 
{
    fn try_mul(self, rhs: u64) -> Result<Self, ProgramError> {
        Ok(Self(
            self.0
                .checked_mul(U192::from(rhs))
                .ok_or(CoinbackedError::MathError)?,
        ))
    }
}

impl TryMul<Decimal> for Decimal 
{
    fn try_mul(self, rhs: Self) -> Result<Self, ProgramError> {
        Ok(Self(
            self.0
                .checked_mul(rhs.0)
                .ok_or(CoinbackedError::MathError)?
                .checked_div(Self::wad())
                .ok_or(CoinbackedError::MathError)?,
        ))
    }
}

