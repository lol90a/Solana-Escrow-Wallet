use std::sync::Arc;

use crate::{
    application::services::EscrowService,
    infrastructure::postgres::repository::PostgresEscrowRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub escrow_service: Arc<EscrowService<PostgresEscrowRepository>>,
}

impl AppState {
    pub fn new(escrow_service: Arc<EscrowService<PostgresEscrowRepository>>) -> Self {
        Self { escrow_service }
    }
}
