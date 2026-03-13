use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "engram", about = "Persistent structured memory for AI agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize .engram/ in current directory
    Init,

    /// Node operations
    Node {
        #[command(subcommand)]
        action: NodeAction,
    },

    /// Full-text search across all nodes
    Search {
        /// Search query
        query: String,
    },

    /// Traverse graph from a node
    Traverse {
        /// Entry node id
        id: String,

        /// Max traversal depth
        #[arg(long, default_value_t = 5)]
        depth: u32,

        /// Minimum edge weight to follow
        #[arg(long, default_value_t = 0)]
        min_weight: u8,

        /// Token budget
        #[arg(long, default_value_t = 4000)]
        budget: usize,
    },

    /// Show all nodes pointing to this node
    Backlinks {
        /// Node id
        id: String,
    },

    /// Show dirty/stale nodes, apply weight decay
    Status,

    /// Check graph integrity
    Check,

    /// Rebuild _index.yaml, _backlinks.yaml, and SQLite
    RebuildIndex,
}

#[derive(Subcommand)]
pub enum NodeAction {
    /// Print a node
    Get {
        /// Node id
        id: String,
    },
    /// Create a new node (stdin or $EDITOR)
    Create,
    /// Update an existing node
    Update {
        /// Node id
        id: String,
    },
    /// Mark a node as deprecated
    Deprecate {
        /// Node id
        id: String,
    },
}
