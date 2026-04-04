use ratatui_textarea::TextArea;
use tui_input::Input;
use tui_tree_widget::TreeState;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Overlay {
    None,
    ConfirmDeprecate,
    CreateForm,
    AddEdgeForm,
    EditForm,
}

pub const EDGE_TYPES: &[&str] = &["uses", "depends_on", "implements", "rationale", "related"];
pub const STATUSES: &[&str] = &["active", "dirty", "stale", "deprecated"];

pub struct CreateFormState {
    pub id_input: Input,
    pub content_textarea: TextArea<'static>,
    pub weight_input: Input,
    pub focused_field: usize, // 0=id, 1=content, 2=weight
}

impl CreateFormState {
    pub fn new() -> Self {
        let mut weight = Input::default();
        for c in "50".chars() {
            weight.handle(tui_input::InputRequest::InsertChar(c));
        }
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(ratatui::prelude::Style::default());
        Self {
            id_input: Input::default(),
            content_textarea: textarea,
            weight_input: weight,
            focused_field: 0,
        }
    }

    pub fn content_text(&self) -> String {
        self.content_textarea.lines().join("\n")
    }

    pub fn next_field(&mut self) {
        self.focused_field = (self.focused_field + 1) % 3;
    }

    pub fn prev_field(&mut self) {
        self.focused_field = if self.focused_field == 0 {
            2
        } else {
            self.focused_field - 1
        };
    }

    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        match self.focused_field {
            0 => {
                use tui_input::backend::crossterm::EventHandler;
                self.id_input
                    .handle_event(&crossterm::event::Event::Key(key));
            }
            1 => {
                self.content_textarea.input(ratatui_textarea::Input {
                    key: convert_key(key.code),
                    ctrl: key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL),
                    alt: key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
                    shift: key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT),
                });
            }
            _ => {
                use tui_input::backend::crossterm::EventHandler;
                self.weight_input
                    .handle_event(&crossterm::event::Event::Key(key));
            }
        }
    }
}

pub struct AddEdgeFormState {
    pub target_input: Input,
    pub edge_type_index: usize,
    pub weight_input: Input,
    pub focused_field: usize,
    pub suggestions: Vec<String>,
    pub selected_suggestion: usize,
}

impl AddEdgeFormState {
    pub fn new() -> Self {
        let mut weight = Input::default();
        for c in "50".chars() {
            weight.handle(tui_input::InputRequest::InsertChar(c));
        }
        Self {
            target_input: Input::default(),
            edge_type_index: 0,
            weight_input: weight,
            focused_field: 0,
            suggestions: Vec::new(),
            selected_suggestion: 0,
        }
    }

    pub fn update_suggestions(&mut self, node_ids: &[String]) {
        let query = self.target_input.value().to_lowercase();
        if query.is_empty() {
            self.suggestions.clear();
        } else {
            self.suggestions = node_ids
                .iter()
                .filter(|id| id.to_lowercase().contains(&query))
                .take(8)
                .cloned()
                .collect();
        }
        self.selected_suggestion = 0;
    }

    pub fn next_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_suggestion = (self.selected_suggestion + 1) % self.suggestions.len();
        }
    }

    pub fn prev_suggestion(&mut self) {
        if !self.suggestions.is_empty() {
            self.selected_suggestion = if self.selected_suggestion == 0 {
                self.suggestions.len() - 1
            } else {
                self.selected_suggestion - 1
            };
        }
    }

    pub fn accept_suggestion(&mut self) {
        if let Some(id) = self.suggestions.get(self.selected_suggestion).cloned() {
            self.target_input = Input::default();
            for c in id.chars() {
                self.target_input
                    .handle(tui_input::InputRequest::InsertChar(c));
            }
            self.suggestions.clear();
        }
    }

    pub fn next_field(&mut self) {
        self.focused_field = (self.focused_field + 1) % 3;
    }

    pub fn prev_field(&mut self) {
        self.focused_field = if self.focused_field == 0 {
            2
        } else {
            self.focused_field - 1
        };
    }

    pub fn next_type(&mut self) {
        self.edge_type_index = (self.edge_type_index + 1) % EDGE_TYPES.len();
    }

    pub fn prev_type(&mut self) {
        self.edge_type_index = if self.edge_type_index == 0 {
            EDGE_TYPES.len() - 1
        } else {
            self.edge_type_index - 1
        };
    }

    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        use tui_input::backend::crossterm::EventHandler;
        let input = match self.focused_field {
            0 => &mut self.target_input,
            2 => &mut self.weight_input,
            _ => return, // type field uses left/right, not text input
        };
        input.handle_event(&crossterm::event::Event::Key(key));
    }
}

pub struct EditFormState {
    pub node_id: String,
    pub content_textarea: TextArea<'static>,
    pub weight_input: Input,
    pub status_index: usize,
    pub focused_field: usize, // 0=content, 1=weight, 2=status
}

impl EditFormState {
    pub fn empty() -> Self {
        Self {
            node_id: String::new(),
            content_textarea: TextArea::default(),
            weight_input: Input::default(),
            status_index: 0,
            focused_field: 0,
        }
    }

    pub fn from_node(node: &crate::models::node::Node) -> Self {
        use crate::models::node::NodeStatus;
        let lines: Vec<String> = node.content.lines().map(String::from).collect();
        let mut textarea = TextArea::new(lines);
        textarea.set_cursor_line_style(ratatui::prelude::Style::default());
        let mut weight = Input::default();
        for c in node.weight.to_string().chars() {
            weight.handle(tui_input::InputRequest::InsertChar(c));
        }
        let status_index = match node.status {
            NodeStatus::Active => 0,
            NodeStatus::Dirty => 1,
            NodeStatus::Stale => 2,
            NodeStatus::Deprecated => 3,
        };
        Self {
            node_id: node.id.clone(),
            content_textarea: textarea,
            weight_input: weight,
            status_index,
            focused_field: 0,
        }
    }

    pub fn content_text(&self) -> String {
        self.content_textarea.lines().join("\n")
    }

    pub fn next_field(&mut self) {
        self.focused_field = (self.focused_field + 1) % 3;
    }

    pub fn prev_field(&mut self) {
        self.focused_field = if self.focused_field == 0 {
            2
        } else {
            self.focused_field - 1
        };
    }

    pub fn next_status(&mut self) {
        self.status_index = (self.status_index + 1) % STATUSES.len();
    }

    pub fn prev_status(&mut self) {
        self.status_index = if self.status_index == 0 {
            STATUSES.len() - 1
        } else {
            self.status_index - 1
        };
    }

    pub fn handle_input(&mut self, key: crossterm::event::KeyEvent) {
        match self.focused_field {
            0 => {
                // Convert our crossterm 0.28 KeyEvent to ratatui_textarea::Input manually
                self.content_textarea.input(ratatui_textarea::Input {
                    key: convert_key(key.code),
                    ctrl: key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL),
                    alt: key.modifiers.contains(crossterm::event::KeyModifiers::ALT),
                    shift: key
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::SHIFT),
                });
            }
            1 => {
                use tui_input::backend::crossterm::EventHandler;
                self.weight_input
                    .handle_event(&crossterm::event::Event::Key(key));
            }
            _ => {} // status uses left/right
        }
    }
}

fn convert_key(code: crossterm::event::KeyCode) -> ratatui_textarea::Key {
    use crossterm::event::KeyCode;
    use ratatui_textarea::Key;
    match code {
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::F(n) => Key::F(n),
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Enter => Key::Enter,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Tab => Key::Tab,
        KeyCode::Delete => Key::Delete,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::PageUp => Key::PageUp,
        KeyCode::PageDown => Key::PageDown,
        KeyCode::Esc => Key::Esc,
        _ => Key::Null,
    }
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
    pub overlay: Overlay,
    pub create_form: CreateFormState,
    pub add_edge_form: AddEdgeFormState,
    pub edit_form: EditFormState,
    pub status_message: Option<(String, bool)>, // (message, is_error)
    pub tree_state: TreeState<String>,
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
            overlay: Overlay::None,
            create_form: CreateFormState::new(),
            add_edge_form: AddEdgeFormState::new(),
            edit_form: EditFormState::empty(),
            status_message: None,
            tree_state: TreeState::default(),
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.nodes.get(self.selected_index)
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

    pub fn reload_nodes(&mut self, mut nodes: Vec<Node>) {
        let selected_id = self.selected_node().map(|n| n.id.clone());
        match self.sort_by {
            SortBy::Id => nodes.sort_by(|a, b| a.id.cmp(&b.id)),
            SortBy::Weight => nodes.sort_by(|a, b| b.weight.cmp(&a.weight)),
            SortBy::Touched => nodes.sort_by(|a, b| b.touched.cmp(&a.touched)),
            SortBy::Status => {
                nodes.sort_by(|a, b| format!("{:?}", a.status).cmp(&format!("{:?}", b.status)))
            }
        }
        self.nodes = nodes;
        if let Some(id) = selected_id
            && let Some(pos) = self.nodes.iter().position(|n| n.id == id)
        {
            self.selected_index = pos;
        }
        if self.selected_index >= self.nodes.len() {
            self.selected_index = self.nodes.len().saturating_sub(1);
        }
    }

    pub fn open_create_form(&mut self) {
        self.create_form = CreateFormState::new();
        self.overlay = Overlay::CreateForm;
    }

    pub fn open_edit_form(&mut self) {
        if let Some(node) = self.selected_node().cloned() {
            self.edit_form = EditFormState::from_node(&node);
            self.overlay = Overlay::EditForm;
        }
    }

    pub fn open_add_edge_form(&mut self) {
        self.add_edge_form = AddEdgeFormState::new();
        self.overlay = Overlay::AddEdgeForm;
    }

    pub fn confirm_deprecate(&mut self) {
        self.overlay = Overlay::ConfirmDeprecate;
    }

    pub fn close_overlay(&mut self) {
        self.overlay = Overlay::None;
    }

    pub fn set_status(&mut self, msg: String, is_error: bool) {
        self.status_message = Some((msg, is_error));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
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

    #[test]
    fn reload_nodes_updates_list() {
        let nodes = vec![make_node("a"), make_node("b")];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        assert_eq!(app.nodes.len(), 2);
        let new_nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        app.reload_nodes(new_nodes);
        assert_eq!(app.nodes.len(), 3);
    }

    #[test]
    fn reload_nodes_preserves_selection_by_id() {
        let nodes = vec![make_node("a"), make_node("b"), make_node("c")];
        let mut app = App::new(nodes, std::path::PathBuf::from("/tmp"));
        app.selected_index = 1; // "b"
        let new_nodes = vec![make_node("c"), make_node("a"), make_node("b")];
        app.reload_nodes(new_nodes);
        assert_eq!(app.selected_node().unwrap().id, "b");
    }

    #[test]
    fn create_form_field_cycling() {
        let mut form = CreateFormState::new();
        assert_eq!(form.focused_field, 0);
        form.next_field();
        assert_eq!(form.focused_field, 1);
        form.next_field();
        assert_eq!(form.focused_field, 2);
        form.next_field();
        assert_eq!(form.focused_field, 0);
    }

    #[test]
    fn add_edge_form_type_cycling() {
        let mut form = AddEdgeFormState::new();
        assert_eq!(form.edge_type_index, 0);
        form.next_type();
        assert_eq!(form.edge_type_index, 1);
        form.prev_type();
        assert_eq!(form.edge_type_index, 0);
        form.prev_type();
        assert_eq!(form.edge_type_index, EDGE_TYPES.len() - 1);
    }

    #[test]
    fn create_form_default_weight() {
        let form = CreateFormState::new();
        assert_eq!(form.weight_input.value(), "50");
    }

    #[test]
    fn edit_form_from_node() {
        let node = make_node("test:node");
        let form = EditFormState::from_node(&node);
        assert_eq!(form.node_id, "test:node");
        assert_eq!(form.content_text(), "Content for test:node");
        assert_eq!(form.weight_input.value(), "50");
        assert_eq!(form.status_index, 0); // Active
    }
}
