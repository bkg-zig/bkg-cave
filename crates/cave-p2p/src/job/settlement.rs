use crate::wallet::Wallet;
use crate::Result;

/// Settlement status for a job payment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettlementStatus {
    /// Payment is pending.
    Pending,
    /// Payment has been released to the worker.
    Released,
    /// Payment was refunded to the requester.
    Refunded,
    /// Payment is in escrow.
    Escrowed,
}

/// A settlement record for a completed job.
#[derive(Debug, Clone)]
pub struct Settlement {
    pub job_id: String,
    pub worker: String,
    pub requester: String,
    pub amount: u64,
    pub status: SettlementStatus,
    pub escrow_id: Option<String>,
    pub created_at: u64,
    pub settled_at: Option<u64>,
}

/// Handles job payment settlement using the wallet system.
pub struct SettlementEngine {
    wallet: std::sync::Arc<std::sync::Mutex<Wallet>>,
    settlements: Vec<Settlement>,
}

impl SettlementEngine {
    /// Create a new settlement engine.
    pub fn new(wallet: std::sync::Arc<std::sync::Mutex<Wallet>>) -> Self {
        Self {
            wallet,
            settlements: Vec::new(),
        }
    }

    /// Settle a job payment by creating an escrow.
    /// The escrow is NOT automatically released; call `release_settlement` or `refund_job` later.
    pub fn settle_job(
        &mut self,
        job_id: impl Into<String>,
        worker: impl Into<String>,
        requester: impl Into<String>,
        amount: u64,
    ) -> Result<Settlement> {
        let job_id = job_id.into();
        let worker = worker.into();
        let requester = requester.into();

        // Create escrow from requester to worker
        let mut wallet = self.wallet.lock().map_err(|e| crate::Error::Other(e.to_string()))?;
        let escrow_id = wallet
            .create_escrow(amount, &worker, &job_id, 86400)
            .map_err(|e| crate::Error::Other(e.to_string()))?;
        drop(wallet);

        let settlement = Settlement {
            job_id: job_id.clone(),
            worker: worker.clone(),
            requester: requester.into(),
            amount,
            status: SettlementStatus::Escrowed,
            escrow_id: Some(escrow_id.clone()),
            created_at: now_secs(),
            settled_at: None,
        };
        self.settlements.push(settlement.clone());

        Ok(settlement)
    }

    /// Release a settled escrow to the worker.
    pub fn release_settlement(&mut self, job_id: &str) -> Result<()> {
        let settlement = self
            .settlements
            .iter_mut()
            .find(|s| s.job_id == job_id)
            .ok_or_else(|| crate::Error::Other(format!("settlement for job {job_id} not found")))?;

        if settlement.status != SettlementStatus::Escrowed {
            return Err(crate::Error::Other(format!(
                "job {job_id} is not in escrow (status: {:?})",
                settlement.status
            )));
        }

        let escrow_id = settlement
            .escrow_id
            .as_ref()
            .ok_or_else(|| crate::Error::Other("no escrow id".into()))?;

        let mut wallet = self.wallet.lock().map_err(|e| crate::Error::Other(e.to_string()))?;
        wallet
            .release_escrow(escrow_id)
            .map_err(|e| crate::Error::Other(e.to_string()))?;
        drop(wallet);

        settlement.status = SettlementStatus::Released;
        settlement.settled_at = Some(now_secs());

        Ok(())
    }

    /// Refund a job payment to the requester.
    pub fn refund_job(&mut self, job_id: &str) -> Result<u64> {
        let settlement = self
            .settlements
            .iter_mut()
            .find(|s| s.job_id == job_id)
            .ok_or_else(|| crate::Error::Other(format!("settlement for job {job_id} not found")))?;

        if settlement.status != SettlementStatus::Escrowed {
            return Err(crate::Error::Other(format!(
                "job {job_id} is not in escrow (status: {:?})",
                settlement.status
            )));
        }

        let escrow_id = settlement
            .escrow_id
            .as_ref()
            .ok_or_else(|| crate::Error::Other("no escrow id".into()))?;

        let mut wallet = self.wallet.lock().map_err(|e| crate::Error::Other(e.to_string()))?;
        let refunded = wallet
            .refund_escrow(escrow_id)
            .map_err(|e| crate::Error::Other(e.to_string()))?;
        drop(wallet);

        settlement.status = SettlementStatus::Refunded;
        settlement.settled_at = Some(now_secs());

        Ok(refunded)
    }

    /// Get a settlement by job ID.
    pub fn get_settlement(&self, job_id: &str) -> Option<&Settlement> {
        self.settlements.iter().find(|s| s.job_id == job_id)
    }

    /// Get all settlements.
    pub fn all_settlements(&self) -> &[Settlement] {
        &self.settlements
    }

    /// Get total amount settled.
    pub fn total_settled(&self) -> u64 {
        self.settlements
            .iter()
            .filter(|s| s.status == SettlementStatus::Released)
            .map(|s| s.amount)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_wallet() -> std::sync::Arc<std::sync::Mutex<Wallet>> {
        std::sync::Arc::new(std::sync::Mutex::new(Wallet::new(
            crate::wallet::WalletConfig::default(),
            10_000,
        )))
    }

    #[test]
    fn test_settle_job() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet.clone());

        let settlement = engine
            .settle_job("job-1", "worker-1", "alice", 1000)
            .unwrap();

        assert_eq!(settlement.job_id, "job-1");
        assert_eq!(settlement.worker, "worker-1");
        assert_eq!(settlement.amount, 1000);
        assert_eq!(settlement.status, SettlementStatus::Escrowed);
        assert!(settlement.escrow_id.is_some());
    }

    #[test]
    fn test_release_settlement() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet.clone());

        engine.settle_job("job-1", "worker-1", "alice", 1000).unwrap();
        engine.release_settlement("job-1").unwrap();

        let settlement = engine.get_settlement("job-1").unwrap();
        assert_eq!(settlement.status, SettlementStatus::Released);
        assert!(settlement.settled_at.is_some());
    }

    #[test]
    fn test_refund_job() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet.clone());

        engine.settle_job("job-1", "worker-1", "alice", 1000).unwrap();
        let refunded = engine.refund_job("job-1").unwrap();
        assert_eq!(refunded, 1000);

        let settlement = engine.get_settlement("job-1").unwrap();
        assert_eq!(settlement.status, SettlementStatus::Refunded);
    }

    #[test]
    fn test_refund_nonexistent_job_fails() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet);
        let result = engine.refund_job("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_total_settled() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet);

        engine.settle_job("job-1", "worker-1", "alice", 1000).unwrap();
        engine.settle_job("job-2", "worker-2", "bob", 2000).unwrap();
        engine.release_settlement("job-1").unwrap();
        engine.refund_job("job-2").unwrap();

        assert_eq!(engine.total_settled(), 1000);
    }

    #[test]
    fn test_settle_reduces_available_balance() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet.clone());

        let initial = wallet.lock().unwrap().balance().available;
        engine.settle_job("job-1", "worker-1", "alice", 1000).unwrap();

        let after = wallet.lock().unwrap().balance().available;
        assert_eq!(after, initial - 1000);
    }

    #[test]
    fn test_refund_restores_balance() {
        let wallet = make_wallet();
        let mut engine = SettlementEngine::new(wallet.clone());

        let initial = wallet.lock().unwrap().balance().available;
        engine.settle_job("job-1", "worker-1", "alice", 1000).unwrap();
        engine.refund_job("job-1").unwrap();

        let after = wallet.lock().unwrap().balance().available;
        assert_eq!(after, initial);
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
