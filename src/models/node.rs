use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Active,
    Dirty,
    Stale,
    Deprecated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Uses,
    DependsOn,
    Implements,
    Rationale,
    Related,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: EdgeType,
    pub weight: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub content: String,
    pub weight: u8,
    pub status: NodeStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
    pub created: DateTime<Utc>,
    pub touched: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data_lake: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edges: Vec<Edge>,
}
