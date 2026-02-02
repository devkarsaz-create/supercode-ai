use crate::types::AgentState;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub state: AgentState,
}

#[derive(Debug, Clone)]
pub struct AgentGraph {
    pub nodes: Vec<Node>,
    pub edges: Vec<(usize, usize)>,
}

impl AgentGraph {
    pub fn new() -> Self {
        Self { nodes: vec![], edges: vec![] }
    }

    pub fn add_node(&mut self, name: &str, state: AgentState) {
        let id = self.nodes.len();
        self.nodes.push(Node { id, name: name.into(), state });
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.push((from, to));
    }

    pub fn print(&self) {
        println!("AgentGraph:");
        for n in &self.nodes {
            println!("- {} ({}): {:?}", n.id, n.name, n.state);
        }
        println!("Edges:");
        for (f, t) in &self.edges {
            println!("{} -> {}", f, t);
        }
    }
}
