use anchor_lang::prelude::{Sysvar, Signer};

use {
    crate::{MixtureMachine, ErrorCode},
    anchor_lang::{
        prelude::{Account, AccountInfo, Clock, ProgramError, ProgramResult, Pubkey},
        solana_program::{
            program::invoke_signed,
            program_pack::{IsInitialized, Pack},
        },
    },
    spl_associated_token_account::get_associated_token_address,
};

pub fn assert_initialized<T: Pack + IsInitialized>(
    account_info: &AccountInfo,
) -> Result<T, ProgramError> {
    let account: T = T::unpack_unchecked(&account_info.data.borrow())?;
    if !account.is_initialized() {
        Err(ErrorCode::Uninitialized.into())
    } else {
        Ok(account)
    }
}

// pub fn assert_valid_go_live<'info>(
//     payer: &Signer<'info>,
//     clock: &Sysvar<Clock>,
//     mixture_machine: &Account<'info, MixtureMachine>,
// ) -> ProgramResult {
//     match mixture_machine.data.go_live_date {
//         None => {
//             if *payer.key != mixture_machine.authority {
//                 return Err(ErrorCode::MixtureMachineNotLive.into());
//             }
//         }
//         Some(val) => {
//             if clock.unix_timestamp < val && *payer.key != mixture_machine.authority {
//                 return Err(ErrorCode::MixtureMachineNotLive.into());
//             }
//         }
//     }

//     Ok(())
// }

pub fn assert_owned_by(account: &AccountInfo, owner: &Pubkey) -> ProgramResult {
    if account.owner != owner {
        Err(ErrorCode::IncorrectOwner.into())
    } else {
        Ok(())
    }
}
///TokenTransferParams
pub struct TokenTransferParams<'a: 'b, 'b> {
    /// source
    pub source: AccountInfo<'a>,
    /// destination
    pub destination: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: &'b [&'b [u8]],
    /// token_program
    pub token_program: AccountInfo<'a>,
}

#[inline(always)]
pub fn spl_token_transfer(params: TokenTransferParams<'_, '_>) -> ProgramResult {
    let TokenTransferParams {
        source,
        destination,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;

    let mut signer_seeds = vec![];
    if authority_signer_seeds.len() > 0 {
        signer_seeds.push(authority_signer_seeds)
    }

    let result = invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            source.key,
            destination.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, destination, authority, token_program],
        &signer_seeds,
    );

    result.map_err(|_| ErrorCode::TokenTransferFailed.into())
}

pub fn assert_is_ata<'a>(
    ata: &AccountInfo,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> core::result::Result<spl_token::state::Account, ProgramError> {
    assert_owned_by(ata, &spl_token::id())?;
    let ata_account: spl_token::state::Account = assert_initialized(ata)?;
    assert_keys_equal(ata_account.owner, *wallet)?;
    assert_keys_equal(get_associated_token_address(wallet, mint), *ata.key)?;
    Ok(ata_account)
}

pub fn assert_keys_equal(key1: Pubkey, key2: Pubkey) -> ProgramResult {
    if key1 != key2 {
        Err(ErrorCode::PublicKeyMismatch.into())
    } else {
        Ok(())
    }
}

/// TokenBurnParams
pub struct TokenBurnParams<'a: 'b, 'b> {
    /// mint
    pub mint: AccountInfo<'a>,
    /// source
    pub source: AccountInfo<'a>,
    /// amount
    pub amount: u64,
    /// authority
    pub authority: AccountInfo<'a>,
    /// authority_signer_seeds
    pub authority_signer_seeds: Option<&'b [&'b [u8]]>,
    /// token_program
    pub token_program: AccountInfo<'a>,
}

pub fn spl_token_burn(params: TokenBurnParams<'_, '_>) -> ProgramResult {
    let TokenBurnParams {
        mint,
        source,
        authority,
        token_program,
        amount,
        authority_signer_seeds,
    } = params;
    let mut seeds: Vec<&[&[u8]]> = vec![];
    if let Some(seed) = authority_signer_seeds {
        seeds.push(seed);
    }
    let result = invoke_signed(
        &spl_token::instruction::burn(
            token_program.key,
            source.key,
            mint.key,
            authority.key,
            &[],
            amount,
        )?,
        &[source, mint, authority, token_program],
        seeds.as_slice(),
    );
    result.map_err(|_| ErrorCode::TokenBurnFailed.into())
}

// /// Create a new account instruction
// pub fn process_create_metadata_accounts_logic(
//     program_id: &Pubkey,
//     accounts: CreateMetadataAccountsLogicArgs,
//     data: DataV2,
//     allow_direct_creator_writes: bool,
//     mut is_mutable: bool,
//     is_edition: bool,
//     add_token_standard: bool,
// ) -> ProgramResult {
//     let CreateMetadataAccountsLogicArgs {
//         metadata_account_info,
//         mint_info,
//         mint_authority_info,
//         payer_account_info,
//         update_authority_info,
//         system_account_info,
//         rent_info,
//     } = accounts;

//     let mut update_authority_key = *update_authority_info.key;
//     let existing_mint_authority = get_mint_authority(mint_info)?;
//     // IMPORTANT NOTE
//     // This allows the Metaplex Foundation to Create but not update metadata for SPL tokens that have not populated their metadata.
//     assert_mint_authority_matches_mint(&existing_mint_authority, mint_authority_info).or_else(
//         |e| {
//             // Allow seeding by the authority seed populator
//             if mint_authority_info.key == &SEED_AUTHORITY && mint_authority_info.is_signer {
//                 // When metadata is seeded, the mint authority should be able to change it
//                 if let COption::Some(auth) = existing_mint_authority {
//                     update_authority_key = auth;
//                     is_mutable = true;
//                 }
//                 Ok(())
//             } else {
//                 Err(e)
//             }
//         },
//     )?;
//     assert_owned_by(mint_info, &spl_token::id())?;

//     let metadata_seeds = &[
//         PREFIX.as_bytes(),
//         program_id.as_ref(),
//         mint_info.key.as_ref(),
//     ];
//     let (metadata_key, metadata_bump_seed) =
//         Pubkey::find_program_address(metadata_seeds, program_id);
//     let metadata_authority_signer_seeds = &[
//         PREFIX.as_bytes(),
//         program_id.as_ref(),
//         mint_info.key.as_ref(),
//         &[metadata_bump_seed],
//     ];

//     if metadata_account_info.key != &metadata_key {
//         return Err(MetadataError::InvalidMetadataKey.into());
//     }

//     create_or_allocate_account_raw(
//         *program_id,
//         metadata_account_info,
//         rent_info,
//         system_account_info,
//         payer_account_info,
//         MAX_METADATA_LEN,
//         metadata_authority_signer_seeds,
//     )?;

//     let mut metadata = Metadata::from_account_info(metadata_account_info)?;
//     let compatible_data = data.to_v1();
//     assert_data_valid(
//         &compatible_data,
//         &update_authority_key,
//         &metadata,
//         allow_direct_creator_writes,
//         update_authority_info.is_signer,
//         false,
//     )?;

//     let mint_decimals = get_mint_decimals(mint_info)?;

//     metadata.mint = *mint_info.key;
//     metadata.key = Key::MetadataV1;
//     metadata.data = data.to_v1();
//     metadata.is_mutable = is_mutable;
//     metadata.update_authority = update_authority_key;
//     assert_valid_use(&data.uses, &None)?;
//     metadata.uses = data.uses;
//     assert_collection_update_is_valid(is_edition, &None, &data.collection)?;
//     metadata.collection = data.collection;
//     if add_token_standard {
//         let token_standard = if is_edition {
//             TokenStandard::NonFungibleEdition
//         } else if mint_decimals == 0 {
//             TokenStandard::FungibleAsset
//         } else {
//             TokenStandard::Fungible
//         };
//         metadata.token_standard = Some(token_standard);
//     } else {
//         metadata.token_standard = None;
//     }
//     puff_out_data_fields(&mut metadata);

//     let edition_seeds = &[
//         PREFIX.as_bytes(),
//         program_id.as_ref(),
//         metadata.mint.as_ref(),
//         EDITION.as_bytes(),
//     ];
//     let (_, edition_bump_seed) = Pubkey::find_program_address(edition_seeds, program_id);
//     metadata.edition_nonce = Some(edition_bump_seed);
//     metadata.serialize(&mut *metadata_account_info.data.borrow_mut())?;

//     Ok(())
// }
