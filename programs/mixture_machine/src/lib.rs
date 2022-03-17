pub mod utils;

use {
    crate::utils::{
        assert_is_ata, assert_owned_by, //assert_initialized, assert_keys_equal,
        spl_token_transfer, TokenTransferParams, spl_token_burn, TokenBurnParams,
    },
    anchor_lang::{
        prelude::*,
        solana_program::{
            log::sol_log_compute_units,
            program::invoke_signed, //invoke
            serialize_utils::{read_pubkey, read_u16},
            sysvar, //system_instruction, 
            pubkey::Pubkey,
        },
        AnchorDeserialize, AnchorSerialize, Discriminator, Key,
    },
    anchor_spl::token::Token, //TokenAccount
    //arrayref::array_ref,
    mpl_token_metadata::{
        instruction::create_metadata_accounts, // create_master_edition, update_metadata_accounts,
        state::{
            MAX_CREATOR_LIMIT, MAX_SYMBOL_LENGTH,
        }, //MAX_CREATOR_LEN, MAX_NAME_LENGTH, MAX_URI_LENGTH,
    },
    // spl_token::{
    //     state::Mint,
    //     instruction::transfer
    // },
    std::str::FromStr, //cell::RefMut, ops::Deref, 
    //spl_associated_token_account::create_associated_token_account,
};

declare_id!("7TzTuLobYxcPJw62EMEq93C72vBd8tmRP8CbQ7e4tS3z");

// const EXPIRE_OFFSET: i64 = 10 * 60;
const PREFIX: &str = "mixture_machine";
const BLOCK_HASHES: &str = "SysvarRecentB1ockHashes11111111111111111111";
#[program]
pub mod mixture_machine {

    use super::*;

    pub fn compose_nft<'info>(
        ctx: Context<'_, '_, '_, 'info, ComposeNFT<'info>>,
        creator_bump: u8,
        // children_mint: Vec<Pubkey> // child NFT mint account
        // children_ata: Vec<Pubkey>, // child NFT ata pubkey
    ) -> ProgramResult{
        let mixture_machine = &mut ctx.accounts.mixture_machine;
        let mixture_machine_creator = &ctx.accounts.mixture_machine_creator;
        // msg!("{} | {}", &mixture_machine.key(), &mixture_machine_creator.key);
        // let clock = &ctx.accounts.clock; // delete this.
        let payer = &ctx.accounts.payer;
        let token_program = &ctx.accounts.token_program;
        //Account name the same for IDL compatability
        let recent_slothashes = &ctx.accounts.recent_blockhashes;
        let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;

        // let mm_key = mixture_machine.key();
        // let authority_seeds = [PREFIX.as_bytes(), mm_key.as_ref()];
        // let mm_id = mixture_machine::id();
        // let (mm_pda, mm_bump) = Pubkey::find_program_address(&authority_seeds, &mm_id);
        // msg!("{} | {} | {}", &mixture_machine.key(), &mixture_machine_creator.key, &mm_pda);
        // msg!("{}", &mm_bump);

        if recent_slothashes.key().to_string() == BLOCK_HASHES {
            msg!("recent_blockhashes is deprecated and will break soon");
        }
        if recent_slothashes.key() != sysvar::slot_hashes::id()
            && recent_slothashes.key().to_string() != BLOCK_HASHES
        {
            return Err(ErrorCode::IncorrectSlotHashesPubkey.into());
        }

        let mut remaining_accounts_counter: usize = 0;
        if ctx.remaining_accounts.len() <= remaining_accounts_counter {
            return Err(ErrorCode::ChildrenAuthorityMissing.into());
        }

        let children_number = ctx.remaining_accounts.len() / 4; // 나중에 checked_div로 바꿔주기

        // order of remaining accounts: child transfer authority - child mint - child ata - child vault
        for _i in 0..children_number {
            // child NFT transfer authority
            let child_authority_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            // mint account of child NFT
            let child_mint = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            // minter's ata of child NFT
            let child_ata_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            // program's vault pda of child NFT
            let child_vault_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;

            // msg!("{} | {}", &child_authority_info.key, &child_mint.key);
            // msg!("{} | {}", &child_ata_info.key, &child_vault_info.key);

            // creating program's vault account of child NFT
            let child_ata = assert_is_ata(child_ata_info, &payer.key(), &child_mint.key)?;        
            // let vault_infos = vec![
            //     ctx.accounts.payer.to_account_info(), // funding_account.clone(),
            //     child_vault_info.clone(), // associated_account.clone(),
            //     payer_wallet.to_account_info(),// mixture_machine_creator.to_account_info(), // fee_owner_acct.clone(),
            //     child_mint.clone(),// mint_account.clone(),
            //     ctx.accounts.system_program.to_account_info(), // sys_program_acct.clone(),
            //     ctx.accounts.token_program.to_account_info(), // spl_token_program_acct.clone(),
            //     ctx.accounts.rent.to_account_info(), // sys_rent_acct.clone(),
            // ];
            
            // invoke(
            //     &create_associated_token_account(
            //         &ctx.accounts.payer.key,
            //         &payer_wallet.key, //&mixture_machine_creator.key(),
            //         &child_mint.key,
            //     ),
            //     vault_infos.as_slice(),
            // )?;
            
            // transferring ownership of child NFT (NFT minter -> Mixture PDA)
            if child_ata.amount < 1 {
                return Err(ErrorCode::NotEnoughTokens.into());
            }

            // msg!("c string");
            spl_token_transfer(TokenTransferParams {
                source: child_ata_info.clone(), //token_account_info.clone(),
                destination: child_vault_info.clone(), //wallet.to_account_info(),
                authority: child_authority_info.clone(), //transfer_authority_info.clone(),
                authority_signer_seeds: &[],
                token_program: token_program.to_account_info(),
                amount: 1,
            })?;

            // msg!("d string");
        }

        // child NFT A의 Ownership 이동에 사용 용도 변경 (부모 NFT minter -> Mixture PDA)
        // if let Some(mint) = mixture_machine.token_mint {
        // let token_account_info = &ctx.remaining_accounts[remaining_accounts_counter];
        // remaining_accounts_counter += 1;
        // let transfer_authority_info = &ctx.remaining_accounts[remaining_accounts_counter];
        // let token_account = assert_is_ata(token_account_info, &payer.key(), &mint)?;

        // if token_account.amount < 1 {
        //     return Err(ErrorCode::NotEnoughTokens.into());
        // }

        // spl_token_transfer(TokenTransferParams {
        //     source: token_account_info.clone(),
        //     destination: wallet.to_account_info(),
        //     authority: transfer_authority_info.clone(),
        //     authority_signer_seeds: &[],
        //     token_program: token_program.to_account_info(),
        //     amount: 1,
        // })?;
        // }
        // let data = recent_slothashes.data.borrow();
        // let most_recent = array_ref![data, 4, 8];

        // let index = u64::from_le_bytes(*most_recent);
        // let modded: usize = index
        //     .checked_rem(mixture_machine.data.items_available)
        //     .ok_or(ErrorCode::NumericalOverflowError)? as usizmixture_machinee;

        // let config_line = get_config_line(&mixture_machine, modded, mixture_machine.items_redeemed)?;

        let mm_key = mixture_machine.key();
        let authority_seeds = [PREFIX.as_bytes(), mm_key.as_ref(), &[creator_bump]];


        let mut creators: Vec<mpl_token_metadata::state::Creator> =
            vec![mpl_token_metadata::state::Creator {
                address: mixture_machine_creator.key(),
                verified: true,
                share: 0,
            }];
        
        creators.push(mpl_token_metadata::state::Creator {
            address: mixture_machine.key(),
            verified: false,
            share: 0,
        });

        // msg!("e string");

        for c in &mixture_machine.data.creators {
            creators.push(mpl_token_metadata::state::Creator {
                address: c.address,
                verified: false,
                share: c.share,
            });
        }

        // msg!("f string");

        let metadata_infos = vec![
            ctx.accounts.metadata.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.mint_authority.to_account_info(),
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.token_metadata_program.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            mixture_machine_creator.to_account_info(),
        ];

        // let master_edition_infos = vec![
        //     ctx.accounts.master_edition.to_account_info(),
        //     ctx.accounts.mint.to_account_info(),
        //     ctx.accounts.mint_authority.to_account_info(),
        //     ctx.accounts.payer.to_account_info(),
        //     ctx.accounts.metadata.to_account_info(),
        //     ctx.accounts.token_metadata_program.to_account_info(),
        //     ctx.accounts.token_program.to_account_info(),
        //     ctx.accounts.system_program.to_account_info(),
        //     ctx.accounts.rent.to_account_info(),
        //     mixture_machine_creator.to_account_info(),
        // ];
        msg!("Before metadata");
        sol_log_compute_units();

        invoke_signed(
            &create_metadata_accounts(
                *ctx.accounts.token_metadata_program.key,
                *ctx.accounts.metadata.key,
                *ctx.accounts.mint.key,
                *ctx.accounts.mint_authority.key,
                *ctx.accounts.payer.key,
                mixture_machine_creator.key(),
                mixture_machine.data.name.clone(),
                mixture_machine.data.symbol.clone(),
                mixture_machine.data.uri.clone(), 
                Some(creators),
                0, // mixture_machine.data.seller_fee_basis_points,
                true,
                false, // candy_machine.data.is_mutable,
            ),
            metadata_infos.as_slice(),
            &[&authority_seeds],
        )?;

        msg!("Before instr check");
        sol_log_compute_units();

        let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();

        let instruction_sysvar = instruction_sysvar_account_info.data.borrow();

        let mut idx = 0;
        let num_instructions = read_u16(&mut idx, &instruction_sysvar)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let associated_token =
            Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();

        for index in 0..num_instructions {
            let mut current = 2 + (index * 2) as usize;
            let start = read_u16(&mut current, &instruction_sysvar).unwrap();

            current = start as usize;
            let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
            current += (num_accounts as usize) * (1 + 32);
            let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

            if program_id != mixture_machine::id()
                && program_id != spl_token::id()
                && program_id != anchor_lang::solana_program::system_program::ID
                && program_id != associated_token
            {
                msg!("Transaction had ix with program id {}", program_id);
                return Err(ErrorCode::SuspiciousTransaction.into());
            }
        }

        msg!("At the end");
        sol_log_compute_units();
        Ok(())
    }


    pub fn decompose_nft<'info>(
        ctx: Context<'_, '_, '_, 'info, DecomposeNFT<'info>>,
        creator_bump: u8,
    ) -> ProgramResult{
        // get mixture_machine PDA from parent NFT's metadata, in creators[1].
        let mixture_machine = &mut ctx.accounts.mixture_machine;
        let payer = &ctx.accounts.payer;
        let token_program = &ctx.accounts.token_program;
        let parent_token_mint = &ctx.accounts.parent_token_mint;
        //Account name the same for IDL compatability
        let recent_slothashes = &ctx.accounts.recent_blockhashes;
        let instruction_sysvar_account = &ctx.accounts.instruction_sysvar_account;

        if recent_slothashes.key().to_string() == BLOCK_HASHES {
            msg!("recent_blockhashes is deprecated and will break soon");
        }
        if recent_slothashes.key() != sysvar::slot_hashes::id()
            && recent_slothashes.key().to_string() != BLOCK_HASHES
        {
            return Err(ErrorCode::IncorrectSlotHashesPubkey.into());
        }

        let mut remaining_accounts_counter: usize = 0;
        if ctx.remaining_accounts.len() <= remaining_accounts_counter {
            return Err(ErrorCode::ChildrenAuthorityMissing.into());
        }

        let children_number = ctx.remaining_accounts.len() / 2; // 나중에 checked_div로 바꿔주기

        let creator_key = &ctx.accounts.mixture_machine_creator;
        let mm_seeds = [PREFIX.as_bytes()];
        let mm_id = mixture_machine::id();
        let (mm_pda, mm_bump) = Pubkey::find_program_address(&mm_seeds, &mm_id);
        let authority_seeds = [PREFIX.as_bytes(), &[mm_bump]];

        msg!("{} | {} | {}", &creator_key.key(), &mm_pda, &mm_bump);

        msg!("a string");
        // order of remaining accounts: child transfer authority - child mint - child ata - child vault
        for _i in 0..children_number {
            // mint account of child NFT, get this from getTokenAccountsByOwner of "mixture PDA".
            // let child_mint = &ctx.remaining_accounts[remaining_accounts_counter];
            // remaining_accounts_counter += 1;
            // user's ata of child NFT, for return.
            let child_ata_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            // ata of child NFT owned by "mixture PDA", get this from getTokenAccountsByOwner of "mixture PDA".
            let child_vault_info = &ctx.remaining_accounts[remaining_accounts_counter];
            remaining_accounts_counter += 1;
            msg!("b string");
            // msg!("{} | {}", &child_authority_info.key, &child_mint.key);
            // msg!("{} | {}", &child_ata_info.key, &child_vault_info.key);

            // msg!("c string");
            // spl_token_transfer(TokenTransferParams {
            //     source: child_ata_info.clone(), //token_account_info.clone(),
            //     destination: child_vault_info.clone(), //wallet.to_account_info(),
            //     authority: child_authority_info.clone(), //transfer_authority_info.clone(),
            //     authority_signer_seeds: &[],
            //     token_program: token_program.to_account_info(),
            //     amount: 1,
            // })?;

            let transfer_infos = vec![
                child_vault_info.clone(),
                child_ata_info.clone(),
                creator_key.to_account_info(),
                token_program.to_account_info(),
            ];
            msg!("{} | {} ", &child_vault_info.key, &child_ata_info.key);

            invoke_signed(
                &spl_token::instruction::transfer(
                    token_program.key,
                    child_vault_info.key,
                    child_ata_info.key,
                    &creator_key.key,
                    &[],
                    1,
                )?,
                transfer_infos.as_slice(),
                &[&authority_seeds],
            )?;

            msg!("c string");
        }

        msg!("Before parent burn");
        sol_log_compute_units();


        spl_token_burn(TokenBurnParams {
            mint: parent_token_mint.to_account_info(),
            source: ctx.accounts.parent_token_account.to_account_info(),
            amount: 1,
            authority: ctx.accounts.parent_burn_authority.to_account_info(),
            authority_signer_seeds: None,
            token_program: token_program.to_account_info(),
        })?;


        msg!("Before instr check");
        sol_log_compute_units();

        let instruction_sysvar_account_info = instruction_sysvar_account.to_account_info();

        let instruction_sysvar = instruction_sysvar_account_info.data.borrow();

        let mut idx = 0;
        let num_instructions = read_u16(&mut idx, &instruction_sysvar)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        let associated_token =
            Pubkey::from_str("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL").unwrap();

        for index in 0..num_instructions {
            let mut current = 2 + (index * 2) as usize;
            let start = read_u16(&mut current, &instruction_sysvar).unwrap();

            current = start as usize;
            let num_accounts = read_u16(&mut current, &instruction_sysvar).unwrap();
            current += (num_accounts as usize) * (1 + 32);
            let program_id = read_pubkey(&mut current, &instruction_sysvar).unwrap();

            if program_id != mixture_machine::id()
                && program_id != spl_token::id()
                && program_id != anchor_lang::solana_program::system_program::ID
                && program_id != associated_token
            {
                msg!("Transaction had ix with program id {}", program_id);
                return Err(ErrorCode::SuspiciousTransaction.into());
            }
        }

        msg!("At the end");
        sol_log_compute_units();
        Ok(())
    }

    // pub fn add_config_lines(
    //     ctx: Context<AddConfigLines>,
    //     index: u32,
    //     config_lines: Vec<ConfigLine>,
    // ) -> ProgramResult {
    //     let mixture_machine = &mut ctx.accounts.mixture_machine;
    //     let account = mixture_machine.to_account_info();
    //     let current_count = get_config_count(&account.data.borrow_mut())?;
    //     let mut data = account.data.borrow_mut();
    //     let mut fixed_config_lines = vec![];
    //     // No risk overflow because you literally cant store this many in an account
    //     // going beyond u32 only happens with the hidden store candies, which dont use this.
    //     if index > (mixture_machine.data.items_available as u32) - 1 {
    //         return Err(ErrorCode::IndexGreaterThanLength.into());
    //     }
    //     if mixture_machine.data.hidden_settings.is_some() {
    //         return Err(ErrorCode::HiddenSettingsConfigsDoNotHaveConfigLines.into());
    //     }
    //     for line in &config_lines {
    //         let mut array_of_zeroes = vec![];
    //         while array_of_zeroes.len() < MAX_NAME_LENGTH - line.name.len() {
    //             array_of_zeroes.push(0u8);
    //         }
    //         let name = line.name.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        
    //         let mut array_of_zeroes = vec![];
    //         while array_of_zeroes.len() < MAX_URI_LENGTH - line.uri.len() {
    //             array_of_zeroes.push(0u8);
    //         }
    //         let uri = line.uri.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
    //         fixed_config_lines.push(ConfigLine { name, uri })
    //     }

    //     let as_vec = fixed_config_lines.try_to_vec()?;
    //     // remove unneeded u32 because we're just gonna edit the u32 at the front
    //     let serialized: &[u8] = &as_vec.as_slice()[4..];

    //     let position = CONFIG_ARRAY_START + 4 + (index as usize) * CONFIG_LINE_SIZE;

    //     let array_slice: &mut [u8] =
    //         &mut data[position..position + fixed_config_lines.len() * CONFIG_LINE_SIZE];

    //     array_slice.copy_from_slice(serialized);

    //     let bit_mask_vec_start = CONFIG_ARRAY_START
    //         + 4
    //         + (mixture_machine.data.items_available as usize) * CONFIG_LINE_SIZE
    //         + 4;

    //     let mut new_count = current_count;
    //     for i in 0..fixed_config_lines.len() {
    //         let position = (index as usize)
    //             .checked_add(i)
    //             .ok_or(ErrorCode::NumericalOverflowError)?;
    //         let my_position_in_vec = bit_mask_vec_start
    //             + position
    //             .checked_div(8)
    //             .ok_or(ErrorCode::NumericalOverflowError)?;
    //         let position_from_right = 7 - position
    //             .checked_rem(8)
    //             .ok_or(ErrorCode::NumericalOverflowError)?;
    //         let mask = u8::pow(2, position_from_right as u32);

    //         let old_value_in_vec = data[my_position_in_vec];
    //         data[my_position_in_vec] = data[my_position_in_vec] | mask;
    //         msg!(
    //             "My position in vec is {} my mask is going to be {}, the old value is {}",
    //             position,
    //             mask,
    //             old_value_in_vec
    //         );
    //         msg!(
    //             "My new value is {} and my position from right is {}",
    //             data[my_position_in_vec],
    //             position_from_right
    //         );
    //         if old_value_in_vec != data[my_position_in_vec] {
    //             msg!("Increasing count");
    //             new_count = new_count
    //                 .checked_add(1)
    //                 .ok_or(ErrorCode::NumericalOverflowError)?;
    //         }
    //     }

    //     // plug in new count.
    //     data[CONFIG_ARRAY_START..CONFIG_ARRAY_START + 4]
    //         .copy_from_slice(&(new_count as u32).to_le_bytes());

    //     Ok(())
    // }

    pub fn initialize_mixture_machine(
        ctx: Context<InitializeMixtureMachine>,
        data: MixtureMachineData,
    ) -> ProgramResult {
        let mixture_machine_account = &mut ctx.accounts.mixture_machine;

        if data.uuid.len() != 6 {
            return Err(ErrorCode::UuidMustBeExactly6Length.into());
        }

        let mut mixture_machine = MixtureMachine {
            data,
            authority: *ctx.accounts.authority.key,
            // wallet: *ctx.accounts.wallet.key,
            // token_mint: None,
            // items_redeemed: 0,
        };
        
        // // token_mint 관련 설정. 필요X
        // if ctx.remaining_accounts.len() > 0 {
        //     let token_mint_info = &ctx.remaining_accounts[0];
        //     let _token_mint: Mint = assert_initialized(&token_mint_info)?;
        //     let token_account: spl_token::state::Account =
        //         assert_initialized(&ctx.accounts.wallet)?;

        //     assert_owned_by(&token_mint_info, &spl_token::id())?;
        //     assert_owned_by(&ctx.accounts.wallet, &spl_token::id())?;

        //     if token_account.mint != *token_mint_info.key {
        //         return Err(ErrorCode::MintMismatch.into());
        //     }

        //     mixture_machine.token_mint = Some(*token_mint_info.key);
        // }

        // symbol의 남은 칸 0으로 채워줌
        let mut array_of_zeroes = vec![];
        while array_of_zeroes.len() < MAX_SYMBOL_LENGTH - mixture_machine.data.symbol.len() {
            array_of_zeroes.push(0u8);
        }
        let new_symbol =
            mixture_machine.data.symbol.clone() + std::str::from_utf8(&array_of_zeroes).unwrap();
        mixture_machine.data.symbol = new_symbol;

        // - 1 because we are going to be a creator
        if mixture_machine.data.creators.len() > MAX_CREATOR_LIMIT - 1 {
            return Err(ErrorCode::TooManyCreators.into());
        }

        let mut new_data = MixtureMachine::discriminator().try_to_vec().unwrap();
        new_data.append(&mut mixture_machine.try_to_vec().unwrap());
        let mut data = mixture_machine_account.data.borrow_mut();
        // god forgive me couldnt think of better way to deal with this
        for i in 0..new_data.len() {
            data[i] = new_data[i];
        }

        // let vec_start = CONFIG_ARRAY_START
        //     + 4
        //     + (mixture_machine.data.items_available as usize) * CONFIG_LINE_SIZE;
        // let as_bytes = (mixture_machine
        //     .data
        //     .items_available
        //     .checked_div(8)
        //     .ok_or(ErrorCode::NumericalOverflowError)? as u32)
        //     .to_le_bytes();
        // for i in 0..4 {
        //     data[vec_start + i] = as_bytes[i]
        // }

        Ok(())
    }
}

// fn get_space_for_candy(data: MixtureMachineData) -> core::result::Result<usize, ProgramError> {
//     let num = if data.hidden_settings.is_some() {
//         CONFIG_ARRAY_START
//     } else {
//         CONFIG_ARRAY_START
//             + 4
//             + (data.items_available as usize) * CONFIG_LINE_SIZE
//             + 8
//             + 2 * ((data
//             .items_available
//             .checked_div(8)
//             .ok_or(ErrorCode::NumericalOverflowError)?
//             + 1) as usize)
//     };

//     Ok(num)
// }

/// Create a new candy machine.
#[derive(Accounts)]
#[instruction(data: MixtureMachineData)]
pub struct InitializeMixtureMachine<'info> {
    #[account(zero, rent_exempt = skip, constraint = mixture_machine.to_account_info().owner == program_id)]
    mixture_machine: UncheckedAccount<'info>,
    authority: UncheckedAccount<'info>,
    payer: Signer<'info>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
}

/// Add multiple config lines to the candy machine.
// #[derive(Accounts)]
// pub struct AddConfigLines<'info> {
//     #[account(mut, has_one = authority)]
//     mixture_machine: Account<'info, MixtureMachine>,
//     authority: Signer<'info>,
// }

/// Withdraw SOL from candy machine account.
// #[derive(Accounts)]
// pub struct WithdrawFunds<'info> {
//     #[account(mut, has_one = authority)]
//     mixture_machine: Account<'info, MixtureMachine>,
//     #[account(address = mixture_machine.authority)]
//     authority: Signer<'info>,
// }

/// Mint a new NFT pseudo-randomly from the config array.
#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct ComposeNFT<'info> {
    #[account(mut)]
    mixture_machine: Account<'info, MixtureMachine>,
    #[account(
        seeds=[PREFIX.as_bytes(), mixture_machine.key().as_ref()], bump=creator_bump
    )]
    mixture_machine_creator: UncheckedAccount<'info>,
    payer: Signer<'info>,
    // With the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.
    #[account(mut)]
    metadata: UncheckedAccount<'info>,
    // Parent NFT's token mint account with no metadata
    #[account(mut)]
    mint: UncheckedAccount<'info>,
    mint_authority: Signer<'info>,
    update_authority: Signer<'info>, // delete this in future.
    #[account(address = mpl_token_metadata::id())]
    token_metadata_program: UncheckedAccount<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    // clock: Sysvar<'info, Clock>,
    // #[account(address = sysvar::recent_blockhashes::ID)]
    recent_blockhashes: UncheckedAccount<'info>,
    #[account(address = sysvar::instructions::id())]
    instruction_sysvar_account: UncheckedAccount<'info>,
    // transfer_authority: Signer<'info>, // child NFT transfer authority
}


#[derive(Accounts)]
#[instruction(creator_bump: u8)]
pub struct DecomposeNFT<'info> {
    // mixture_machine PDA which is associated with parent NFT and owns child NFTs.
    #[account(mut)]
    mixture_machine: Account<'info, MixtureMachine>,
    #[account(
        seeds=[PREFIX.as_bytes()], bump=creator_bump
    )]
    mixture_machine_creator: UncheckedAccount<'info>,
    payer: Signer<'info>,
    // With the following accounts we aren't using anchor macros because they are CPI'd
    // through to token-metadata which will do all the validations we need on them.
    #[account(mut)]
    parent_token_mint: UncheckedAccount<'info>,
    #[account(mut)]
    parent_token_account: UncheckedAccount<'info>,
    parent_burn_authority: Signer<'info>,
    token_program: Program<'info, Token>,
    system_program: Program<'info, System>,
    rent: Sysvar<'info, Rent>,
    // clock: Sysvar<'info, Clock>,
    // #[account(address = sysvar::recent_blockhashes::ID)]
    recent_blockhashes: UncheckedAccount<'info>,
    #[account(address = sysvar::instructions::id())]
    instruction_sysvar_account: UncheckedAccount<'info>,
    // transfer_authority: Signer<'info>, // child NFT transfer authority
}

/// Candy machine state and config data.
#[account]
#[derive(Default)]
pub struct MixtureMachine {
    pub authority: Pubkey,
    pub data: MixtureMachineData,
    // there's a borsh vec u32 denoting how many actual lines of data there are currently (eventually equals items available)
    // There is actually lines and lines of data after this but we explicitly never want them deserialized.
    // here there is a borsh vec u32 indicating number of bytes in bitmask array.
    // here there is a number of bytes equal to ceil(max_number_of_lines/8) and it is a bit mask used to figure out when to increment borsh vec u32
}

/// Candy machine settings data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct MixtureMachineData {
    pub uuid: String,
    // pub price: u64,
    /// The symbol for the asset
    pub symbol: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    // pub seller_fee_basis_points: u16,
    // pub max_supply: u64,
    // pub is_mutable: bool,
    // pub retain_authority: bool,
    // pub go_live_date: Option<i64>,
    // pub end_settings: Option<EndSettings>,
    pub creators: Vec<Creator>,
    // pub hidden_settings: Option<HiddenSettings>,
    // pub whitelist_mint_settings: Option<WhitelistMintSettings>,
    // pub items_available: u64,
    /// If [`Some`] requires gateway tokens on mint
    // pub gatekeeper: Option<GatekeeperConfig>,
    pub name: String,
    pub uri: String,
}

// pub const CONFIG_ARRAY_START: usize = 8 + // key
//     32 + // authority
//     32 + //wallet
//     33 + // token mint
//     4 + 6 + // uuid
//     8 + // price
//     8 + // items available
//     9 + // go live
//     10 + // end settings
//     4 + MAX_SYMBOL_LENGTH + // u32 len + symbol
//     2 + // seller fee basis points
//     4 + MAX_CREATOR_LIMIT*MAX_CREATOR_LEN + // optional + u32 len + actual vec
//     8 + //max supply
//     1 + // is mutable
//     1 + // retain authority
//     1 + // option for hidden setting
//     4 + MAX_NAME_LENGTH + // name length,
//     4 + MAX_URI_LENGTH + // uri length,
//     32 + // hash
//     4 +  // max number of lines;
//     8 + // items redeemed
//     1 + // whitelist option
//     1 + // whitelist mint mode
//     1 + // allow presale
//     9 + // discount price
//     32 + // mint key for whitelist
//     1 + 32 + 1 // gatekeeper
// ;

/// Hidden Settings for large mints used with offline data.
// #[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
// pub struct HiddenSettings {
//     pub name: String,
//     pub uri: String,
//     pub hash: [u8; 32],
// }

// pub fn get_config_count(data: &RefMut<&mut [u8]>) -> core::result::Result<usize, ProgramError> {
//     return Ok(u32::from_le_bytes(*array_ref![data, CONFIG_ARRAY_START, 4]) as usize);
// }

// pub fn get_good_index(
//     arr: &mut RefMut<&mut [u8]>,
//     items_available: usize,
//     index: usize,
//     pos: bool,
// ) -> core::result::Result<(usize, bool), ProgramError> {
//     let mut index_to_use = index;
//     let mut taken = 1;
//     let mut found = false;
//     let bit_mask_vec_start = CONFIG_ARRAY_START
//         + 4
//         + (items_available) * CONFIG_LINE_SIZE
//         + 4
//         + items_available
//         .checked_div(8)
//         .ok_or(ErrorCode::NumericalOverflowError)?
//         + 4;

//     while taken > 0 && index_to_use < items_available {
//         let my_position_in_vec = bit_mask_vec_start
//             + index_to_use
//             .checked_div(8)
//             .ok_or(ErrorCode::NumericalOverflowError)?;
//         /*msg!(
//             "My position is {} and value there is {}",
//             my_position_in_vec,
//             arr[my_position_in_vec]
//         );*/
//         if arr[my_position_in_vec] == 255 {
//             //msg!("We are screwed here, move on");
//             let eight_remainder = 8 - index_to_use
//                 .checked_rem(8)
//                 .ok_or(ErrorCode::NumericalOverflowError)?;
//             let reversed = 8 - eight_remainder + 1;
//             if (eight_remainder != 0 && pos) || (reversed != 0 && !pos) {
//                 //msg!("Moving by {}", eight_remainder);
//                 if pos {
//                     index_to_use += eight_remainder;
//                 } else {
//                     if index_to_use < 8 {
//                         break;
//                     }
//                     index_to_use -= reversed;
//                 }
//             } else {
//                 //msg!("Moving by 8");
//                 if pos {
//                     index_to_use += 8;
//                 } else {
//                     index_to_use -= 8;
//                 }
//             }
//         } else {
//             let position_from_right = 7 - index_to_use
//                 .checked_rem(8)
//                 .ok_or(ErrorCode::NumericalOverflowError)?;
//             let mask = u8::pow(2, position_from_right as u32);

//             taken = mask & arr[my_position_in_vec];
//             if taken > 0 {
//                 //msg!("Index to use {} is taken", index_to_use);
//                 if pos {
//                     index_to_use += 1;
//                 } else {
//                     if index_to_use == 0 {
//                         break;
//                     }
//                     index_to_use -= 1;
//                 }
//             } else if taken == 0 {
//                 //msg!("Index to use {} is not taken, exiting", index_to_use);
//                 found = true;
//                 arr[my_position_in_vec] = arr[my_position_in_vec] | mask;
//             }
//         }
//     }

//     Ok((index_to_use, found))
// }

// pub fn get_config_line<'info>(
//     a: &Account<'info, MixtureMachine>,
//     index: usize,
//     mint_number: u64,
// ) -> core::result::Result<ConfigLine, ProgramError> {
//     if let Some(hs) = &a.data.hidden_settings {
//         return Ok(ConfigLine {
//             name: hs.name.clone() + "#" + &(mint_number + 1).to_string(),
//             uri: hs.uri.clone(),
//         });
//     }
//     msg!("Index is set to {:?}", index);
//     let a_info = a.to_account_info();

//     let mut arr = a_info.data.borrow_mut();

//     let (mut index_to_use, good) =
//         get_good_index(&mut arr, a.data.items_available as usize, index, true)?;
//     if !good {
//         let (index_to_use_new, good_new) =
//             get_good_index(&mut arr, a.data.items_available as usize, index, false)?;
//         index_to_use = index_to_use_new;
//         if !good_new {
//             return Err(ErrorCode::CannotFindUsableConfigLine.into());
//         }
//     }

//     msg!(
//         "Index actually ends up due to used bools {:?}",
//         index_to_use
//     );
//     if arr[CONFIG_ARRAY_START + 4 + index_to_use * (CONFIG_LINE_SIZE)] == 1 {
//         return Err(ErrorCode::CannotFindUsableConfigLine.into());
//     }

//     let data_array = &mut arr[CONFIG_ARRAY_START + 4 + index_to_use * (CONFIG_LINE_SIZE)
//         ..CONFIG_ARRAY_START + 4 + (index_to_use + 1) * (CONFIG_LINE_SIZE)];

//     let mut name_vec = vec![];
//     let mut uri_vec = vec![];
//     for i in 4..4 + MAX_NAME_LENGTH {
//         if data_array[i] == 0 {
//             break;
//         }
//         name_vec.push(data_array[i])
//     }
//     for i in 8 + MAX_NAME_LENGTH..8 + MAX_NAME_LENGTH + MAX_URI_LENGTH {
//         if data_array[i] == 0 {
//             break;
//         }
//         uri_vec.push(data_array[i])
//     }
//     let config_line: ConfigLine = ConfigLine {
//         name: match String::from_utf8(name_vec) {
//             Ok(val) => val,
//             Err(_) => return Err(ErrorCode::InvalidString.into()),
//         },
//         uri: match String::from_utf8(uri_vec) {
//             Ok(val) => val,
//             Err(_) => return Err(ErrorCode::InvalidString.into()),
//         },
//     };

//     Ok(config_line)
// }

// /// Individual config line for storing NFT data pre-mint.
// pub const CONFIG_LINE_SIZE: usize = 4 + MAX_NAME_LENGTH + 4 + MAX_URI_LENGTH;
// #[derive(AnchorSerialize, AnchorDeserialize, Debug)]
// pub struct ConfigLine {
//     pub name: String,
//     /// URI pointing to JSON representing the asset
//     pub uri: String,
// }

// Unfortunate duplication of token metadata so that IDL picks it up.

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

#[error]
pub enum ErrorCode {
    #[msg("Missing children NFT transfer authority when required")]
    ChildrenAuthorityMissing,
    #[msg("Account does not have correct owner!")]
    IncorrectOwner,
    #[msg("Account is not initialized!")]
    Uninitialized,
    // #[msg("Mint Mismatch!")]
    // MintMismatch,
    // #[msg("Index greater than length!")]
    // IndexGreaterThanLength,
    #[msg("Numerical overflow error!")]
    NumericalOverflowError,
    #[msg("Can only provide up to 4 creators to mixture machine (because mixture machine is one)!")]
    TooManyCreators,
    #[msg("Uuid must be exactly of 6 length")]
    UuidMustBeExactly6Length,
    #[msg("Not enough tokens to pay for this minting")]
    NotEnoughTokens,
    // #[msg("Not enough SOL to pay for this minting")]
    // NotEnoughSOL,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    // #[msg("Breeding machine is empty!")]
    // BreedingMachineEmpty,
    // #[msg("Breeding machine is not live!")]
    // BreedingMachineNotLive,
    // #[msg("Configs that are using hidden uris do not have config lines, they have a single hash representing hashed order")]
    // HiddenSettingsConfigsDoNotHaveConfigLines,
    // #[msg("Cannot change number of lines unless is a hidden config")]
    // CannotChangeNumberOfLines,
    #[msg("Derived key invalid")]
    DerivedKeyInvalid,
    #[msg("Public key mismatch")]
    PublicKeyMismatch,
    // #[msg("No whitelist token present")]
    // NoWhitelistToken,
    #[msg("Token burn failed")]
    TokenBurnFailed,
    // #[msg("Missing gateway app when required")]
    // GatewayAppMissing,
    // #[msg("Invalid gateway token expire time")]
    // GatewayTokenExpireTimeInvalid,
    // #[msg("Missing gateway network expire feature when required")]
    // NetworkExpireFeatureMissing,
    // #[msg("Unable to find an unused config line near your random number index")]
    // CannotFindUsableConfigLine,
    // #[msg("Invalid string")]
    // InvalidString,
    #[msg("Suspicious transaction detected")]
    SuspiciousTransaction,
    // #[msg("Cannot Switch to Hidden Settings after items available is greater than 0")]
    // CannotSwitchToHiddenSettings,
    #[msg("Incorrect SlotHashes PubKey")]
    IncorrectSlotHashesPubkey,
}