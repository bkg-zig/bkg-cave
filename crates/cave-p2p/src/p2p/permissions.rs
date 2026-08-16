#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    Read,
    Write,
    Execute,
    Admin,
    Custom(String),
}

impl Capability {
    pub fn as_str(&self) -> &str {
        match self {
            Capability::Read => "read",
            Capability::Write => "write",
            Capability::Execute => "execute",
            Capability::Admin => "admin",
            Capability::Custom(s) => s.as_str(),
        }
    }
}

impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "read" => Capability::Read,
            "write" => Capability::Write,
            "execute" => Capability::Execute,
            "admin" => Capability::Admin,
            _ => Capability::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PermissionSet {
    pub peer_id: String,
    pub capabilities: Vec<Capability>,
    pub granted_at: u64,
    pub expires_at: u64,
}

pub fn can_access(permissions: &PermissionSet, capability: &str) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now >= permissions.expires_at {
        return false;
    }
    permissions
        .capabilities
        .iter()
        .any(|c| c.as_str() == capability)
}

pub fn grant_permission(peer_id: &str, capability: &str) -> PermissionSet {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    PermissionSet {
        peer_id: peer_id.to_string(),
        capabilities: vec![Capability::from(capability)],
        granted_at: now,
        expires_at: now + 86400,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_access_granted() {
        let permissions = grant_permission("peer1", "read");
        assert!(can_access(&permissions, "read"));
    }

    #[test]
    fn test_can_access_missing() {
        let permissions = grant_permission("peer1", "read");
        assert!(!can_access(&permissions, "write"));
    }

    #[test]
    fn test_can_access_admin_granted() {
        let permissions = grant_permission("peer1", "admin");
        assert!(can_access(&permissions, "admin"));
    }

    #[test]
    fn test_expired_permission_rejected() {
        let mut permissions = grant_permission("peer1", "read");
        permissions.expires_at = 0;
        assert!(!can_access(&permissions, "read"));
    }

    #[test]
    fn test_capability_from_str() {
        let cap: Capability = "read".into();
        assert_eq!(cap, Capability::Read);
        let cap: Capability = "Write".into();
        assert_eq!(cap, Capability::Write);
        let cap: Capability = "custom_perm".into();
        assert_eq!(cap, Capability::Custom("custom_perm".to_string()));
    }
}
