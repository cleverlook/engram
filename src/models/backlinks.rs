use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEdge {
    pub from: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    pub weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBacklinks {
    pub node: String,
    pub incoming: Vec<IncomingEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceBacklinks {
    pub namespace: String,
    pub backlinks: Vec<NodeBacklinks>,
}
