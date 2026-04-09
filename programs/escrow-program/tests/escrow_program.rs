use anchor_lang::prelude::AnchorSerialize;
use escrow::{EscrowAccount, EscrowStatus};

#[test]
fn escrow_account_size_is_stable() {
    assert_eq!(EscrowAccount::SIZE, 90);
}

#[test]
fn pending_status_serializes() {
    let bytes = EscrowStatus::Pending
        .try_to_vec()
        .expect("status should serialize");

    assert_eq!(bytes.len(), 1);
}
