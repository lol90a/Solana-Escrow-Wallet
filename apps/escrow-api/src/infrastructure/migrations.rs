use deadpool_postgres::Pool;

pub async fn run_migrations(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = pool.get().await?;
    client
        .batch_execute(
            "CREATE TABLE IF NOT EXISTS escrows (
                id          TEXT PRIMARY KEY,
                buyer       TEXT NOT NULL,
                seller      TEXT NOT NULL,
                amount      BIGINT NOT NULL,
                amount_sol  DOUBLE PRECISION NOT NULL,
                status      TEXT NOT NULL DEFAULT 'Pending'
                                CHECK (status IN ('Pending', 'Completed', 'Cancelled')),
                escrow_id   BIGINT NOT NULL,
                pda         TEXT NOT NULL,
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_escrows_buyer ON escrows (buyer);
            CREATE INDEX IF NOT EXISTS idx_escrows_status ON escrows (status);",
        )
        .await?;

    tracing::info!("Database migrations applied");
    Ok(())
}
