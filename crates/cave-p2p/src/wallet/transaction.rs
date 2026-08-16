use serde::{Deserialize, Serialize};
use crate::identity::NodeIdentity;

/// Signed transaction envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub transaction: crate::wallet::Transaction,
    pub signature: Vec<u8>,
    pub public_key: [u8; 32],
}

impl SignedTransaction {
    /// Sign a transaction with the given identity.
    pub fn sign(transaction: crate::wallet::Transaction, identity: &NodeIdentity) -> Self {
        let payload = serde_json::to_vec(&transaction).unwrap_or_default();
        let signature = identity.sign(&payload);
        let public_key = identity.public_key_bytes();
        Self {
            transaction,
            signature: signature.to_vec(),
            public_key,
        }
    }

    /// Verify the transaction signature.
    pub fn verify(&self) -> bool {
        let payload = match serde_json::to_vec(&self.transaction) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let sig_array: [u8; 64] = match self.signature.as_slice().try_into() {
            Ok(arr) => arr,
            Err(_) => return false,
        };
        self.transaction
            .id
            .starts_with("wallet-")
            && NodeIdentity::verify_with_key(&payload, &sig_array, &self.public_key).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_and_verify_transaction() {
        let identity = crate::identity::NodeIdentity::generate().unwrap();
        let tx = crate::wallet::Transaction {
            id: "wallet-1234".into(),
            tx_type: crate::wallet::TransactionType::Credit,
            amount: 1000,
            timestamp: 1000,
            reason: "test".into(),
            counterparty: "alice".into(),
            job_id: None,
        };

        let signed = SignedTransaction::sign(tx.clone(), &identity);
        assert!(signed.verify());
        assert_eq!(signed.public_key, identity.public_key_bytes());
    }

    #[test]
    fn test_verify_rejects_tampered_transaction() {
        let identity = crate::identity::NodeIdentity::generate().unwrap();
        let mut tx = crate::wallet::Transaction {
            id: "wallet-1234".into(),
            tx_type: crate::wallet::TransactionType::Credit,
            amount: 1000,
            timestamp: 1000,
            reason: "test".into(),
            counterparty: "alice".into(),
            job_id: None,
        };

        let signed = SignedTransaction::sign(tx.clone(), &identity);
        tx.amount = 9999;
        let tampered = SignedTransaction {
            transaction: tx,
            signature: signed.signature.clone(),
            public_key: signed.public_key,
        };
        assert!(!tampered.verify());
    }

    #[test]
    fn test_verify_rejects_wrong_key() {
        let identity1 = crate::identity::NodeIdentity::generate().unwrap();
        let identity2 = crate::identity::NodeIdentity::generate().unwrap();
        let tx = crate::wallet::Transaction {
            id: "wallet-1234".into(),
            tx_type: crate::wallet::TransactionType::Credit,
            amount: 1000,
            timestamp: 1000,
            reason: "test".into(),
            counterparty: "alice".into(),
            job_id: None,
        };

        let signed = SignedTransaction::sign(tx, &identity1);
        let wrong_key = identity2.public_key_bytes();
        let wrong = SignedTransaction {
            transaction: signed.transaction,
            signature: signed.signature,
            public_key: wrong_key,
        };
        assert!(!wrong.verify());
    }
}
