use std::rc::Rc;

use super::graph_builder::GraphBuilder;
use super::op::Operation;
use super::var_store::VarIndex;

pub struct Node {
    name: String,
    op: Rc<Operation>,
    inputs: Vec<Option<VarIndex>>,
    outputs: Vec<Option<VarIndex>>,
}

impl Node {
    pub fn new(name: String, op: Rc<Operation>, num_in: u64, num_out: u64) -> Self {
        Node {
            name: name,
            op: op,
            inputs: vec![None; num_in as usize],
            outputs: vec![None; num_out as usize],
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}
