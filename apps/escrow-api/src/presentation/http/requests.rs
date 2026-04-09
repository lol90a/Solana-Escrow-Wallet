use serde::Deserialize;

use crate::domain::escrow::NewEscrow;

#[derive(Debug, Deserialize)]
pub struct ListEscrowsQuery {
    pub buyer: Option<String>,
}

pub type CreateEscrowRequest = NewEscrow;
