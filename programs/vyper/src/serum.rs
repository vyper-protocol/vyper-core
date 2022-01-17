use anchor_lang::prelude::*;
use anchor_lang::solana_program;

pub fn allocate_serum_account<'info>(
    unpadded_len: usize,
    account: &AccountInfo<'info>,
    seeds: &[&[u8]],
    authority: &Signer<'info>,
    system_program: &Program<'info, System>,
    rent: &Sysvar<'info, Rent>,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let space = 8 + 5 + unpadded_len + 7;

    let lamports = rent.minimum_balance(space);

    let ix = solana_program::system_instruction::create_account(
        &authority.key(),
        account.key,
        lamports,
        space as u64,
        program_id,
    );

    let signer = &[&seeds[..]];

    solana_program::program::invoke_signed(
        &ix,
        &[
            authority.to_account_info().clone(),
            account.clone(),
            system_program.to_account_info().clone(),
        ],
        signer,
    )?;
    Ok(())
}
