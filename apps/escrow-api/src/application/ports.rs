use crate::{
    application::errors::AppError,
    domain::escrow::{Escrow, NewEscrow},
};

#[allow(async_fn_in_trait)]
pub trait EscrowRepository: Send + Sync + 'static {
    async fn list(&self, buyer: Option<&str>) -> Result<Vec<Escrow>, AppError>;
    async fn create(&self, input: &NewEscrow) -> Result<Escrow, AppError>;
    async fn update_status(&self, id: &str, status: &str) -> Result<Escrow, AppError>;
    async fn delete(&self, id: &str) -> Result<(), AppError>;
}
