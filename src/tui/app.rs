use tui_input::Input;

use crate::models::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NodeList,
    NodeDetail,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    Id,
    Weight,
    Touched,
    Status,
}

pub struct DetailState {
    pub scroll: u16,
    pub selected_edge: usize,
    pub history: Vec<String>,
}

pub struct SearchState {
    pub input: Input,
    pub results: Vec<String>, // node IDs
    pub selected: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            input: Input::default(),
            results: Vec::new(),
            selected: 0,
        }
    }

    pub fn next(&mut self) {
        if !self.results.is_empty() {
            self.selected = (self.selected + 1) % self.results.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.results.is_empty() {
            self.selected = if self.selected == 0 {
                self.results.len() - 1
            } else {
                self.selected - 1
            };
        }
    }
}

impl DetailState {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            selected_edge: 0,
            history: Vec::new(),
        }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn next_edge(&mut self, edge_count: usize) {
        if edge_count > 0 {
            self.selected_edge = (self.selected_edge + 1) % edge_count;
        }
    }

    pub fn prev_edge(&mut self, edge_count: usize) {
        if edge_count > 0 {
            self.selected_edge = if self.selected_edge == 0 {
                edge_count - 1
            } else {
                self.selected_edge - 1
            };
        }
    }
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub nodes: Vec<Node>,
    pub selected_index: usize,
    pub engram_dir: std::path::PathBuf,
    pub detail_state: DetailState,
    pub search_state: SearchState,
    pub sort_by: SortBy,
}

impl App {
    pub fn new(mut nodes: Vec<Node>, engram_dir: std::path::PathBuf) -> Self {
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        Self {
            running: true,
            view: View::NodeList,
            nodes,
            selected_index: 0,
            engram_dir,
            detail_state: DetailState::new(),
            search_state: SearchState::new(),
            sort_by: SortBy::Id,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.nodes.get(self.selected_index)
    }

    pub fn next(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.nodes.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.nodes.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.nodes.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub fn enter_detail(&mut self) {
        if self.selected_node().is_some() {
            self.detail_state = DetailState::new();
            self.view = View::NodeDetail;
        }
    }

    pub fn back(&mut self) {
        match self.view {
            View::NodeDetail | View::Search => self.view = View::NodeList,
            _ => {}
        }
    }

    pub fn navigate_to_edge(&mut self) {
        let Some(node) = self.nodes.get(self.selected_index) else {
            return;
        };
        let Some(edge) = node.edges.get(self.detail_state.selected_edge) else {
            return;
        };
        let target_id = edge.to.clone();
        let Some(target_pos) = self.nodes.iter().position(|n| n.id == target_id) else {
            return;
        };
        self.detail_state.history.push(node.id.clone());
        self.selected_index = target_pos;
        self.detail_state.scroll = 0;
        self.detail_state.selected_edge = 0;
    }

    pub fn navigate_back(&mut self) {
        if let Some(prev_id) = self.detail_state.history.pop() {
            if let Some(pos) = self.nodes.iter().position(|n| n.id == prev_id) {
                self.selected_index = pos;
                self.detail_state.scroll = 0;
                self.detail_state.selected_edge = 0;
            } else {
                self.view = View::NodeList;
            }
        } else {
            self.view = View::NodeList;
        }
    }

    pub fn enter_search(&mut self) {
        self.search_state = SearchState::new();
        self.view = View::Search;
    }

    pub fn execute_search(&mut self) {
        let query = self.search_state.input.value().trim().to_string();
        if query.is_empty() {
            self.search_state.results.clear();
            return;
        }
        match crate::db::search(&self.engram_dir, &query) {
            Ok(ids) => {
                self.search_state.results = ids;
                self.search_state.selected = 0;
            }
            Err(_) => {
                self.search_state.results.clear();
            }
        }
    }

    pub fn open_search_result(&mut self) {
        if let Some(id) = self
            .search_state
            .results
            .get(self.search_state.selected)
            .cloned()
            && let Some(pos) = self.nodes.iter().position(|n| n.id == id)
        {
            self.selected_index = pos;
            self.detail_state = DetailState::new();
            self.view = View::NodeDetail;
        }
    }

    pub fn cycle_sort(&mut self) {
        self.sort_by = match self.sort_by {
            SortBy::Id => SortBy::Weight,
            SortBy::Weight => SortBy::Touched,
            SortBy::Touched => SortBy::Status,
            SortBy::Status => SortBy::Id,
        };
        let selected_id = self.selected_node().map(|n| n.id.clone());
        match self.sort_by {
            SortBy::Id => self.nodes.sort_by(|a, b| a.id.cmp(&b.id)),
            SortBy::Weight => self.nodes.sort_by(|a, b| b.weight.cmp(&a.weight)),
            SortBy::Touched => self.nodes.sort_by(|a, b| b.touched.cmp(&a.touched)),
            SortBy::Status => self
                .nodes
                .sort_by(|a, b| format!("{:?}", a.status).cmp(&format!("{:?}", b.status))),
        }
        // Preserve selection
        if let Some(id) = selected_id
            && let Some(pos) = self.nodes.iter().position(|n| n.id == id)
        {
            self.selected_index = pos;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_node(id: &str) -> Node {
        make_node_with_edges(id, vec![])
    }

    #[test]
    fn navigation_wraps_around() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        assert_eq!(app.selected_index, 0);
        app.previous(); // wrap to end
        assert_eq!(app.selected_index, 2);
        app.next(); // wrap to start
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn enter_detail_and_back() {
        let nodes = vec![make_node("a")];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        app.enter_detail();
        assert_eq!(app.view, View::NodeDetail);
        app.back();
        assert_eq!(app.view, View::NodeList);
    }

    #[test]
    fn enter_search_and_back() {
        let nodes = vec![make_node("a")];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        app.enter_search();
        assert_eq!(app.view, View::Search);
        app.back();
        assert_eq!(app.view, View::NodeList);
    }

    #[test]
    fn empty_nodes_navigation_safe() {
        let mut app = App::new(vec![], std::path::PathBuf::from("/tmp"));
        app.next(); // should not panic
        app.previous(); // should not panic
        assert_eq!(app.selected_index, 0);
        assert!(app.selected_node().is_none());
    }

    #[test]
    fn quit_sets_running_false() {
        let mut app = App::new(vec![], std::path::PathBuf::from("/tmp"));
        assert!(app.running);
        app.quit();
        assert!(!app.running);
    }

    fn make_node_with_edges(id: &str, edges: Vec<crate::models::node::Edge>) -> Node {
        Node {
            id: id.to_string(),
            content: format!("Content for {id}"),
            weight: 50,
            status: crate::models::node::NodeStatus::Active,
            source_files: vec![],
            source_hash: None,
            created: Utc::now(),
            touched: Utc::now(),
            data_lake: vec![],
            edges,
        }
    }

    fn make_edge(to: &str) -> crate::models::node::Edge {
        crate::models::node::Edge {
            to: to.to_string(),
            edge_type: crate::models::node::EdgeType::Related,
            weight: 50,
        }
    }

    #[test]
    fn navigate_to_edge_pushes_history() {
        let nodes = vec![
            make_node_with_edges("a", vec![make_edge("b")]),
            make_node_with_edges("b", vec![]),
        ];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        // "a" is index 0, "b" is index 1 (sorted by id)
        app.enter_detail();
        app.navigate_to_edge();
        // Should now be on node "b"
        assert_eq!(app.selected_index, 1);
        assert_eq!(app.selected_node().unwrap().id, "b");
        // History should have 1 entry (the previous index)
        assert_eq!(app.detail_state.history.len(), 1);
        assert_eq!(app.detail_state.history[0], "a");
        assert_eq!(app.view, View::NodeDetail);
    }

    #[test]
    fn navigate_back_pops_history() {
        let nodes = vec![
            make_node_with_edges("a", vec![make_edge("b")]),
            make_node_with_edges("b", vec![]),
        ];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        app.enter_detail();
        app.navigate_to_edge();
        assert_eq!(app.selected_node().unwrap().id, "b");
        app.navigate_back();
        assert_eq!(app.selected_node().unwrap().id, "a");
        assert!(app.detail_state.history.is_empty());
        assert_eq!(app.view, View::NodeDetail);
    }

    #[test]
    fn navigate_back_with_empty_history_goes_to_list() {
        let nodes = vec![make_node_with_edges("a", vec![])];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        app.enter_detail();
        assert_eq!(app.view, View::NodeDetail);
        app.navigate_back();
        assert_eq!(app.view, View::NodeList);
    }

    #[test]
    fn edge_selection_wraps() {
        let mut detail = DetailState::new();
        // 3 edges: wrapping forward
        detail.next_edge(3);
        assert_eq!(detail.selected_edge, 1);
        detail.next_edge(3);
        assert_eq!(detail.selected_edge, 2);
        detail.next_edge(3);
        assert_eq!(detail.selected_edge, 0); // wrapped
        // wrapping backward
        detail.prev_edge(3);
        assert_eq!(detail.selected_edge, 2); // wrapped back
        detail.prev_edge(3);
        assert_eq!(detail.selected_edge, 1);
    }

    #[test]
    fn navigate_to_nonexistent_edge_target_is_noop() {
        let nodes = vec![make_node_with_edges("a", vec![make_edge("nonexistent")])];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        app.enter_detail();
        app.navigate_to_edge();
        // Should stay on "a"
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.selected_node().unwrap().id, "a");
        assert!(app.detail_state.history.is_empty());
    }
}
