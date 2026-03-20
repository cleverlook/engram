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

    /// Manage data lake artifacts
    Lake {
        #[command(subcommand)]
        action: LakeAction,
    },

    /// Generate shell completions
    Completion {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,

        /// Install to standard shell completion directory
        #[arg(long)]
        install: bool,
    },

    /// Launch interactive TUI for browsing the node graph
    Tui,
}

#[derive(Subcommand)]
pub enum NodeAction {
    /// Print a node
    Get {
        /// Node id
        id: String,
    },
    /// Create a new node
    Create {
        /// Node id (e.g. auth:oauth:google)
        id: Option<String>,

        /// Node content
        #[arg(short, long)]
        content: Option<String>,

        /// Node weight (0-100)
        #[arg(short, long, default_value_t = 50)]
        weight: u8,

        /// Link data lake files (repeatable)
        #[arg(short, long)]
        data_lake: Vec<String>,

        /// Add edge (repeatable, format: "target:type:weight" e.g. "auth:session:uses:50")
        #[arg(long)]
        add_edge: Vec<String>,

        /// Add source file (repeatable)
        #[arg(long)]
        add_source_file: Vec<String>,

        /// Open $EDITOR to create node
        #[arg(short, long)]
        edit: bool,
    },
    /// Update an existing node
    Update {
        /// Node id
        id: String,

        /// New content
        #[arg(short, long)]
        content: Option<String>,

        /// New weight (0-100)
        #[arg(short, long)]
        weight: Option<u8>,

        /// Add data lake files (repeatable)
        #[arg(long)]
        add_data_lake: Vec<String>,

        /// Remove data lake files (repeatable)
        #[arg(long)]
        remove_data_lake: Vec<String>,

        /// Add edge (repeatable, format: "target:type:weight" e.g. "auth:session:uses:50")
        #[arg(long)]
        add_edge: Vec<String>,

        /// Remove edge by target node id (repeatable)
        #[arg(long)]
        remove_edge: Vec<String>,

        /// Add source file (repeatable)
        #[arg(long)]
        add_source_file: Vec<String>,

        /// Remove source file (repeatable)
        #[arg(long)]
        remove_source_file: Vec<String>,

        /// Open $EDITOR with current node content
        #[arg(short, long)]
        edit: bool,
    },
    /// Mark a node as deprecated
    Deprecate {
        /// Node id
        id: String,
    },
}

#[derive(Subcommand)]
pub enum LakeAction {
    /// Add a file to the data lake
    Add {
        /// File path to copy into data lake
        file: String,

        /// Link to this node (optional)
        #[arg(short, long)]
        link: Option<String>,
    },
    /// List all files in the data lake
    List,
    /// Remove a file from the data lake
    Remove {
        /// Filename in data lake
        file: String,
    },
}
