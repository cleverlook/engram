use crate::models::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NodeList,
    NodeDetail,
    Search,
}

pub struct DetailState {
    pub scroll: u16,
}

impl DetailState {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub nodes: Vec<Node>,
    pub selected_index: usize,
    pub engram_dir: std::path::PathBuf,
    pub detail_state: DetailState,
}

impl App {
    pub fn new(nodes: Vec<Node>, engram_dir: std::path::PathBuf) -> Self {
        Self {
            running: true,
            view: View::NodeList,
            nodes,
            selected_index: 0,
            engram_dir,
            detail_state: DetailState::new(),
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

    pub fn enter_search(&mut self) {
        // Will be implemented in Task 6
        self.view = View::Search;
    }
}
