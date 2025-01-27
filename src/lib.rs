use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

pub enum SolanaInstruction {
    InitializeAccount,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

impl SolanaInstruction {
    pub fn match_instruction(input: &[u8]) -> Result<Self, ProgramError> {
        let (&tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        Ok(match tag {
            0 => Self::InitializeAccount,
            1 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;
                Self::Deposit { amount }
            }
            2 => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| slice.try_into().ok())
                    .map(u64::from_le_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;
                Self::Withdraw { amount }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        })
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolanaInstruction::match_instruction(instruction_data)?;

    match instruction {
        SolanaInstruction::InitializeAccount => {
            msg!("Instruction: InitializeAccount");
            let accounts_iter = &mut accounts.iter();
            let initializer = next_account_info(accounts_iter)?;
            let pda_account = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;

            let rent_exemption = Rent::get()?.minimum_balance(0);

            let (pda, bump_seed) =
                Pubkey::find_program_address(&[b"user", initializer.key.as_ref()], program_id);

            if pda != *pda_account.key {
                return Err(ProgramError::InvalidArgument);
            }

            invoke_signed(
                &system_instruction::create_account(
                    initializer.key,
                    pda_account.key,
                    rent_exemption,
                    0,
                    program_id,
                ),
                &[
                    initializer.clone(),
                    pda_account.clone(),
                    system_program.clone(),
                ],
                &[&[b"user", initializer.key.as_ref(), &[bump_seed]]],
            )?;
        }
        SolanaInstruction::Deposit { amount } => {
            if amount == 0 {
                return Err(ProgramError::InvalidInstructionData);
            }
            msg!("Instruction: Deposit {}", amount);
            let accounts_iter = &mut accounts.iter();
            let depositor = next_account_info(accounts_iter)?;
            let pda_account = next_account_info(accounts_iter)?;
            let system_program = next_account_info(accounts_iter)?;

            let (pda, _) =
                Pubkey::find_program_address(&[b"user", depositor.key.as_ref()], program_id);

            if pda != *pda_account.key {
                return Err(ProgramError::InvalidArgument);
            }

            invoke(
                &system_instruction::transfer(depositor.key, pda_account.key, amount),
                &[
                    depositor.clone(),
                    pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }
        SolanaInstruction::Withdraw { amount } => {
            msg!("Instruction: Withdraw {}", amount);
            let accounts_iter = &mut accounts.iter();
            let withdrawer = next_account_info(accounts_iter)?;
            let pda_account = next_account_info(accounts_iter)?;

            let balance = **pda_account.lamports.borrow();
            if amount > balance {
                return Err(ProgramError::InsufficientFunds);
            }

            if pda_account.owner != program_id {
                return Err(ProgramError::IncorrectProgramId);
            }

            let (pda, _) =
                Pubkey::find_program_address(&[b"user", withdrawer.key.as_ref()], program_id);

            if pda != *pda_account.key {
                return Err(ProgramError::InvalidArgument);
            }

            **pda_account.try_borrow_mut_lamports()? -= amount;
            **withdrawer.try_borrow_mut_lamports()? += amount;
        }
    }

    Ok(())
}
