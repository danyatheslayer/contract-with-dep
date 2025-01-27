#[cfg(test)]
mod tests {
    use solana_sdk::account::AccountSharedData;

    use {
        solana_program::{
            instruction::{AccountMeta, Instruction},
            pubkey::Pubkey,
            system_program,
        },
        solana_program_test::{processor, tokio, ProgramTest, ProgramTestContext},
        solana_sdk::{
            signature::{Keypair, Signer},
            transaction::Transaction,
        },
        std::str::FromStr,
    };

    use contract_solana::process_instruction;

    async fn setup_program_test() -> ProgramTestContext {
        let program_id = Pubkey::from_str("DWZr6WcGKbTgATQDVgBkfBWJzDafynkK9zNXpdbvCwZu").unwrap();
        let program_test = ProgramTest::new(
            "my_solana_program",
            program_id,
            processor!(process_instruction),
        );
        program_test.start_with_context().await
    }

    fn generate_pda(user_pubkey: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"user", user_pubkey.as_ref()], program_id)
    }

    #[tokio::test]
    async fn test_initialize_account() {
        let mut context = setup_program_test().await;
        let program_id = Pubkey::from_str("DWZr6WcGKbTgATQDVgBkfBWJzDafynkK9zNXpdbvCwZu").unwrap();
        let payer = Keypair::new();

        let (pda, _) = generate_pda(&payer.pubkey(), &program_id);

        context.set_account(
            &payer.pubkey(),
            &AccountSharedData::new(10_000_000, 0, &system_program::id()),
        );

        let instruction = Instruction::new_with_bytes(
            program_id,
            &[0],
            vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[&payer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .expect("Transaction failed");
    }

    #[tokio::test]
    async fn test_deposit() {
        let mut context = setup_program_test().await;
        let program_id = Pubkey::from_str("DWZr6WcGKbTgATQDVgBkfBWJzDafynkK9zNXpdbvCwZu").unwrap();
        let depositor = Keypair::new();

        let (pda, _) = generate_pda(&depositor.pubkey(), &program_id);

        context.set_account(
            &depositor.pubkey(),
            &AccountSharedData::new(1_000_000_000, 0, &system_program::id()),
        );

        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &[0],
            vec![
                AccountMeta::new(depositor.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let initialize_transaction = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&depositor.pubkey()),
            &[&depositor],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(initialize_transaction)
            .await
            .expect("Initialize transaction failed");

        let instruction_data = [1u8]
            .into_iter()
            .chain(10_000_000u64.to_le_bytes().to_vec().into_iter())
            .collect::<Vec<_>>();
        let deposit_instruction = Instruction::new_with_bytes(
            program_id,
            &instruction_data,
            vec![
                AccountMeta::new(depositor.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let deposit_transaction = Transaction::new_signed_with_payer(
            &[deposit_instruction],
            Some(&depositor.pubkey()),
            &[&depositor],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(deposit_transaction)
            .await
            .expect("Deposit transaction failed");

        let pda_account = context
            .banks_client
            .get_account(pda)
            .await
            .expect("PDA account not found")
            .unwrap();

        assert_eq!(
            pda_account.lamports, 10890880,
            "PDA balance should be 10_890_880 with rent"
        );
    }

    #[tokio::test]
    async fn test_withdraw() {
        let mut context = setup_program_test().await;
        let program_id = Pubkey::from_str("DWZr6WcGKbTgATQDVgBkfBWJzDafynkK9zNXpdbvCwZu").unwrap();
        let withdrawer = Keypair::new();

        let (pda, _) = generate_pda(&withdrawer.pubkey(), &program_id);

        context.set_account(
            &withdrawer.pubkey(),
            &AccountSharedData::new(1_000_000_000, 0, &system_program::id()),
        );

        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &[0],
            vec![
                AccountMeta::new(withdrawer.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let initialize_transaction = Transaction::new_signed_with_payer(
            &[initialize_instruction],
            Some(&withdrawer.pubkey()),
            &[&withdrawer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(initialize_transaction)
            .await
            .expect("Initialize transaction failed");

        let deposit_instruction_data = [1u8]
            .into_iter()
            .chain(10_000_000u64.to_le_bytes().to_vec().into_iter())
            .collect::<Vec<_>>();
        let deposit_instruction = Instruction::new_with_bytes(
            program_id,
            &deposit_instruction_data,
            vec![
                AccountMeta::new(withdrawer.pubkey(), true),
                AccountMeta::new(pda, false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
        );

        let deposit_transaction = Transaction::new_signed_with_payer(
            &[deposit_instruction],
            Some(&withdrawer.pubkey()),
            &[&withdrawer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(deposit_transaction)
            .await
            .expect("Deposit transaction failed");

        let withdraw_instruction_data = [2u8]
            .into_iter()
            .chain(5_000_000u64.to_le_bytes().to_vec().into_iter())
            .collect::<Vec<_>>();
        let withdraw_instruction = Instruction::new_with_bytes(
            program_id,
            &withdraw_instruction_data,
            vec![
                AccountMeta::new(withdrawer.pubkey(), false),
                AccountMeta::new(pda, false),
            ],
        );

        dbg!();

        let withdraw_transaction = Transaction::new_signed_with_payer(
            &[withdraw_instruction],
            Some(&withdrawer.pubkey()),
            &[&withdrawer],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(withdraw_transaction)
            .await
            .expect("Withdraw transaction failed");

        let pda_account = context
            .banks_client
            .get_account(pda)
            .await
            .expect("PDA account not found")
            .unwrap();

        let withdrawer_account = context
            .banks_client
            .get_account(withdrawer.pubkey())
            .await
            .expect("Withdrawer not founf")
            .unwrap();

        assert_eq!(
            pda_account.lamports, 5_890_880,
            "PDA balance should be 5_890_880"
        );
        assert_eq!(
            withdrawer_account.lamports, 994_094_120,
            "Withdrawer balance should be 994_094_120"
        );
    }
}
