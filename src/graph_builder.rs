use std::rc::Rc;

use dl;
use graphics;
use matrix;
use piston::input;
use opengl_graphics::GlGraphics;

use super::node::{Node, NodeAction};
use super::op::Operation;
use super::var_store::VarStore;

pub struct GraphBuilder {
    pub graph: dl::Graph,
    pub vars: VarStore,
    nodes: Vec<Node>,
    node_action: Option<(usize, NodeAction)>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        GraphBuilder {
            graph: dl::Graph::new(),
            vars: VarStore::new(),
            nodes: vec![],
            node_action: None,
        }
    }

    pub fn add_node(&mut self, name: String, pos: [f64; 2], op: Rc<Operation>) {
        let num_in = op.num_inputs;
        let num_out = op.num_outputs;
        self.nodes.push(Node::new(name, pos, op, num_in, num_out));
    }

    pub fn event(&mut self, event: &input::Event, cursor: [f64; 2]) {
        for (i, node) in self.nodes.iter_mut().enumerate() {
            node.event(event, cursor);
            if let Some(action) = node.action {
                self.node_action = Some((i, action));
            }
        }
    }

    pub fn draw(&self, c: &graphics::Context, gl: &mut GlGraphics) {
        for node in &self.nodes {
            node.draw(c, gl);
        }
    }

    pub fn gpu_build(&mut self, ctx: &matrix::Context) {
        self.vars.gpu_build(ctx, &mut self.graph);
    }
}
