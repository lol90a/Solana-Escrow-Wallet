use deadpool_postgres::Pool;
use tokio_postgres::Row;

use crate::{
    application::{errors::AppError, ports::EscrowRepository},
    domain::escrow::{Escrow, EscrowStatus, NewEscrow},
};

pub struct PostgresEscrowRepository {
    pool: Pool,
}

impl PostgresEscrowRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl EscrowRepository for PostgresEscrowRepository {
    async fn list(&self, buyer: Option<&str>) -> Result<Vec<Escrow>, AppError> {
        let client = self.pool.get().await.map_err(map_pool_error)?;
        let rows = match buyer {
            Some(buyer) => {
                client
                    .query(
                        "SELECT id, buyer, seller, amount, amount_sol, status, escrow_id, pda,
                                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
                         FROM escrows
                         WHERE buyer = $1
                         ORDER BY created_at DESC",
                        &[&buyer],
                    )
                    .await
            }
            None => {
                client
                    .query(
                        "SELECT id, buyer, seller, amount, amount_sol, status, escrow_id, pda,
                                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at
                         FROM escrows
                         ORDER BY created_at DESC
                         LIMIT 100",
                        &[],
                    )
                    .await
            }
        }
        .map_err(map_query_error)?;

        rows.iter().map(map_row).collect()
    }

    async fn create(&self, input: &NewEscrow) -> Result<Escrow, AppError> {
        let client = self.pool.get().await.map_err(map_pool_error)?;
        let row = client
            .query_one(
                "INSERT INTO escrows (id, buyer, seller, amount, amount_sol, status, escrow_id, pda)
                 VALUES ($1, $2, $3, $4, $5, 'Pending', $6, $7)
                 RETURNING id, buyer, seller, amount, amount_sol, status, escrow_id, pda,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at",
                &[
                    &input.id,
                    &input.buyer,
                    &input.seller,
                    &input.amount,
                    &input.amount_sol,
                    &input.escrow_id,
                    &input.pda,
                ],
            )
            .await
            .map_err(map_query_error)?;

        map_row(&row)
    }

    async fn update_status(&self, id: &str, status: &str) -> Result<Escrow, AppError> {
        let client = self.pool.get().await.map_err(map_pool_error)?;
        let row = client
            .query_opt(
                "UPDATE escrows
                 SET status = $2
                 WHERE id = $1 AND status = 'Pending'
                 RETURNING id, buyer, seller, amount, amount_sol, status, escrow_id, pda,
                           to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS created_at",
                &[&id, &status],
            )
            .await
            .map_err(map_query_error)?;

        match row {
            Some(row) => map_row(&row),
            None => Err(AppError::NotFound("escrow not found or not pending".into())),
        }
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let client = self.pool.get().await.map_err(map_pool_error)?;
        let deleted_rows = client
            .execute("DELETE FROM escrows WHERE id = $1", &[&id])
            .await
            .map_err(map_query_error)?;

        if deleted_rows == 0 {
            return Err(AppError::NotFound("escrow not found".into()));
        }

        Ok(())
    }
}

fn map_row(row: &Row) -> Result<Escrow, AppError> {
    let status_raw: String = row.get("status");
    let status = EscrowStatus::try_from(status_raw.as_str()).map_err(AppError::Infrastructure)?;

    Ok(Escrow {
        id: row.get("id"),
        buyer: row.get("buyer"),
        seller: row.get("seller"),
        amount: row.get("amount"),
        amount_sol: row.get("amount_sol"),
        status,
        escrow_id: row.get("escrow_id"),
        pda: row.get("pda"),
        created_at: row.get("created_at"),
    })
}

fn map_pool_error(error: deadpool_postgres::PoolError) -> AppError {
    tracing::error!("database pool error: {error}");
    AppError::Infrastructure("database unavailable".into())
}

fn map_query_error(error: tokio_postgres::Error) -> AppError {
    tracing::error!("database query error: {error}");
    AppError::Infrastructure("database operation failed".into())
}
