use crate::errors::ErrorCode;
use crate::*;

pub fn admin(nirv_center: &Account<NirvCenter>, signer: &AccountInfo) -> Result<()> {
    if !signer.key.eq(&nirv_center.policy_owner) {
        return Err(error!(ErrorCode::Unauthorized));
    }
    Ok(())
}

pub fn is_debug(nirv_center: &Account<NirvCenter>) -> Result<()> {
    if !nirv_center.debug_mode {
        return Err(error!(ErrorCode::DebugRequired));
    }
    Ok(())
}
