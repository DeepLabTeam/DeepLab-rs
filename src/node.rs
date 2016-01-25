use std::rc::Rc;

use graphics;
use piston::input;
use opengl_graphics::GlGraphics;
use vecmath;

use super::graph_builder::GraphBuilder;
use super::op::Operation;
use super::var_store::VarIndex;

pub enum NodeAction {
    DragInput(usize),
    DragOutput(usize),
    Idle,
}

pub struct Node {
    name: String,
    op: Rc<Operation>,
    inputs: Vec<Option<VarIndex>>,
    outputs: Vec<Option<VarIndex>>,
    pos: [f64; 2],
    action: NodeAction,
}

impl Node {
    pub fn new(name: String, pos: [f64; 2], op: Rc<Operation>, num_in: u64, num_out: u64) -> Self {
        Node {
            name: name,
            op: op,
            inputs: vec![None; num_in as usize],
            outputs: vec![None; num_out as usize],
            pos: pos,
            action: NodeAction::Idle,
        }
    }

    pub fn event(&mut self, event: &input::Event, cursor: [f64; 2]) {
        use piston::input::*;

        let input_spacing = 32.0 / (self.inputs.len() as f64);

        let click = || {
            for (i, input) in self.inputs.iter().enumerate() {
                let mut pos = [-4.0, input_spacing*(i as f64) + input_spacing/2.0 - 4.0];
                pos[0] += self.pos[0];
                pos[1] += self.pos[1];
                if is_over_circle(pos, 4.0, pos) {
                    self.action = NodeAction::DragInput(i);
                }
            }

            let output_spacing = 32.0 / (self.outputs.len() as f64);
            for (i, output) in self.outputs.iter().enumerate() {
                let mut pos = [64.0 - 4.0, output_spacing*(i as f64) + output_spacing/2.0 - 4.0];
                pos[0] += self.pos[0];
                pos[1] += self.pos[1];
                if is_over_circle(pos, 4.0, pos) {
                    self.action = NodeAction::DragOutput(i);
                }
            }
        };

        event.press(|button| {
            match button {
                Button::Mouse(button) => {
                    /*match button {
                        mouse::Button::Left => {
                            click();
                        },
                        _ => { },
                    }*/
                },
                _ => { },
            }
        });
    }

    pub fn draw(&self, c: &graphics::Context, gl: &mut GlGraphics) {
        use graphics::{Ellipse, Rectangle, Transformed};

        Rectangle::new([1.0, 0.0, 0.0, 1.0]).draw([self.pos[0], self.pos[1], 64.0, 32.0], &c.draw_state, c.transform, gl);

        let input_spacing = 32.0 / (self.inputs.len() as f64);

        for (i, input) in self.inputs.iter().enumerate() {
            let pos = [-4.0, input_spacing*(i as f64) + input_spacing/2.0 - 4.0];
            let c = c.trans(self.pos[0], self.pos[1]);
            match *input {
                Some(input) => {
                    Ellipse::new([1.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
                },
                None => {
                    Ellipse::new([0.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
                },
            }
        }

        let output_spacing = 32.0 / (self.outputs.len() as f64);

        for (i, output) in self.outputs.iter().enumerate() {
            let pos = [64.0 - 4.0, output_spacing*(i as f64) + output_spacing/2.0 - 4.0];
            let c = c.trans(self.pos[0], self.pos[1]);
            match *output {
                Some(output) => {
                    Ellipse::new([1.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
                },
                None => {
                    Ellipse::new([0.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
                },
            }
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

/// Return whether or not a given point is over a circle at a given point on a
/// Cartesian plane. We use this to determine whether the mouse is over the button.
pub fn is_over_circle(circ_center: [f64; 2], radius: f64, mouse_point: [f64; 2]) -> bool {
    // Offset vector from the center of the circle to the mouse.
    let offset = vecmath::vec2_sub(mouse_point, circ_center);

    vecmath::vec2_len(offset) <= radius / 2.0
}
