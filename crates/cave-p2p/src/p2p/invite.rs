#[derive(Debug, Clone)]
pub struct Invite {
    pub inviter_peer_id: String,
    pub invitee_peer_id: Option<String>,
    pub token: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub permissions: Vec<String>,
}

pub fn generate_invite(inviter: &str) -> Invite {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Invite {
        inviter_peer_id: inviter.to_string(),
        invitee_peer_id: None,
        token: uuid::Uuid::new_v4().to_string(),
        created_at: now,
        expires_at: now + 86400,
        permissions: vec!["read".to_string()],
    }
}

pub fn validate_invite(invite: &Invite) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now < invite.expires_at
}

pub fn redeem_invite(invite: &mut Invite, invitee: &str) -> bool {
    if !validate_invite(invite) {
        return false;
    }
    if invite.invitee_peer_id.is_some() {
        return false;
    }
    invite.invitee_peer_id = Some(invitee.to_string());
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_invite() {
        let invite = generate_invite("peer1");
        assert_eq!(invite.inviter_peer_id, "peer1");
        assert!(invite.invitee_peer_id.is_none());
        assert!(!invite.token.is_empty());
        assert!(invite.expires_at > invite.created_at);
    }

    #[test]
    fn test_validate_invite() {
        let invite = generate_invite("peer1");
        assert!(validate_invite(&invite));
    }

    #[test]
    fn test_redeem_invite() {
        let mut invite = generate_invite("peer1");
        assert!(redeem_invite(&mut invite, "peer2"));
        assert_eq!(invite.invitee_peer_id, Some("peer2".to_string()));
    }

    #[test]
    fn test_redeem_twice_fails() {
        let mut invite = generate_invite("peer1");
        assert!(redeem_invite(&mut invite, "peer2"));
        assert!(!redeem_invite(&mut invite, "peer3"));
    }

    #[test]
    fn test_expired_invite_rejected() {
        let mut invite = generate_invite("peer1");
        invite.expires_at = 0;
        assert!(!validate_invite(&invite));
        assert!(!redeem_invite(&mut invite, "peer2"));
    }
}
