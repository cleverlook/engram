use crate::models::node::Node;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    NodeList,
    NodeDetail,
    Search,
}

pub struct App {
    pub running: bool,
    pub view: View,
    pub nodes: Vec<Node>,
    pub selected_index: usize,
    pub engram_dir: std::path::PathBuf,
}

impl App {
    pub fn new(nodes: Vec<Node>, engram_dir: std::path::PathBuf) -> Self {
        Self {
            running: true,
            view: View::NodeList,
            nodes,
            selected_index: 0,
            engram_dir,
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
}
