use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLibraryEntry {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub capabilities: Vec<String>,
}

pub fn builtin_entries() -> Vec<AgentLibraryEntry> {
    vec![
        AgentLibraryEntry {
            name: "deep-researcher".into(),
            description: "Deep research agent that gathers and synthesizes information from multiple sources.".into(),
            system_prompt: "You are a research specialist. Gather comprehensive information and synthesize findings.".into(),
            capabilities: vec![
                "web_search".into(),
                "data_analysis".into(),
                "summarization".into(),
            ],
        },
        AgentLibraryEntry {
            name: "code-reviewer".into(),
            description: "Code review agent that analyzes code for bugs, security issues, and style violations.".into(),
            system_prompt: "You are a code review specialist. Analyze code for correctness, security, and style.".into(),
            capabilities: vec![
                "code_analysis".into(),
                "security_audit".into(),
                "style_checking".into(),
            ],
        },
        AgentLibraryEntry {
            name: "creative-writer".into(),
            description: "Creative writing agent for generating prose, poetry, and marketing copy.".into(),
            system_prompt: "You are a creative writing specialist. Generate engaging, original content.".into(),
            capabilities: vec![
                "content_generation".into(),
                "editing".into(),
                "style_adaptation".into(),
            ],
        },
        AgentLibraryEntry {
            name: "data-analyst".into(),
            description: "Data analysis agent that processes datasets and generates insights.".into(),
            system_prompt: "You are a data analysis specialist. Process data and extract actionable insights.".into(),
            capabilities: vec![
                "data_processing".into(),
                "statistical_analysis".into(),
                "visualization".into(),
            ],
        },
    ]
}

pub fn save_user_entries() -> std::io::Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_entries_has_four() {
        let entries = builtin_entries();
        assert_eq!(entries.len(), 4);
    }

    #[test]
    fn deep_researcher_present() {
        let entries = builtin_entries();
        let dr = entries
            .iter()
            .find(|e| e.name == "deep-researcher")
            .unwrap();
        assert!(dr.description.contains("research"));
        assert!(dr.capabilities.contains(&"web_search".to_string()));
    }

    #[test]
    fn code_reviewer_present() {
        let entries = builtin_entries();
        let cr = entries.iter().find(|e| e.name == "code-reviewer").unwrap();
        assert!(cr.description.contains("Code review"));
    }

    #[test]
    fn creative_writer_present() {
        let entries = builtin_entries();
        let cw = entries
            .iter()
            .find(|e| e.name == "creative-writer")
            .unwrap();
        assert!(cw.description.contains("writing"));
    }

    #[test]
    fn data_analyst_present() {
        let entries = builtin_entries();
        let da = entries.iter().find(|e| e.name == "data-analyst").unwrap();
        assert!(da.description.contains("Data analysis"));
    }

    #[test]
    fn each_entry_has_capabilities() {
        for entry in builtin_entries() {
            assert!(!entry.capabilities.is_empty());
        }
    }

    #[test]
    fn save_user_entries_ok() {
        assert!(save_user_entries().is_ok());
    }
}
