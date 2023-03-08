//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the TokenLending program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum CoinbackedError 
{

    /// Math operation error
    #[error("Math operation erorr")]
    MathError,

}

impl From<CoinbackedError> for ProgramError 
{
    fn from(e: CoinbackedError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for CoinbackedError 
{
    fn type_of() -> &'static str {
        "Coindbacked Error"
    }
}