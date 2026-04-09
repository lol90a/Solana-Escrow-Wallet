use crate::{
    application::{errors::AppError, ports::EscrowRepository},
    domain::escrow::{Escrow, EscrowStatus, NewEscrow},
};

pub struct EscrowService<R> {
    repository: R,
}

impl<R> EscrowService<R>
where
    R: EscrowRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn list_escrows(&self, buyer: Option<&str>) -> Result<Vec<Escrow>, AppError> {
        self.repository.list(buyer).await
    }

    pub async fn create_escrow(&self, input: NewEscrow) -> Result<Escrow, AppError> {
        input.validate().map_err(AppError::Validation)?;
        self.repository.create(&input).await
    }

    pub async fn release_escrow(&self, id: &str) -> Result<Escrow, AppError> {
        self.repository
            .update_status(id, EscrowStatus::Completed.as_db_value())
            .await
    }

    pub async fn cancel_escrow(&self, id: &str) -> Result<Escrow, AppError> {
        self.repository
            .update_status(id, EscrowStatus::Cancelled.as_db_value())
            .await
    }

    pub async fn delete_escrow(&self, id: &str) -> Result<(), AppError> {
        self.repository.delete(id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use crate::{
        application::{errors::AppError, ports::EscrowRepository},
        domain::escrow::{Escrow, EscrowStatus, NewEscrow},
    };

    struct MockEscrowRepository {
        updated_status: Arc<Mutex<Option<String>>>,
    }

    impl MockEscrowRepository {
        fn new(updated_status: Arc<Mutex<Option<String>>>) -> Self {
            Self { updated_status }
        }
    }

    impl EscrowRepository for MockEscrowRepository {
        async fn list(&self, _buyer: Option<&str>) -> Result<Vec<Escrow>, AppError> {
            Ok(vec![sample_escrow()])
        }

        async fn create(&self, input: &NewEscrow) -> Result<Escrow, AppError> {
            Ok(Escrow {
                id: input.id.clone(),
                buyer: input.buyer.clone(),
                seller: input.seller.clone(),
                amount: input.amount,
                amount_sol: input.amount_sol,
                status: EscrowStatus::Pending,
                escrow_id: input.escrow_id,
                pda: input.pda.clone(),
                created_at: "2026-04-09T00:00:00Z".into(),
            })
        }

        async fn update_status(&self, _id: &str, status: &str) -> Result<Escrow, AppError> {
            *self.updated_status.lock().expect("status mutex poisoned") = Some(status.to_string());

            Ok(Escrow {
                status: EscrowStatus::try_from(status).expect("status should be valid"),
                ..sample_escrow()
            })
        }

        async fn delete(&self, _id: &str) -> Result<(), AppError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn release_escrow_uses_completed_status() {
        let updated_status = Arc::new(Mutex::new(None));
        let service = super::EscrowService::new(MockEscrowRepository::new(updated_status.clone()));

        let escrow = service
            .release_escrow("escrow-1")
            .await
            .expect("release should succeed");

        assert_eq!(escrow.status, EscrowStatus::Completed);
        assert_eq!(
            updated_status
                .lock()
                .expect("status mutex poisoned")
                .as_deref(),
            Some("Completed")
        );
    }

    #[tokio::test]
    async fn create_escrow_returns_created_entity() {
        let service =
            super::EscrowService::new(MockEscrowRepository::new(Arc::new(Mutex::new(None))));
        let input = sample_new_escrow();

        let escrow = service
            .create_escrow(input.clone())
            .await
            .expect("create should succeed");

        assert_eq!(escrow.id, input.id);
        assert_eq!(escrow.status, EscrowStatus::Pending);
    }

    #[test]
    fn rejects_invalid_new_escrow() {
        let input = NewEscrow {
            id: String::new(),
            buyer: String::new(),
            seller: "seller".into(),
            amount: 0,
            amount_sol: 0.0,
            escrow_id: 1,
            pda: String::new(),
        };

        assert!(input.validate().is_err());
    }

    fn sample_new_escrow() -> NewEscrow {
        NewEscrow {
            id: "escrow-1".into(),
            buyer: "buyer-1".into(),
            seller: "seller-1".into(),
            amount: 1_000_000,
            amount_sol: 0.001,
            escrow_id: 7,
            pda: "pda-1".into(),
        }
    }

    fn sample_escrow() -> Escrow {
        Escrow {
            id: "escrow-1".into(),
            buyer: "buyer-1".into(),
            seller: "seller-1".into(),
            amount: 1_000_000,
            amount_sol: 0.001,
            status: EscrowStatus::Pending,
            escrow_id: 7,
            pda: "pda-1".into(),
            created_at: "2026-04-09T00:00:00Z".into(),
        }
    }
}
