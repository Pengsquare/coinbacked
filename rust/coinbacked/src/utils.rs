//! Utilities

// general
pub const CO_OPERATION_BYTE_LEN: usize = 1;
pub const CO_SIGNATURE_BASE58_BYTE_LEN: usize = 128;
pub const CO_LAMPORTS_BYTE_LEN: usize = 8;
pub const CO_TOKEN_AMOUNT_BYTE_LEN: usize = 8;
pub const CO_BUMP_BYTE_LEN: usize = 1;
pub const CO_PUBKEY_BYTE_LEN: usize = 32;

pub const CO_SEED_COINBACKED: &[u8; 10] = b"COINBACKED"; // [67, 79, 73, 78, 66, 65, 67, 75, 69, 68]
pub const CO_ACCOUNT_BACKING_BYTE_LEN: usize = CO_PUBKEY_BYTE_LEN + CO_LAMPORTS_BYTE_LEN + CO_BUMP_BYTE_LEN;

pub const CO_PROTOCOL_FEE: u64 = 5000;
pub const CO_SEED_PROTOCOL_TREASURY: &[u8; 19] = b"COINBACKED-TREASURY";
pub const CO_ACCOUNT_PROTOCOL_TREASURY_BYTE_LEN: usize = CO_LAMPORTS_BYTE_LEN + CO_BUMP_BYTE_LEN;

// operation specific
pub const CO_OP_CREATE_BACKING_ACCOUNT:u8 = 0;
pub const CO_OP_CREATE_BACKING_ACCOUNT_BYTE_LEN:usize = CO_OPERATION_BYTE_LEN + CO_LAMPORTS_BYTE_LEN + CO_SIGNATURE_BASE58_BYTE_LEN;

pub const CO_OP_VALIDATE_BACKING_ACCOUNT:u8 = 1;
pub const CO_OP_VALIDATE_BACKING_ACCOUNT_BYTE_LEN: usize = CO_OPERATION_BYTE_LEN;

pub const CO_OP_ADD_TO_BALANCE_OF_BACKING_ACCOUNT: u8 = 2;
pub const CO_OP_ADD_TO_BALANCE_OF_BACKING_ACCOUNT_BYTE_LEN:usize = CO_OPERATION_BYTE_LEN + CO_LAMPORTS_BYTE_LEN + CO_SIGNATURE_BASE58_BYTE_LEN;

pub const CO_OP_BURN_AND_FREE_BALANCE: u8 = 3;
pub const CO_OP_BURN_AND_FREE_BALANCE_BYTE_LEN:usize = CO_OPERATION_BYTE_LEN + CO_TOKEN_AMOUNT_BYTE_LEN + CO_SIGNATURE_BASE58_BYTE_LEN;

pub const CO_OP_CLEAN_ACCOUNTS_AFTER_BURNING: u8 = 4;
pub const CO_OP_CLEAN_ACCOUNTS_AFTER_BURNING_BYTE_LEN:usize = CO_OPERATION_BYTE_LEN;

pub const CO_OP_ADMIN_CREATE_TREASURY_ACCOUNT: u8 = 10;
pub const CO_OP_ADMIN_CREATE_TREASURY_ACCOUNT_BYTE_LEN:usize = CO_OPERATION_BYTE_LEN;

pub const CO_OP_ADMIN_TRANSFER_FROM_TREASURY_ACCOUNT: u8 = 11;
pub const CO_OP_ADMIN_TRANSFER_FROM_TREASURY_ACCOUNT_BYTE_LEN:usize = CO_OPERATION_BYTE_LEN + CO_LAMPORTS_BYTE_LEN;

