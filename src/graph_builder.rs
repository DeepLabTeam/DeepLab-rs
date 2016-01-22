use std::rc::Rc;

use dl;

use super::node::Node;
use super::op::Operation;

pub struct GraphBuilder {
    graph: dl::Graph,
    nodes: Vec<Node>,
    vars: Vec<(u64, u64)>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder {
            graph: dl::Graph::new(),
            nodes: vec![],
            vars: vec![],
        }
    }

    pub fn add_node(&mut self, name: String, op: Rc<Operation>, num_in: u64, num_out: u64) {
        self.nodes.push(Node::new(name, op, num_in, num_out));
    }
}
