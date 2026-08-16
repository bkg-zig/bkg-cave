use serde::{Deserialize, Serialize};

pub mod transaction;

pub const TOKEN_DECIMALS: u32 = 6;
pub const MICRO_BKG: u64 = 1_000_000;

pub fn to_micro(tokens: f64) -> u64 {
    (tokens * MICRO_BKG as f64) as u64
}

pub fn from_micro(micro: u64) -> f64 {
    micro as f64 / MICRO_BKG as f64
}

pub type EscrowId = String;
pub type TransactionId = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Credit,
    Debit,
    EscrowCreation,
    EscrowRelease,
    EscrowRefund,
    Stake,
    Unstake,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Active,
    Released,
    Refunded,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: TransactionId,
    pub tx_type: TransactionType,
    pub amount: u64,
    pub timestamp: u64,
    pub reason: String,
    pub counterparty: String,
    pub job_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escrow {
    pub id: EscrowId,
    pub amount: u64,
    pub recipient: String,
    pub job_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: EscrowStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletConfig {
    pub max_spend_per_tx: u64,
    pub max_spend_per_hour: u64,
    pub max_spend_per_day: u64,
    pub reserve_balance: u64,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            max_spend_per_tx: to_micro(10.0),
            max_spend_per_hour: to_micro(50.0),
            max_spend_per_day: to_micro(200.0),
            reserve_balance: to_micro(5.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BalanceSnapshot {
    pub available: u64,
    pub in_escrow: u64,
    pub staked: u64,
    pub total: u64,
}

#[derive(Debug, Clone)]
pub struct WalletState {
    available: u64,
    in_escrow: u64,
    staked: u64,
    escrows: Vec<Escrow>,
    transactions: Vec<Transaction>,
    hourly_spend: u64,
    #[allow(dead_code)]
    daily_spend: u64,
}

pub struct Wallet {
    state: WalletState,
    config: WalletConfig,
}

impl Wallet {
    pub fn new(config: WalletConfig, initial_balance: u64) -> Self {
        Self {
            state: WalletState {
                available: initial_balance,
                in_escrow: 0,
                staked: 0,
                escrows: Vec::new(),
                transactions: Vec::new(),
                hourly_spend: 0,
                daily_spend: 0,
            },
            config,
        }
    }

    pub fn balance(&self) -> BalanceSnapshot {
        BalanceSnapshot {
            available: self.state.available,
            in_escrow: self.state.in_escrow,
            staked: self.state.staked,
            total: self.state.available + self.state.in_escrow + self.state.staked,
        }
    }

    pub fn credit(&mut self, amount: u64, reason: &str, counterparty: &str) -> TransactionId {
        self.state.available += amount;
        let tx = Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::Credit,
            amount,
            timestamp: now_secs(),
            reason: reason.to_string(),
            counterparty: counterparty.to_string(),
            job_id: None,
        };
        self.state.transactions.push(tx.clone());
        tx.id
    }

    pub fn debit(
        &mut self,
        amount: u64,
        reason: &str,
        counterparty: &str,
    ) -> Result<TransactionId, WalletError> {
        if amount > self.state.available {
            return Err(WalletError::InsufficientBalance);
        }
        if amount > self.config.max_spend_per_tx {
            return Err(WalletError::ExceedsMaxSpendPerTx);
        }

        let now = now_secs();
        let hour_ago = now - 3600;
        self.state.hourly_spend = self
            .state
            .transactions
            .iter()
            .filter(|t| {
                matches!(
                    t.tx_type,
                    TransactionType::Debit | TransactionType::EscrowCreation
                ) && t.timestamp > hour_ago
            })
            .map(|t| t.amount)
            .sum();

        if self.state.hourly_spend + amount > self.config.max_spend_per_hour {
            return Err(WalletError::HourlyLimitExceeded);
        }

        self.state.available -= amount;
        let tx = Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::Debit,
            amount,
            timestamp: now,
            reason: reason.to_string(),
            counterparty: counterparty.to_string(),
            job_id: None,
        };
        self.state.transactions.push(tx.clone());
        Ok(tx.id)
    }

    pub fn create_escrow(
        &mut self,
        amount: u64,
        recipient: &str,
        job_id: &str,
        ttl_secs: u64,
    ) -> Result<EscrowId, WalletError> {
        if amount > self.state.available {
            return Err(WalletError::InsufficientBalance);
        }

        let now = now_secs();
        self.state.available -= amount;
        self.state.in_escrow += amount;

        let escrow = Escrow {
            id: uuid_v4(),
            amount,
            recipient: recipient.to_string(),
            job_id: job_id.to_string(),
            created_at: now,
            expires_at: now + ttl_secs,
            status: EscrowStatus::Active,
        };
        let id = escrow.id.clone();
        self.state.escrows.push(escrow);

        self.state.transactions.push(Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::EscrowCreation,
            amount,
            timestamp: now,
            reason: format!("escrow for job {job_id}"),
            counterparty: recipient.to_string(),
            job_id: Some(job_id.to_string()),
        });

        Ok(id)
    }

    pub fn release_escrow(&mut self, escrow_id: &str) -> Result<u64, WalletError> {
        let escrow = self
            .state
            .escrows
            .iter_mut()
            .find(|e| e.id == escrow_id && e.status == EscrowStatus::Active)
            .ok_or(WalletError::EscrowNotFound)?;

        escrow.status = EscrowStatus::Released;
        self.state.in_escrow -= escrow.amount;

        self.state.transactions.push(Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::EscrowRelease,
            amount: escrow.amount,
            timestamp: now_secs(),
            reason: format!("release escrow {escrow_id}"),
            counterparty: escrow.recipient.clone(),
            job_id: Some(escrow.job_id.clone()),
        });

        Ok(escrow.amount)
    }

    pub fn refund_escrow(&mut self, escrow_id: &str) -> Result<u64, WalletError> {
        let escrow = self
            .state
            .escrows
            .iter_mut()
            .find(|e| e.id == escrow_id && e.status == EscrowStatus::Active)
            .ok_or(WalletError::EscrowNotFound)?;

        escrow.status = EscrowStatus::Refunded;
        self.state.in_escrow -= escrow.amount;
        self.state.available += escrow.amount;

        self.state.transactions.push(Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::EscrowRefund,
            amount: escrow.amount,
            timestamp: now_secs(),
            reason: format!("refund escrow {escrow_id}"),
            counterparty: escrow.recipient.clone(),
            job_id: Some(escrow.job_id.clone()),
        });

        Ok(escrow.amount)
    }

    pub fn stake(&mut self, amount: u64) -> Result<(), WalletError> {
        if amount > self.state.available {
            return Err(WalletError::InsufficientBalance);
        }
        self.state.available -= amount;
        self.state.staked += amount;

        self.state.transactions.push(Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::Stake,
            amount,
            timestamp: now_secs(),
            reason: "stake".into(),
            counterparty: String::new(),
            job_id: None,
        });

        Ok(())
    }

    pub fn unstake(&mut self, amount: u64) -> Result<(), WalletError> {
        if amount > self.state.staked {
            return Err(WalletError::InsufficientStake);
        }
        self.state.staked -= amount;
        self.state.available += amount;

        self.state.transactions.push(Transaction {
            id: uuid_v4(),
            tx_type: TransactionType::Unstake,
            amount,
            timestamp: now_secs(),
            reason: "unstake".into(),
            counterparty: String::new(),
            job_id: None,
        });

        Ok(())
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.state.transactions
    }

    pub fn active_escrows(&self) -> Vec<&Escrow> {
        self.state
            .escrows
            .iter()
            .filter(|e| e.status == EscrowStatus::Active)
            .collect()
    }

    pub fn sweep_expired_escrows(&mut self) -> u64 {
        let now = now_secs();
        let mut swept = 0;
        self.state.escrows.iter_mut().for_each(|e| {
            if e.status == EscrowStatus::Active && e.expires_at <= now {
                e.status = EscrowStatus::Expired;
                self.state.in_escrow -= e.amount;
                self.state.available += e.amount;
                swept += e.amount;
            }
        });
        swept
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("wallet-{nanos}")
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("insufficient balance")]
    InsufficientBalance,
    #[error("insufficient stake")]
    InsufficientStake,
    #[error("exceeds max spend per transaction")]
    ExceedsMaxSpendPerTx,
    #[error("hourly spending limit exceeded")]
    HourlyLimitExceeded,
    #[error("daily spending limit exceeded")]
    DailyLimitExceeded,
    #[error("escrow not found or not active")]
    EscrowNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credit_and_balance() {
        let mut wallet = Wallet::new(WalletConfig::default(), 0);
        wallet.credit(1000, "test", "alice");
        let bal = wallet.balance();
        assert_eq!(bal.available, 1000);
        assert_eq!(bal.total, 1000);
    }

    #[test]
    fn test_debit_insufficient_balance() {
        let mut wallet = Wallet::new(WalletConfig::default(), 100);
        let result = wallet.debit(200, "test", "bob");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WalletError::InsufficientBalance
        ));
    }

    #[test]
    fn test_escrow_lifecycle() {
        let mut wallet = Wallet::new(
            WalletConfig {
                max_spend_per_tx: 10_000,
                ..Default::default()
            },
            5000,
        );

        let escrow_id = wallet
            .create_escrow(1000, "worker1", "job-1", 3600)
            .unwrap();
        assert_eq!(wallet.balance().available, 4000);
        assert_eq!(wallet.balance().in_escrow, 1000);

        wallet.release_escrow(&escrow_id).unwrap();
        assert_eq!(wallet.balance().in_escrow, 0);
        assert_eq!(wallet.balance().available, 4000);
    }

    #[test]
    fn test_stake_and_unstake() {
        let mut wallet = Wallet::new(WalletConfig::default(), 5000);
        wallet.stake(2000).unwrap();
        assert_eq!(wallet.balance().staked, 2000);
        assert_eq!(wallet.balance().available, 3000);

        wallet.unstake(1000).unwrap();
        assert_eq!(wallet.balance().staked, 1000);
        assert_eq!(wallet.balance().available, 4000);
    }

    #[test]
    fn test_escrow_refund() {
        let mut wallet = Wallet::new(
            WalletConfig {
                max_spend_per_tx: 10_000,
                ..Default::default()
            },
            5000,
        );

        let escrow_id = wallet
            .create_escrow(1000, "worker1", "job-1", 3600)
            .unwrap();
        wallet.refund_escrow(&escrow_id).unwrap();
        assert_eq!(wallet.balance().available, 5000);
        assert_eq!(wallet.balance().in_escrow, 0);
    }

    #[test]
    fn test_sweep_expired_escrows() {
        let mut wallet = Wallet::new(
            WalletConfig {
                max_spend_per_tx: 10_000,
                ..Default::default()
            },
            5000,
        );

        wallet.create_escrow(1000, "worker1", "job-1", 0).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));

        let swept = wallet.sweep_expired_escrows();
        assert_eq!(swept, 1000);
        assert_eq!(wallet.balance().available, 5000);
    }

    #[test]
    fn test_to_from_micro() {
        assert_eq!(to_micro(1.0), 1_000_000);
        assert_eq!(from_micro(1_000_000), 1.0);
        assert_eq!(to_micro(0.5), 500_000);
        assert_eq!(from_micro(500_000), 0.5);
    }
}
