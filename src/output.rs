use console::style;

use crate::models::node::{Node, NodeStatus};

pub fn print_success(msg: &str) {
    println!("{} {}", style("✓").green().bold(), msg);
}

pub fn print_info(msg: &str) {
    println!("{} {}", style("ℹ").cyan(), msg);
}

pub fn print_node_header(node: &Node) {
    let status_styled = match node.status {
        NodeStatus::Active => style("active").green(),
        NodeStatus::Dirty => style("dirty").yellow(),
        NodeStatus::Stale => style("stale").yellow(),
        NodeStatus::Deprecated => style("deprecated").red(),
    };

    println!(
        "{} {} {} {}",
        style(&node.id).bold(),
        style(format!("w:{}", node.weight)).dim(),
        status_styled,
        style(format!("touched:{}", node.touched)).dim(),
    );
}

pub fn print_node_full(node: &Node) {
    print_node_header(node);
    println!();
    println!("{}", node.content.trim());

    if !node.edges.is_empty() {
        println!();
        println!("{}", style("edges:").dim());
        for edge in &node.edges {
            println!(
                "  {} {} {} {}",
                style("→").cyan(),
                edge.to,
                style(format!("{:?}", edge.edge_type)).dim(),
                style(format!("w:{}", edge.weight)).dim(),
            );
        }
    }

    if !node.data_lake.is_empty() {
        println!();
        println!("{}", style("data_lake:").dim());
        for dl in &node.data_lake {
            println!("  {}", dl);
        }
    }
}
