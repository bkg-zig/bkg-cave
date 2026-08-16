use serde::{Deserialize, Serialize};

pub const TOOL_MANIFEST_TOPIC: &str = "bkg-p2p/tools/v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifest {
    pub name: String,
    pub description: String,
    pub category: String,
    pub parameters: serde_json::Value,
    pub local_only: bool,
    pub tags: Vec<String>,
    pub peer_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManifestAdvert {
    pub peer_id: String,
    pub tools: Vec<ToolManifest>,
}

impl ToolManifestAdvert {
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            tools: Vec::new(),
        }
    }

    pub fn add_tool(&mut self, tool: ToolManifest) {
        self.tools.push(tool);
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn has_tool(&self, name: &str) -> bool {
        self.tools.iter().any(|t| t.name == name)
    }

    pub fn tools_by_category(&self, category: &str) -> Vec<&ToolManifest> {
        self.tools
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }
}

impl ToolManifest {
    pub fn new(
        name: String,
        description: String,
        category: String,
        parameters: serde_json::Value,
        local_only: bool,
        tags: Vec<String>,
        peer_id: String,
    ) -> Self {
        Self {
            name,
            description,
            category,
            parameters,
            local_only,
            tags,
            peer_id,
        }
    }

    pub fn can_execute_remotely(&self) -> bool {
        !self.local_only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_manifest_creation() {
        let tool = ToolManifest::new(
            "web_search".into(),
            "Search the web".into(),
            "utility".into(),
            serde_json::json!({"query": {"type": "string"}}),
            false,
            vec!["search".into(), "web".into()],
            "peer1".into(),
        );
        assert_eq!(tool.name, "web_search");
        assert!(tool.can_execute_remotely());
    }

    #[test]
    fn test_tool_manifest_advert() {
        let mut advert = ToolManifestAdvert::new("peer1".into());
        assert_eq!(advert.tool_count(), 0);

        let tool = ToolManifest::new(
            "web_search".into(),
            "Search the web".into(),
            "utility".into(),
            serde_json::Value::Null,
            false,
            vec![],
            "peer1".into(),
        );
        advert.add_tool(tool);
        assert_eq!(advert.tool_count(), 1);
        assert!(advert.has_tool("web_search"));
    }

    #[test]
    fn test_tool_manifest_local_only() {
        let local = ToolManifest::new(
            "file_read".into(),
            "Read local files".into(),
            "system".into(),
            serde_json::Value::Null,
            true,
            vec![],
            "peer1".into(),
        );
        assert!(!local.can_execute_remotely());
    }
}
