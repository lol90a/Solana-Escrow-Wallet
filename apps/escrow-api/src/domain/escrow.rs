use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EscrowStatus {
    Pending,
    Completed,
    Cancelled,
}

impl EscrowStatus {
    pub fn as_db_value(&self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Completed => "Completed",
            Self::Cancelled => "Cancelled",
        }
    }
}

impl TryFrom<&str> for EscrowStatus {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Pending" => Ok(Self::Pending),
            "Completed" => Ok(Self::Completed),
            "Cancelled" => Ok(Self::Cancelled),
            other => Err(format!("unsupported escrow status: {other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Escrow {
    pub id: String,
    pub buyer: String,
    pub seller: String,
    pub amount: i64,
    pub amount_sol: f64,
    pub status: EscrowStatus,
    pub escrow_id: i64,
    pub pda: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewEscrow {
    pub id: String,
    pub buyer: String,
    pub seller: String,
    pub amount: i64,
    pub amount_sol: f64,
    pub escrow_id: i64,
    pub pda: String,
}

impl NewEscrow {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("id is required".into());
        }
        if self.buyer.trim().is_empty() {
            return Err("buyer is required".into());
        }
        if self.seller.trim().is_empty() {
            return Err("seller is required".into());
        }
        if self.pda.trim().is_empty() {
            return Err("pda is required".into());
        }
        if self.amount <= 0 {
            return Err("amount must be greater than zero".into());
        }
        if self.amount_sol <= 0.0 {
            return Err("amount_sol must be greater than zero".into());
        }

        Ok(())
    }
}
