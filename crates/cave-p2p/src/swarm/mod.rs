use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AgentStatus {
    Idle,
    Working,
    #[default]
    Offline,
}

#[derive(Debug, Clone)]
pub struct SwarmAgent {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
}

impl SwarmAgent {
    pub fn new(id: String, name: String, role: String) -> Self {
        Self {
            id,
            name,
            role,
            status: AgentStatus::Offline,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SwarmEvent {
    AgentJoined {
        agent_id: String,
        name: String,
        role: String,
    },
    AgentLeft {
        agent_id: String,
    },
    JobAssigned {
        agent_id: String,
        job_id: String,
    },
    JobCompleted {
        agent_id: String,
        job_id: String,
    },
    JobFailed {
        agent_id: String,
        job_id: String,
        error: String,
    },
}

pub struct SwarmManager {
    agents: HashMap<String, SwarmAgent>,
}

impl SwarmManager {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn register_local_agent(&mut self, id: String, name: String, role: String) -> &SwarmAgent {
        let status = AgentStatus::Idle;
        let agent = SwarmAgent {
            id: id.clone(),
            name,
            role,
            status,
        };
        self.agents.entry(id.clone()).or_insert(agent);
        self.agents.get(&id).unwrap()
    }

    pub fn handle_network_event(&mut self, event: SwarmEvent) {
        match event {
            SwarmEvent::AgentJoined {
                agent_id,
                name,
                role,
            } => {
                self.agents
                    .entry(agent_id.clone())
                    .or_insert(SwarmAgent::new(agent_id, name, role));
            }
            SwarmEvent::AgentLeft { agent_id } => {
                if let Some(agent) = self.agents.get_mut(&agent_id) {
                    agent.status = AgentStatus::Offline;
                }
            }
            SwarmEvent::JobAssigned { agent_id, .. } => {
                if let Some(agent) = self.agents.get_mut(&agent_id) {
                    agent.status = AgentStatus::Working;
                }
            }
            SwarmEvent::JobCompleted { agent_id, .. } | SwarmEvent::JobFailed { agent_id, .. } => {
                if let Some(agent) = self.agents.get_mut(&agent_id) {
                    agent.status = AgentStatus::Idle;
                }
            }
        }
    }

    pub fn record_job_action(&mut self, agent_id: &str, action: &str) {
        match action {
            "assign" => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.status = AgentStatus::Working;
                }
            }
            "complete" | "fail" => {
                if let Some(agent) = self.agents.get_mut(agent_id) {
                    agent.status = AgentStatus::Idle;
                }
            }
            _ => {}
        }
    }

    pub fn any_local_agent_id(&self) -> Option<&str> {
        self.agents
            .iter()
            .find(|(_, a)| a.status != AgentStatus::Offline)
            .map(|(id, _)| id.as_str())
    }

    pub fn get_agent(&self, id: &str) -> Option<&SwarmAgent> {
        self.agents.get(id)
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn agents(&self) -> &HashMap<String, SwarmAgent> {
        &self.agents
    }
}

impl Default for SwarmManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_swarm_is_empty() {
        let sm = SwarmManager::new();
        assert_eq!(sm.agent_count(), 0);
    }

    #[test]
    fn register_local_agent() {
        let mut sm = SwarmManager::new();
        sm.register_local_agent("agent-1".into(), "alpha".into(), "worker".into());
        assert_eq!(sm.agent_count(), 1);
        let agent = sm.get_agent("agent-1").unwrap();
        assert_eq!(agent.name, "alpha");
        assert_eq!(agent.status, AgentStatus::Idle);
    }

    #[test]
    fn handle_join_event() {
        let mut sm = SwarmManager::new();
        sm.handle_network_event(SwarmEvent::AgentJoined {
            agent_id: "remote-1".into(),
            name: "bot".into(),
            role: "worker".into(),
        });
        assert_eq!(sm.agent_count(), 1);
    }

    #[test]
    fn job_assignment_changes_status() {
        let mut sm = SwarmManager::new();
        sm.register_local_agent("a1".into(), "alice".into(), "worker".into());
        sm.record_job_action("a1", "assign");
        assert_eq!(sm.get_agent("a1").unwrap().status, AgentStatus::Working);
        sm.record_job_action("a1", "complete");
        assert_eq!(sm.get_agent("a1").unwrap().status, AgentStatus::Idle);
    }

    #[test]
    fn any_local_agent_returns_some() {
        let mut sm = SwarmManager::new();
        sm.register_local_agent("l1".into(), "local".into(), "worker".into());
        assert!(sm.any_local_agent_id().is_some());
    }

    #[test]
    fn any_local_agent_returns_none_when_offline() {
        let mut sm = SwarmManager::new();
        sm.register_local_agent("off".into(), "offline".into(), "worker".into());
        if let Some(agent) = sm.agents.get_mut("off") {
            agent.status = AgentStatus::Offline;
        }
        assert!(sm.any_local_agent_id().is_none());
    }

    #[test]
    fn network_event_agent_left() {
        let mut sm = SwarmManager::new();
        sm.register_local_agent("a1".into(), "alice".into(), "worker".into());
        sm.handle_network_event(SwarmEvent::AgentLeft {
            agent_id: "a1".into(),
        });
        assert_eq!(sm.get_agent("a1").unwrap().status, AgentStatus::Offline);
    }
}
