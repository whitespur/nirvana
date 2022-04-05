use anchor_lang::prelude::*;

#[account]
#[derive(Default, Debug)]
pub struct NirvCenter {
    /// The final PDA that signs
    pub signer_authority: Pubkey,

    /// Seed for PDA
    pub signer_authority_seed: Pubkey,

    /// Canonical bump
    pub signer_authority_bump: [u8; 1],

    /// Is this in debug/dev mode
    pub debug_mode: bool,

    /// Is halted, stopping all action
    pub is_halted: bool,

    /// Owner that can update a nirv center
    pub policy_owner: Pubkey,

    pub config: Pubkey,
}

impl NirvCenter {
    pub fn authority_seeds(&self) -> [&[u8]; 2] {
        [
            self.signer_authority_seed.as_ref(),
            &self.signer_authority_bump,
        ]
    }
}
