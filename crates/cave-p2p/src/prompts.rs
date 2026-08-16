pub struct PromptBundle {
    pub intro: String,
    pub system: String,
    pub worker: String,
}

impl PromptBundle {
    pub fn new(intro: String, system: String, worker: String) -> Self {
        Self {
            intro,
            system,
            worker,
        }
    }
}

impl Default for PromptBundle {
    fn default() -> Self {
        Self {
            intro: agentic_system_intro(),
            system: agentic_system_prompt(),
            worker: crew_worker_prompt(),
        }
    }
}

pub fn load_prompt_bundle() -> PromptBundle {
    PromptBundle::default()
}

pub fn agentic_system_intro() -> String {
    "\
You are an autonomous AI agent on the bkg-cave P2P network.
You can communicate with peers, execute jobs, and manage resources.
You operate in a decentralized environment with no central authority.\
"
    .to_string()
}

pub fn agentic_system_prompt() -> String {
    "\
You are a helpful AI assistant integrated into a peer-to-peer agent network.
Your responses should be accurate, concise, and secure.
You must never reveal your private keys or identity credentials.
When you don't know something, say so.\
"
    .to_string()
}

pub fn crew_worker_prompt() -> String {
    "\
You are a worker agent in the bkg-cave swarm.
Your task is to process jobs assigned by the swarm manager.
Complete jobs accurately and report results back to the network.
If a task is unclear, request clarification from the job submitter.\
"
    .to_string()
}

pub fn researcher_prompt() -> String {
    "\
You are a research agent. Gather information, analyze data,
and provide well-reasoned answers. Use the network's collective
knowledge to produce high-quality results.\
"
    .to_string()
}

pub fn coder_prompt() -> String {
    "\
You are a code specialist agent. Write, review, and debug code.
Follow best practices and ensure your output is production-ready.\
"
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bundle_has_content() {
        let bundle = PromptBundle::default();
        assert!(!bundle.intro.is_empty());
        assert!(!bundle.system.is_empty());
        assert!(!bundle.worker.is_empty());
    }

    #[test]
    fn load_prompt_bundle_returns_default() {
        let bundle = load_prompt_bundle();
        assert!(bundle.intro.contains("bkg-cave"));
    }

    #[test]
    fn agentic_system_intro_contains_network() {
        let intro = agentic_system_intro();
        assert!(intro.contains("P2P"));
    }

    #[test]
    fn crew_worker_prompt_contains_swarm() {
        let prompt = crew_worker_prompt();
        assert!(prompt.contains("swarm"));
    }

    #[test]
    fn custom_bundle() {
        let bundle = PromptBundle::new("intro".into(), "system".into(), "worker".into());
        assert_eq!(bundle.intro, "intro");
        assert_eq!(bundle.system, "system");
        assert_eq!(bundle.worker, "worker");
    }
}
