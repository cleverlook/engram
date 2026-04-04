use std::collections::BTreeMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, BorderType, Borders, Paragraph, Wrap};
use tui_tree_widget::{Block as TreeBlock, Tree, TreeItem};

use crate::models::node::Node;
use crate::tui::app::{App, SortBy};
use crate::tui::theme::{Theme, status_color, status_icon};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    let items = build_tree_items(&app.nodes, app.sort_by);
    render_tree(app, &items, frame, chunks[0]);
    render_preview(app, frame, chunks[1]);
}

/// Intermediate structure: maps segment name -> children or leaf data.
struct NsNode {
    /// If this namespace segment corresponds to an actual node, its index into the nodes slice.
    node_index: Option<usize>,
    children: BTreeMap<String, NsNode>,
}

impl NsNode {
    fn new() -> Self {
        Self {
            node_index: None,
            children: BTreeMap::new(),
        }
    }
}

fn build_tree_items(nodes: &[Node], sort_by: SortBy) -> Vec<TreeItem<'static, String>> {
    let mut root: BTreeMap<String, NsNode> = BTreeMap::new();

    for (i, node) in nodes.iter().enumerate() {
        let segments: Vec<&str> = node.id.split(':').collect();
        insert_node(&mut root, &segments, i);
    }

    build_items_from_map(&root, nodes, sort_by)
}

fn insert_node(map: &mut BTreeMap<String, NsNode>, segments: &[&str], node_index: usize) {
    let key = segments[0].to_string();
    let entry = map.entry(key).or_insert_with(NsNode::new);
    if segments.len() == 1 {
        entry.node_index = Some(node_index);
    } else {
        insert_node(&mut entry.children, &segments[1..], node_index);
    }
}

fn build_items_from_map(
    map: &BTreeMap<String, NsNode>,
    nodes: &[Node],
    sort_by: SortBy,
) -> Vec<TreeItem<'static, String>> {
    // Collect entries and sort them
    let mut entries: Vec<(&String, &NsNode)> = map.iter().collect();
    sort_entries(&mut entries, nodes, sort_by);

    let mut items = Vec::new();
    for (key, ns_node) in entries {
        if ns_node.children.is_empty() {
            // Pure leaf
            if let Some(idx) = ns_node.node_index {
                let node = &nodes[idx];
                let text = format_leaf(key, node);
                items.push(TreeItem::new_leaf(node.id.clone(), text));
            }
        } else {
            // Has children — it's a branch
            let mut child_items = Vec::new();
            // If this branch segment is also a node, add a "self" leaf first
            if let Some(idx) = ns_node.node_index {
                let node = &nodes[idx];
                let self_text = format_leaf(key, node);
                child_items.push(TreeItem::new_leaf(node.id.clone(), self_text));
            }
            child_items.extend(build_items_from_map(&ns_node.children, nodes, sort_by));
            let branch_id = format!("ns:{key}");
            let branch_label = Line::from(Span::styled(
                key.clone(),
                Style::default().fg(Theme::ACCENT).bold(),
            ));
            if let Ok(item) = TreeItem::new(branch_id, branch_label, child_items) {
                items.push(item);
            }
        }
    }
    items
}

fn sort_entries<'a>(entries: &mut Vec<(&'a String, &'a NsNode)>, nodes: &[Node], sort_by: SortBy) {
    match sort_by {
        SortBy::Id => {} // BTreeMap already sorted alphabetically
        SortBy::Weight => {
            entries.sort_by(|a, b| {
                let wa = a.1.node_index.map(|i| nodes[i].weight).unwrap_or(0);
                let wb = b.1.node_index.map(|i| nodes[i].weight).unwrap_or(0);
                wb.cmp(&wa) // descending
            });
        }
        SortBy::Touched => {
            entries.sort_by(|a, b| {
                let ta = a.1.node_index.map(|i| nodes[i].touched);
                let tb = b.1.node_index.map(|i| nodes[i].touched);
                tb.cmp(&ta) // most recent first
            });
        }
        SortBy::Status => {
            entries.sort_by(|a, b| {
                let sa = a.1.node_index.map(|i| format!("{:?}", nodes[i].status));
                let sb = b.1.node_index.map(|i| format!("{:?}", nodes[i].status));
                sa.cmp(&sb)
            });
        }
    }
}

fn format_leaf(label: &str, node: &Node) -> Line<'static> {
    let icon = status_icon(&node.status);
    let color = status_color(&node.status);
    Line::from(vec![
        Span::styled(format!("{icon} "), Style::default().fg(color)),
        Span::styled(label.to_string(), Style::default().fg(Theme::TEXT).bold()),
        Span::styled(
            format!("  w:{}", node.weight),
            Style::default().fg(Theme::DIM),
        ),
    ])
}

fn render_tree(app: &mut App, items: &[TreeItem<'static, String>], frame: &mut Frame, area: Rect) {
    let tree_block = TreeBlock::bordered()
        .border_type(BorderType::Rounded)
        .title(format!(
            " Nodes ({}) [sort: {:?}] ",
            app.nodes.len(),
            app.sort_by
        ));

    let tree = Tree::new(items)
        .expect("tree items valid")
        .block(tree_block)
        .highlight_style(Style::default().bg(Theme::SELECTED_BG).bold())
        .highlight_symbol("▶ ");

    frame.render_stateful_widget(tree, area, &mut app.tree_state);
}

fn render_preview(app: &App, frame: &mut Frame, area: Rect) {
    let selected_node = selected_node_from_tree(app);

    let content = match selected_node {
        Some(node) => {
            let icon = status_icon(&node.status);
            let mut lines = vec![
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled("ID: ", Style::default().fg(Theme::DIM)),
                    Span::styled(&node.id, Style::default().fg(Theme::ACCENT).bold()),
                ]),
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled("Status: ", Style::default().fg(Theme::DIM)),
                    Span::styled(
                        format!("{icon} {:?}", node.status),
                        Style::default().fg(status_color(&node.status)),
                    ),
                    Span::styled("  Weight: ", Style::default().fg(Theme::DIM)),
                    Span::raw(format!("{}", node.weight)),
                ]),
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled("Touched: ", Style::default().fg(Theme::DIM)),
                    Span::raw(node.touched.format("%Y-%m-%d %H:%M").to_string()),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    " ─── Content ──────────────────────────────────",
                    Style::default().fg(Theme::HEADER),
                )),
            ];
            for line in node.content.lines() {
                lines.push(Line::from(format!("  {line}")));
            }
            if !node.edges.is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    format!(
                        " ─── Edges: {} ────────────────────────────────",
                        node.edges.len()
                    ),
                    Style::default().fg(Theme::HEADER),
                )));
                for edge in &node.edges {
                    lines.push(Line::from(vec![
                        Span::styled("   → ", Style::default().fg(Theme::SECONDARY)),
                        Span::styled(&edge.to, Style::default().fg(Theme::SECONDARY)),
                        Span::styled(
                            format!("  [{:?} w:{}]", edge.edge_type, edge.weight),
                            Style::default().fg(Theme::DIM),
                        ),
                    ]));
                }
            }
            lines
        }
        None => vec![Line::from(" No node selected")],
    };

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Preview ")
                .title_style(Style::default().bold()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Get the currently selected node from the tree state.
/// Returns `None` if a namespace branch (not a real node) is selected.
pub fn selected_node_from_tree(app: &App) -> Option<&Node> {
    let selected = app.tree_state.selected();
    if selected.is_empty() {
        return None;
    }
    let id = selected.last().unwrap();
    if id.starts_with("ns:") {
        return None;
    }
    app.nodes.iter().find(|n| &n.id == id)
}

/// Check if the selected tree item is a leaf node (not a namespace branch).
pub fn is_leaf_selected(app: &App) -> bool {
    let selected = app.tree_state.selected();
    if selected.is_empty() {
        return false;
    }
    let id = selected.last().unwrap();
    !id.starts_with("ns:")
}
