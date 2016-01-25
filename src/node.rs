use std::rc::Rc;

use graphics;
use piston::input;
use opengl_graphics::GlGraphics;

use super::graph_builder::GraphBuilder;
use super::op::Operation;
use super::var_store::VarIndex;

pub struct Node {
    name: String,
    op: Rc<Operation>,
    inputs: Vec<Option<VarIndex>>,
    outputs: Vec<Option<VarIndex>>,
    pos: [f64; 2],
}

impl Node {
    pub fn new(name: String, pos: [f64; 2], op: Rc<Operation>, num_in: u64, num_out: u64) -> Self {
        Node {
            name: name,
            op: op,
            inputs: vec![None; num_in as usize],
            outputs: vec![None; num_out as usize],
            pos: pos,
        }
    }

    pub fn event(&mut self, event: &input::Event) {
    }

    pub fn draw(&self, c: &graphics::Context, gl: &mut GlGraphics) {
        use graphics::Rectangle;

        Rectangle::new([1.0, 0.0, 0.0, 1.0]).draw([self.pos[0], self.pos[1], 64.0, 32.0], &c.draw_state, c.transform, gl);
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}
