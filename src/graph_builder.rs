use std::rc::Rc;

use dl;
use graphics;
use piston::input;
use opengl_graphics::GlGraphics;

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

    pub fn add_node(&mut self, name: String, pos: [f64; 2], op: Rc<Operation>) {
        let num_in = op.num_inputs;
        let num_out = op.num_outputs;
        self.nodes.push(Node::new(name, pos, op, num_in, num_out));
    }

    pub fn event(&mut self, event: &input::Event) {
        for node in &mut self.nodes {
            node.event(event);
        }
    }

    pub fn draw(&self, c: &graphics::Context, gl: &mut GlGraphics) {
        for node in &self.nodes {
            node.draw(c, gl);
        }
    }

    pub fn build(&mut self) {
    }
}
