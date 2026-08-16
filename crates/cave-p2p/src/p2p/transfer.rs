use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct TransferManifest {
    pub transfer_id: String,
    pub sender: String,
    pub receiver: String,
    pub total_size: u64,
    pub chunk_size: u32,
    pub checksum: String,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct TransferChunk {
    pub transfer_id: String,
    pub index: u32,
    pub data: Vec<u8>,
    pub checksum: String,
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub fn create_transfer(
    sender: &str,
    receiver: &str,
    data: &[u8],
    chunk_size: u32,
) -> (TransferManifest, Vec<TransferChunk>) {
    let transfer_id = uuid::Uuid::new_v4().to_string();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let checksum = sha256_hex(data);

    let total_chunks = if data.is_empty() {
        0
    } else {
        data.len().div_ceil(chunk_size as usize) as u64
    };

    let mut chunks = Vec::with_capacity(total_chunks as usize);
    for i in 0..total_chunks {
        let start = (i * chunk_size as u64) as usize;
        let end = std::cmp::min(start + chunk_size as usize, data.len());
        let chunk_data = data[start..end].to_vec();
        let chunk_checksum = sha256_hex(&chunk_data);
        chunks.push(TransferChunk {
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: chunk_data,
            checksum: chunk_checksum,
        });
    }

    let manifest = TransferManifest {
        transfer_id: transfer_id.clone(),
        sender: sender.to_string(),
        receiver: receiver.to_string(),
        total_size: data.len() as u64,
        chunk_size,
        checksum,
        created_at: now,
    };

    (manifest, chunks)
}

pub fn verify_transfer(manifest: &TransferManifest, chunks: &[TransferChunk]) -> bool {
    let reconstructed: Vec<u8> = chunks.iter().flat_map(|c| c.data.clone()).collect();
    sha256_hex(&reconstructed) == manifest.checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_transfer_chunks() {
        let data =
            b"Hello, World! This is a test transfer with enough data to create multiple chunks.";
        let (manifest, chunks) = create_transfer("peer1", "peer2", data, 16);
        assert_eq!(manifest.sender, "peer1");
        assert_eq!(manifest.receiver, "peer2");
        assert_eq!(manifest.total_size, data.len() as u64);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_verify_transfer_valid() {
        let data = b"Test data for transfer verification";
        let (manifest, chunks) = create_transfer("peer1", "peer2", data, 8);
        assert!(verify_transfer(&manifest, &chunks));
    }

    #[test]
    fn test_verify_transfer_corrupted() {
        let data = b"Test data for transfer verification";
        let (manifest, mut chunks) = create_transfer("peer1", "peer2", data, 8);
        if let Some(chunk) = chunks.first_mut() {
            chunk.data[0] ^= 0xFF;
        }
        assert!(!verify_transfer(&manifest, &chunks));
    }

    #[test]
    fn test_empty_transfer() {
        let data = b"";
        let (manifest, chunks) = create_transfer("peer1", "peer2", data, 16);
        assert_eq!(manifest.total_size, 0);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_single_chunk_transfer() {
        let data = b"small";
        let (manifest, chunks) = create_transfer("peer1", "peer2", data, 1024);
        assert_eq!(chunks.len(), 1);
        assert!(verify_transfer(&manifest, &chunks));
    }
}
