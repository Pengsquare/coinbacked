//! Program instruction processor
#![allow(missing_docs)]

use solana_program::
{
    native_token::Sol,
    program_pack::Pack,
    entrypoint::ProgramResult,
    msg,
    system_program,
    rent::Rent,
    pubkey::Pubkey,
    program_error::ProgramError,
    account_info::
    {
        next_account_info, AccountInfo
    },
    system_instruction::
    {
        create_account, 
        transfer
    },
    program::
    {
        invoke, 
        invoke_signed
    },
    sysvar::{Sysvar, rent},
};

use arrayref::array_ref;

use spl_token::
{
    state::
    {
        Account, 
        Mint
    },
    instruction::
    {
        burn, 
        close_account
    },
};

use crate::
{
    utils::*,
    state::{BackingAccount, TreasuryAccount},
    instruction::Instruction,
    math::{Decimal, TryMul, TrySub, TryDiv},
};

/// Instruction processor
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult 
{
    match Instruction::unpack(instruction_data)?
    {
        Instruction::CreateBackingAccount{lamports, signed_tos} =>
        {
            msg!("Instruction: Create Backing Account");  
            process_create_backing_account(
                program_id, 
                accounts, 
                lamports, 
                signed_tos)?;         
        },

        Instruction::ValidateBackingAccount =>
        {
            msg!("Instruction: Validate Backing Account");  
            process_validate_backing_account(
                program_id, 
                accounts)?;
        },

        Instruction::AddToBalanceOfBackingAccount {lamports, signed_tos} =>
        {
            msg!("Instruction: Add to Balance of Backing Account");
            process_add_to_balance_of_backing_account(
                program_id, 
                accounts, 
                lamports, 
                signed_tos)?;  
        },

        Instruction::BurnTokenAndFreeBalanace {amount, signed_tos} => 
        {
            msg!("Instruction: Burn Tokwn and Free Balanace");
            process_burn_token_and_free_balanace(
                program_id, 
                accounts, 
                amount, 
                signed_tos)?;
        },

        Instruction::CleanAccountsAfterBurning =>
        {
            msg!("Instruction: Clean Accounts After Burning");
            process_clean_accounts_after_burning(
                accounts)?;
        },

        Instruction::AdminCreateTreasuryAccount =>
        {
            msg!("Instruction: Admin Create Treasury Account");
            process_admin_create_treasury_account(
                program_id,
                accounts)?;
        },

        Instruction::AdminTransferFromTreasuryAccount {lamports} =>
        {
            msg!("Instruction: Admin Transfer From Treasury Account");
            process_admin_transfer_from_treasury_account(
                program_id, 
                accounts, 
                lamports)?;               
        },
    }

    Ok(())
}

fn process_create_backing_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    lamports: u64,
    signed_tos: String,
) -> ProgramResult
{
    let account_info_iter = &mut accounts.iter();
    
    let source_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let backing_pda = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;
   
    let system_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    // checking if payer account is the signer
    if !source_account.is_signer 
    {
        msg!("Account is not signer! Aborting.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    // checking if accounts are writable
    if !backing_pda.is_writable || !protocol_treasury_account.is_writable
    {
        msg!("Required accounts not writable! Aborting.");
        return Err(ProgramError::InvalidAccountData);
    }

    // system & rent correct?
    if system_program.key.ne(&system_program::ID) || rent_sysvar.key.ne(&rent::ID)
    {
        msg!("Invalid system programs! Aborting.");
        return Err(ProgramError::IncorrectProgramId);
    }

    // unpack SPL-related accounts
    let token_account_spl = Account::unpack(&token_account.try_borrow_data()?)?;

    // make sure that token account belongs to mint
    if token_account_spl.mint.ne(mint_account.key)
    {
        msg!("Token account does not belong to mint! Aborting.");
        return Err(ProgramError::InvalidInstructionData); 
    }

    // make sure that source account is owner of token account
    if token_account_spl.owner.ne(source_account.key)
    {
        msg!("Token account does not belong to signer! Aborting.");
        return Err(ProgramError::InvalidInstructionData); 
    }

    // make sure that token account has token amount > 0, only holders can back tokens
    if token_account_spl.amount == 0
    {
        msg!("Token account does not store any token amount, but required for backing! Aborting.");
        return Err(ProgramError::InvalidInstructionData); 
    }

    // check backing pda
    let bump = check_backing_account(backing_pda, mint_account, program_id, true)?;

    // log ToS signature
    msg!("All pre-checks passed. User signed terms of service: {}", signed_tos);

    // create backing account and back with sol
    let rent = Rent::get()?;
    let min_excemption_balance = rent.minimum_balance(CO_ACCOUNT_BACKING_BYTE_LEN).max(1);

    invoke_signed(
        &create_account(
            &source_account.key,
            &backing_pda.key,
            min_excemption_balance + lamports,
            CO_ACCOUNT_BACKING_BYTE_LEN as u64,
            &program_id
        ), 
        &[
            source_account.clone(), 
            backing_pda.clone(), 
            system_program.clone(), 
            rent_sysvar.clone()
        ], 
        &[&[
            mint_account.key.as_ref(),
            program_id.as_ref(),
            CO_SEED_COINBACKED,
            &[bump],
        ]]
    )?;

    let actual_account_data = BackingAccount
    {
            token_key: *mint_account.key,
            rent_excemption: min_excemption_balance, 
            bump: bump
    };

    let data = &mut backing_pda.try_borrow_mut_data()?[..];
    actual_account_data.pack(data);

    // pay protocol
    pay_protocol(source_account, protocol_treasury_account, program_id, accounts, true)?;

    Ok(())
}

fn process_validate_backing_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo]
) -> ProgramResult
{
    let account_info_iter = &mut accounts.iter();

    let payer_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let backing_pda = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;

    let mut validation_failure = false;
    
    // pda check
    let seeds = &[
        mint_account.key.as_ref(),
        program_id.as_ref(),
        CO_SEED_COINBACKED
    ];

    let (backing_pda_key, bump) = Pubkey::find_program_address(seeds, &program_id);

    if backing_pda_key.ne(backing_pda.key)
    {
        msg!("Validation failure: Backing account address is not valid PDA for mint.");
        validation_failure = true;
    }
    else
    {
        msg!("Validation success: Backing account address is valid PDA for mint.");
    }

    let backing_account = BackingAccount::unpack(&backing_pda.data.borrow()[..])?;
    let mint_account_spl = Mint::unpack(&mut mint_account.data.borrow_mut())?;

    // backing account belongs to mint
    if backing_account.token_key.ne(mint_account.key)
    {
        msg!("Validation failure: Backing account not pointing to mint account.");
        validation_failure = true;
    }
    else
    {
        msg!("Validation success: Backing account pointing to mint account.");
    }

    // backing account holds at least rent excemption
    if backing_account.rent_excemption > **backing_pda.lamports.borrow()
    {
        msg!("Validation failure: Backing account does not hold rent excemption.");
        validation_failure = true;
    }
    else
    {
        msg!("Validation success: Backing account holds at least rent excempt.");
    }

    // seed bump check
    if backing_account.bump != bump
    {
        msg!("Validation failure: Backing account bump is incorrect.");
        validation_failure = true;
    }
    else
    {
        msg!("Validation success: Backing account bump is correct.");
    }

    // additional information: current payout per unit
    let per_unit_payout = get_payout_in_lamport(
        token_amount_one_unit(mint_account_spl.decimals), 
        mint_account_spl.supply, 
        backing_pda.lamports(), 
        backing_account.rent_excemption
    )?;
    
    msg!("Information only: Current per token unit payout is: {} lamport / {}", per_unit_payout, Sol(per_unit_payout));

    // check if mint is of fixed supply, if not warn that payout floor is not fixed...
    if mint_account_spl.mint_authority.is_none()
    {
        msg!("Information only: supply of token is fixed, payout floor will remain calculated payout.");
    }
    else
    {
        msg!("Information only: (WARNING) supply of token is NOT fixed, payout floor might decrease.");
    }

    // overall result
    if validation_failure
    {
        msg!("Overall result: validation failed.");
    }
    else
    {
        msg!("Overall result: validation sucessfull.");
    }

    // pay protocol
    pay_protocol(payer_account, protocol_treasury_account, program_id, accounts, true)?;

    Ok(())
}

fn process_add_to_balance_of_backing_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    lamports: u64,
    signed_tos: String
) ->ProgramResult
{
    let account_info_iter = &mut accounts.iter();

    let source_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let backing_pda = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;

     // checking if payer account is the signer
     if !source_account.is_signer 
     {
         msg!("Account is not signer! Aborting.");
         return Err(ProgramError::MissingRequiredSignature);
     }

    // checking if accounts are writable
    if !backing_pda.is_writable || !protocol_treasury_account.is_writable
    {
        msg!("Required accounts not writable! Aborting.");
        return Err(ProgramError::InvalidAccountData);
    }

    // check backing pda
    check_backing_account(backing_pda, mint_account, program_id, false)?;

    // check mint, only increase balance if there is still tokens to guarantee payout
    let mint_account_spl = Mint::unpack(&mut mint_account.data.borrow_mut())?;
    if mint_account_spl.supply == 0
    {
        msg!("Mint supply is 0, cannot add balance! Aborting.");
        return Err(ProgramError::InvalidAccountData);
    }

    // log ToS signature
    msg!("All pre-checks passed. User signed terms of service: {}", signed_tos);

    // transfer lamports to backing account
    invoke(
        &transfer(source_account.key, backing_pda.key, lamports),
        &accounts
    )?;

    // pay protocol
    pay_protocol(source_account, protocol_treasury_account, program_id, accounts, true)?;

    Ok(())
}

fn process_burn_token_and_free_balanace
(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    signed_tos: String
) -> ProgramResult
{
    let account_info_iter = &mut accounts.iter();
            
    let owner_account = next_account_info(account_info_iter)?;
    let mint_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let backing_pda = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;

    let token_program =  next_account_info(account_info_iter)?;
    
     // checking if payer account is the signer
     if !owner_account.is_signer 
     {
         msg!("Account is not signer! Aborting.");
         return Err(ProgramError::MissingRequiredSignature);
     }

    // token program correct?
    if token_program.key.ne(&spl_token::ID)
    {
        msg!("Invalid token program! Aborting.");
        return Err(ProgramError::IncorrectProgramId);
    }

    // checking if accounts are writable
    if !backing_pda.is_writable 
        || !protocol_treasury_account.is_writable 
        || !token_account.is_writable 
        || !owner_account.is_writable
    {
        msg!("Required accounts not writable! Aborting.");
        return Err(ProgramError::InvalidAccountData);
    }

    // unpack SPL-related accounts
    let token_account_spl = Account::unpack(&mut token_account.data.borrow_mut())?;
    let mint_account_spl = Mint::unpack(&mut mint_account.data.borrow_mut())?;
    
    // make sure that token account belongs to mint
    if token_account_spl.mint.ne(mint_account.key)
    {
        msg!("Token account does not belong to mint! Aborting.");
        return Err(ProgramError::InvalidAccountData); 
    }

    // make sure that source account is owner of token account
    if token_account_spl.owner.ne(owner_account.key)
    {
        msg!("Token account does not belong to signer! Aborting.");
        return Err(ProgramError::InvalidInstructionData); 
    }

    // check backing pda
    check_backing_account(backing_pda, mint_account, program_id, false)?;

    // log ToS signature
    msg!("All pre-checks passed. User signed terms of service: {}", signed_tos);

    // calculate lamports to be transfered from backing
    let backing_account = BackingAccount::unpack(&backing_pda.try_borrow_data().unwrap()[..])?;

    let total_payout = get_payout_in_lamport(
        amount,
        mint_account_spl.supply, 
        backing_pda.lamports(), 
        backing_account.rent_excemption
     )?;

     msg!("Calculated payout for burning {} tokens is: {}", amount, total_payout);

    invoke(
        &burn(
            token_program.key, 
            token_account.key,
            mint_account.key,
            owner_account.key,
            &[owner_account.key],
            amount
        )?,accounts
    )?;

    // transfer lamports from backing to target
    **backing_pda.try_borrow_mut_lamports()? = 
        backing_pda.lamports().checked_sub(total_payout)
        .ok_or(ProgramError::InvalidAccountData)?;

    **owner_account.try_borrow_mut_lamports()? = 
        owner_account.lamports().checked_add(total_payout)
        .ok_or(ProgramError::InvalidAccountData)?;

    // pay protocol
    pay_protocol(owner_account, protocol_treasury_account, program_id, accounts, false)?;

    Ok(())
}

fn process_clean_accounts_after_burning(
    accounts: &[AccountInfo],
) -> ProgramResult
{
    let account_info_iter = &mut accounts.iter();
            
    let owner_account = next_account_info(account_info_iter)?;
    let token_account = next_account_info(account_info_iter)?;
    let backing_pda = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;

    let token_program =  next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // checking if payer account is the signer
    if !owner_account.is_signer 
    {
        msg!("Account is not signer! Aborting.");
        return Err(ProgramError::MissingRequiredSignature);
    }

    let token_account_spl = Account::unpack(&mut token_account.data.borrow_mut())?;

    let backing_account_data = &mut backing_pda.data.borrow_mut()[..];
    let backing_account = BackingAccount::unpack(backing_account_data)?;

    // token account empty? then close...
    if token_account_spl.amount == 0
    {
        msg!("Token account now empty, will close it.");
        invoke
        (
            &close_account(
                token_program.key,
                token_account.key,
                owner_account.key,
                owner_account.key,
                &[owner_account.key],
            )?,
            accounts
        )?;
    }

    // backing account empty? then close, protocol will receive funding...
    if **backing_pda.lamports.borrow_mut() <= backing_account.rent_excemption
    {
        msg!("Backing account now empty, will close it.");

        **protocol_treasury_account.lamports.borrow_mut() = 
            protocol_treasury_account.lamports()
                .checked_add(backing_pda.lamports())
                .ok_or(ProgramError::InvalidInstructionData)?;

        // clean account
        **backing_pda.try_borrow_mut_lamports()? = 0;
        backing_account_data.copy_from_slice(&[0; CO_ACCOUNT_BACKING_BYTE_LEN]);
        backing_pda.assign(system_program.key);
        backing_pda.realloc(0, false)?;
    }

    Ok(())
}

fn process_admin_create_treasury_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult
{
    let account_info_iter = &mut accounts.iter();
            
    let owner_account = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;
    let program_account = next_account_info(account_info_iter)?;
    let program_executable_data_account =  next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;
    let rent_sysvar = next_account_info(account_info_iter)?;

    // check program account pointing to current program
    if program_account.key.ne(program_id)
    {
        msg!("Program account has incorrect ID. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // check program executable data account belongs to program account
    if program_executable_data_account.key.ne(&get_program_executable_data_account_key(program_account)?)
    {
        msg!("Program account and program executable data account don't fit. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // check that signer is update authority
    if owner_account.key.ne(&get_update_authority(program_executable_data_account)?)
    {
        msg!("Signer is not update authority for protocol. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // check protocol treasury account valid PDA
    let bump = check_protocol_treasury_account(protocol_treasury_account, program_id)?;

    // create it
    if protocol_treasury_account.owner.ne(program_id)
    {
        let rent = Rent::get()?;
        let min_excemption_balance = rent.minimum_balance(CO_ACCOUNT_PROTOCOL_TREASURY_BYTE_LEN).max(1);

        invoke_signed(
            &create_account(
                &owner_account.key,
                &protocol_treasury_account.key,
                min_excemption_balance,
                CO_ACCOUNT_PROTOCOL_TREASURY_BYTE_LEN as u64,
                &program_id
            ), 
            &[
                owner_account.clone(), 
                protocol_treasury_account.clone(), 
                system_program.clone(), 
                rent_sysvar.clone()
            ], 
            &[&[
                program_id.as_ref(),
                CO_SEED_PROTOCOL_TREASURY,
                &[bump],
            ]]
        )?;

        let treasure = TreasuryAccount 
        {
            rent_excemption: min_excemption_balance,
            bump: bump
        };

        treasure.pack(&mut protocol_treasury_account.try_borrow_mut_data()?);
    }
    else
    {
        msg!("Info: Protocol treasury account seems to exist already. No action needed.");
    }

    Ok(())
}

fn process_admin_transfer_from_treasury_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    lamports: u64
) -> ProgramResult
{      
    let account_info_iter = &mut accounts.iter();
            
    let owner_account = next_account_info(account_info_iter)?;
    let receiving_account = next_account_info(account_info_iter)?;
    let protocol_treasury_account = next_account_info(account_info_iter)?;
    let program_account = next_account_info(account_info_iter)?;
    let program_executable_data_account =  next_account_info(account_info_iter)?;

    // check program account pointing to current program
    if program_account.key.ne(program_id)
    {
        msg!("Program account has incorrect ID. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // check program executable data account belongs to program account
    if program_executable_data_account.key.ne(&get_program_executable_data_account_key(program_account)?)
    {
        msg!("Program account and program executable data account don't fit. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // check that signer is update authority
    if owner_account.key.ne(&get_update_authority(program_executable_data_account)?)
    {
        msg!("Signer is not update authority for protocol. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // check protocol treasury account valid PDA
    check_protocol_treasury_account(protocol_treasury_account, program_id)?;            

    // get stored rent excemption fee
    let treasure = TreasuryAccount::unpack(&mut protocol_treasury_account.data.borrow_mut())?;

    // make sure transfer does not deplete below rent excemption
    if **protocol_treasury_account.lamports.borrow_mut() - lamports < treasure.rent_excemption
    {
        msg!("Maximum transfer amount: {}", **protocol_treasury_account.lamports.borrow_mut()-treasure.rent_excemption);
        msg!("Treasury does not have enough funds, transfer would cut into rent excemption. Aborting.");
        return Err(ProgramError::InsufficientFunds);  
    }
    else
    {
        // transfer lamports from treasury to target
        **protocol_treasury_account.try_borrow_mut_lamports()? = 
            protocol_treasury_account.lamports().checked_sub(lamports)
            .ok_or(ProgramError::InsufficientFunds)?;

        **receiving_account.try_borrow_mut_lamports()? = 
            receiving_account.lamports().checked_add(lamports)
            .ok_or(ProgramError::InsufficientFunds)?;
    }   

    Ok(())
}

// helper functions

fn get_program_executable_data_account_key(
    program_account: &AccountInfo
)-> Result<Pubkey, ProgramError>
{
    let data = program_account.try_borrow_data()?;
    Ok(Pubkey::new(array_ref![data, 4, 32]))
}

fn get_update_authority(
    program_executable_data_account: &AccountInfo
)-> Result<Pubkey, ProgramError>
{
    let data = program_executable_data_account.try_borrow_data()?;
    Ok(Pubkey::new( array_ref![data, 13, 32]))    
}

fn check_backing_account(
    backing_pda: &AccountInfo,
    mint_account: &AccountInfo,
    program_id: &Pubkey,
    should_be_empty: bool,
) -> Result<u8, ProgramError>
{
     // check backing PDA
     let seeds = &[
        mint_account.key.as_ref(),
        program_id.as_ref(),
        CO_SEED_COINBACKED
    ];

    let (backing_pda_key, bump) = Pubkey::find_program_address(seeds, &program_id);

    // check that backing PDA is derived from mint account, etc.
    if backing_pda_key.ne(backing_pda.key)
    {
        msg!("Account key missmatch - PDA for backing account is not matching. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    if should_be_empty
    {
        // check backing PDA does not exist/is empty
        if backing_pda.owner.eq(program_id) || !backing_pda.data_is_empty()
        {
            msg!("Backing account seems to exist already. Aborting.");
            return Err(ProgramError::InvalidAccountData);  
        }
    }
    else
    {
        // check content correct 
        let backing_account = BackingAccount::unpack(&backing_pda.try_borrow_data().unwrap()[..])?;
    
        // backing account belongs to mint
        if backing_account.token_key.ne(mint_account.key)
        {
            msg!("Backing account not pointing to mint account. Aborting.");
            return Err(ProgramError::InvalidAccountData); 
        }
    
        // seed bump check
        if backing_account.bump != bump
        {
            msg!("Backing account bump is incorrect. Aborting.");
            return Err(ProgramError::InvalidAccountData); 
        }
    }

    Ok(bump)
}

fn check_protocol_treasury_account(
    protocol_treasury_account: &AccountInfo,
    program_id: &Pubkey
) -> Result<u8, ProgramError>
{
    let seeds = &[
        program_id.as_ref(),
        CO_SEED_PROTOCOL_TREASURY
    ];

    let (treasury_pda_key, bump) = Pubkey::find_program_address(seeds, &program_id);

    // check pda
    if treasury_pda_key.ne(protocol_treasury_account.key)
    {
        msg!("Account key missmatch - PDA for treasury account is not matching. Aborting.");
        return Err(ProgramError::InvalidAccountData);  
    }

    // if exist, check owner and bump...
    if !protocol_treasury_account.data_is_empty() && protocol_treasury_account.owner.eq(program_id)
    {
        let treasure = TreasuryAccount::unpack(&protocol_treasury_account.try_borrow_data()?)?;
        if treasure.bump != bump
        {
            msg!("Account key missmatch - PDA bump for treasury account is not matching. Aborting.");
            return Err(ProgramError::InvalidAccountData);  
        }
    }

    Ok(bump)
}

fn pay_protocol(
    source_account: &AccountInfo,
    protocol_treasury_account: &AccountInfo,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    invoke_transfer: bool
) -> ProgramResult 
{
    check_protocol_treasury_account(protocol_treasury_account, program_id)?;
    
    // check if PDA does exist
    if protocol_treasury_account.owner.ne(program_id)
    {
        msg!("Treasury account does not seem to exist. No treasury payment executed. Aborting.");
        return Err(ProgramError::InvalidAccountData);
    }

    // actual transfer to treasury - two versions due to different borrowing behaviour
    if invoke_transfer
    {
        invoke(
            &transfer(source_account.key, protocol_treasury_account.key, CO_PROTOCOL_FEE),
            &accounts
        )?;
    }
    else
    {   
        //**source_account.lamports.borrow_mut() -= CO_PROTOCOL_FEE;
        **source_account.try_borrow_mut_lamports()? = 
            source_account.lamports().checked_sub(CO_PROTOCOL_FEE)
            .ok_or(ProgramError::InvalidAccountData)?;
        
        //**protocol_treasury_account.lamports.borrow_mut() += CO_PROTOCOL_FEE;
        **protocol_treasury_account.try_borrow_mut_lamports()? = 
            protocol_treasury_account.lamports().checked_add(CO_PROTOCOL_FEE)
            .ok_or(ProgramError::InvalidAccountData)?;
    }

    Ok(())
}

fn token_amount_one_unit(
    decimals: u8
) -> u64
{
    10_usize.pow(decimals as u32) as u64
}

fn get_payout_in_lamport(
    token_amount: u64,
    supply: u64,
    backing_lamports: u64,
    backing_rent_excemption: u64
) -> Result<u64, ProgramError>
{
    // token_amount * (backing_lamports - backing_rent_excemption) / supply
    Decimal::from(token_amount).try_mul(
        Decimal::from(backing_lamports).try_sub(
                Decimal::from(backing_rent_excemption)
            )?
        )?
        .try_div(Decimal::from(supply))?.try_floor_u64()
}
