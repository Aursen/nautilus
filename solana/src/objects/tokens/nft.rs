use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::instruction::AuthorityType;

use crate::{
    cpi, Create, Metadata, Mint, Mut, NautilusAccountInfo, NautilusMut, NautilusSigner, Signer,
    Wallet,
};

/// The Nautilus object representing the combination of a mint account and a token metadata account.
///
/// This Nautilus object is designed for easily working with tokens and metadata together.
///
/// It's comprised of both a `Mint` and `Metadata` struct, which allows you to access either individually, and
/// most of it's implemented methods access the mint account.
#[derive(Clone)]
pub struct Nft<'a> {
    pub mint: Mint<'a>,
    pub metadata: Metadata<'a>,
}

impl<'a> Nft<'a> {
    /// Instantiate a new `Nft` without loading the account inner data from on-chain.
    pub fn new(
        mint_account: Box<AccountInfo<'a>>,
        metadata_account: Box<AccountInfo<'a>>,
        token_program: Box<AccountInfo<'a>>,
        token_metadata_program: Box<AccountInfo<'a>>,
    ) -> Self {
        Self {
            mint: Mint::new(mint_account, token_program),
            metadata: Metadata::new(metadata_account, token_metadata_program),
        }
    }

    /// Instantiate a new `Nft` and load the account inner data from on-chain.
    pub fn load(
        mint_account: Box<AccountInfo<'a>>,
        metadata_account: Box<AccountInfo<'a>>,
        token_program: Box<AccountInfo<'a>>,
        token_metadata_program: Box<AccountInfo<'a>>,
    ) -> Result<Self, ProgramError> {
        Ok(Self {
            mint: Mint::load(mint_account, token_program)?,
            metadata: Metadata::load(metadata_account, token_metadata_program)?,
        })
    }
}

impl<'a> NautilusAccountInfo<'a> for Nft<'a> {
    fn account_info(&self) -> Box<AccountInfo<'a>> {
        self.mint.account_info()
    }

    fn key(&self) -> &'a Pubkey {
        self.mint.account_info.key
    }

    fn is_signer(&self) -> bool {
        self.mint.account_info.is_signer
    }

    fn is_writable(&self) -> bool {
        self.mint.account_info.is_writable
    }

    fn lamports(&self) -> u64 {
        self.mint.account_info.lamports()
    }

    fn mut_lamports(&self) -> Result<std::cell::RefMut<'_, &'a mut u64>, ProgramError> {
        self.mint.account_info.try_borrow_mut_lamports()
    }

    fn owner(&self) -> &'a Pubkey {
        self.mint.account_info.owner
    }

    fn span(&self) -> Result<usize, ProgramError> {
        self.mint.span()
    }
}

impl<'a> Mut<Nft<'a>> {
    /// Mint new tokens to an associated token account.
    pub fn mint_to(
        &self,
        recipient: impl NautilusMut<'a>,
        mint_authority: impl NautilusSigner<'a>,
    ) -> ProgramResult {
        let multisigs: Option<Vec<Signer<Wallet>>> = None; // TODO: Multisig support
        cpi::token::mint_to(
            self.self_account.mint.token_program.key,
            self.clone(),
            recipient,
            mint_authority.clone(),
            multisigs.clone(),
            1,
        )?;
        cpi::token::set_authority(
            self.self_account.mint.token_program.key,
            self.clone(),
            None,
            AuthorityType::MintTokens,
            mint_authority,
            multisigs,
        )
    }

    /// Change the mint's authority.
    pub fn set_authority(
        &self,
        new_authority: Option<&Pubkey>,
        authority_type: AuthorityType,
        current_authority: impl NautilusSigner<'a>,
    ) -> ProgramResult {
        let multisigs: Option<Vec<Signer<Wallet>>> = None; // TODO: Multisig support
        cpi::token::set_authority(
            self.self_account.mint.token_program.key,
            self.clone(),
            new_authority,
            authority_type,
            current_authority,
            multisigs,
        )
    }
}

impl<'a> Create<'a, Nft<'a>> {
    /// Create a new SPL mint with a Nft Program and
    /// a new SPL metadata account with Nft Metadata Program.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &mut self,
        title: String,
        symbol: String,
        uri: String,
        mint_authority: impl NautilusSigner<'a>,
        update_authority: impl NautilusAccountInfo<'a>,
        freeze_authority: Option<impl NautilusAccountInfo<'a>>,
    ) -> ProgramResult {
        let mut create_mint: Create<Mint> = self.clone().into();
        let mut create_metadata: Create<Metadata> = self.clone().into();
        create_mint.create(0, mint_authority.clone(), freeze_authority)?;
        create_metadata.create(
            title,
            symbol,
            uri,
            self.self_account.mint.to_owned(),
            mint_authority,
            update_authority,
        )?;
        Ok(())
    }

    /// This function is the same as `create(&mut self, ..)` but allows you to specify a rent payer.
    #[allow(clippy::too_many_arguments)]
    pub fn create_with_payer(
        &mut self,
        title: String,
        symbol: String,
        uri: String,
        mint_authority: impl NautilusSigner<'a>,
        update_authority: impl NautilusAccountInfo<'a>,
        freeze_authority: Option<impl NautilusAccountInfo<'a>>,
        payer: impl NautilusSigner<'a>,
    ) -> ProgramResult {
        let mut create_mint: Create<Mint> = self.clone().into();
        let mut create_metadata: Create<Metadata> = self.clone().into();
        create_mint.create_with_payer(
            0,
            mint_authority.clone(),
            freeze_authority,
            payer.clone(),
        )?;
        create_metadata.create_with_payer(
            title,
            symbol,
            uri,
            self.self_account.mint.to_owned(),
            mint_authority,
            update_authority,
            payer,
        )?;
        Ok(())
    }
}

// Converters

impl<'a> From<Nft<'a>> for Mint<'a> {
    fn from(value: Nft<'a>) -> Self {
        value.mint
    }
}

impl<'a> From<Create<'a, Nft<'a>>> for Create<'a, Mint<'a>> {
    fn from(value: Create<'a, Nft<'a>>) -> Self {
        Self {
            self_account: value.self_account.into(),
            fee_payer: value.fee_payer,
            rent: value.rent,
            system_program: value.system_program,
        }
    }
}

impl<'a> From<Nft<'a>> for Metadata<'a> {
    fn from(value: Nft<'a>) -> Self {
        value.metadata
    }
}

impl<'a> From<Create<'a, Nft<'a>>> for Create<'a, Metadata<'a>> {
    fn from(value: Create<'a, Nft<'a>>) -> Self {
        Self {
            self_account: value.self_account.into(),
            fee_payer: value.fee_payer,
            rent: value.rent,
            system_program: value.system_program,
        }
    }
}
