use std::rc::Rc;

use graphics;
use piston::input;
use opengl_graphics::GlGraphics;
use vecmath;

use super::dl_ui::Mouse;
use super::graph_builder::{GraphBuilder, NodeId};
use super::op::Operation;
use super::var_store::VarIndex;

pub struct Node {
    name: String,
    pub op: Rc<Operation>,
    pub inputs: Vec<Option<VarIndex>>,
    pub outputs: Vec<VarIndex>,
    pos: [f64; 2],
    pub action: Option<NodeAction>,
}

impl Node {
    pub fn new(name: String, pos: [f64; 2], op: Rc<Operation>, num_in: u64, outs: Vec<VarIndex>) -> Self {
        Node {
            name: name,
            op: op,
            inputs: vec![None; num_in as usize],
            outputs: outs,
            pos: pos,
            action: None,
        }
    }

    pub fn event(&mut self, event: &input::Event, mouse: &Mouse) {
        use piston::input::*;

        if let Some(action) = self.action {
            match action {
                NodeAction::DropInput(_) | NodeAction::DropOutput(_) => {
                    self.action = None;
                },
                _ => { },
            }
        }

        let mouse_over = is_over_rect([self.pos[0], self.pos[1], 64.0, 32.0], mouse.pos);

        event.press(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.on_lmb_clicked(mouse, mouse_over);
                        },
                        _ => { },
                    }
                },
                _ => { },
            }
        });
        event.release(|button| {
            match button {
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.on_lmb_released(mouse, mouse_over);
                        },
                        _ => { },
                    }
                },
                _ => { },
            }
        });
    }

    pub fn on_lmb_clicked(&mut self, mouse: &Mouse, mouse_over: bool) {
        let input_spacing = 32.0 / (self.inputs.len() as f64);
        for (i, input) in self.inputs.iter().enumerate() {
            let mut pos = [0.0, input_spacing*(i as f64) + input_spacing/2.0];
            pos[0] += self.pos[0];
            pos[1] += self.pos[1];
            if is_over_circle(pos, 5.0, mouse.pos) {
                self.action = Some(NodeAction::DragInput(i));
                println!("Drag input");
            }
        }

        let output_spacing = 32.0 / (self.outputs.len() as f64);
        for (i, output) in self.outputs.iter().enumerate() {
            let mut pos = [64.0, output_spacing*(i as f64) + output_spacing/2.0];
            pos[0] += self.pos[0];
            pos[1] += self.pos[1];
            if is_over_circle(pos, 5.0, mouse.pos) {
                self.action = Some(NodeAction::DragOutput(i));
                println!("Drag output");
            }
        }
    }

    pub fn on_lmb_released(&mut self, mouse: &Mouse, mouse_over: bool) {
        self.action = None;

        for (i, input) in self.inputs.iter().enumerate() {
            let pos = self.get_input_pos(i);
            if is_over_circle(pos, 5.0, mouse.pos) {
                self.action = Some(NodeAction::DropInput(i));
                println!("Drop input");
            }
        }

        for (i, output) in self.outputs.iter().enumerate() {
            let pos = self.get_output_pos(i);
            if is_over_circle(pos, 5.0, mouse.pos) {
                self.action = Some(NodeAction::DropOutput(i));
                println!("Drop output");
            }
        }
    }

    /*pub fn maybe_input_action(&mut self, mouse_over: bool, lmb: bool) {
        use NodeAction::*;

        match (self.action, mouse_over, lmb) {
            (Idle, true, true) => {
            },
        }
    }*/

    pub fn draw(&self, c: &graphics::Context, gl: &mut GlGraphics) {
        use graphics::{Ellipse, Rectangle, Transformed};

        Rectangle::new([0.1, 0.3, 0.8, 1.0]).draw([self.pos[0], self.pos[1], 64.0, 32.0], &c.draw_state, c.transform, gl);

        for (i, input) in self.inputs.iter().enumerate() {
            let mut pos = self.get_input_pos(i);
            pos[0] -= 4.0;
            pos[1] -= 4.0;
            match *input {
                Some(input) => {
                    Ellipse::new([1.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
                },
                None => {
                    Ellipse::new([0.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
                },
            }
        }

        for (i, output) in self.outputs.iter().enumerate() {
            let mut pos = self.get_output_pos(i);
            pos[0] -= 4.0;
            pos[1] -= 4.0;
            Ellipse::new([1.0, 0.0, 0.0, 1.0]).draw([pos[0], pos[1], 8.0, 8.0], &c.draw_state, c.transform, gl);
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn get_input_pos(&self, i: usize) -> [f64; 2] {
        let input_spacing = 32.0 / (self.inputs.len() as f64);
        let mut pos = [0.0, input_spacing*(i as f64) + input_spacing/2.0];
        pos[0] += self.pos[0];
        pos[1] += self.pos[1];
        pos
    }

    pub fn get_output_pos(&self, i: usize) -> [f64; 2] {
        let output_spacing = 32.0 / (self.outputs.len() as f64);
        let mut pos = [64.0, output_spacing*(i as f64) + output_spacing/2.0];
        pos[0] += self.pos[0];
        pos[1] += self.pos[1];
        pos
    }
}

#[derive(Copy, Clone)]
pub enum NodeAction {
    Drag,
    DragInput(usize),
    DragOutput(usize),
    DropInput(usize),
    DropOutput(usize),
}

pub enum NodeResponse {
    Select,
    SelectInput(usize),
    SelectOutput(usize),
    Connect(NodeId, usize, NodeId, usize),
}

impl NodeAction {
    pub fn happened_before(&self, other: &Self, nodes: (NodeId, NodeId)) -> Option<NodeResponse> {
        use self::NodeAction::*;
        use self::NodeResponse::*;

        match *self {
            DragOutput(send_index) => {
                match *other {
                    DropInput(recv_index) => { Some(Connect(nodes.0, send_index, nodes.1, recv_index)) },
                    DropOutput(i) => { Some(SelectOutput(i)) },
                    _ => None,
                }
            }
            DragInput(recv_index) => {
                match *other {
                    DropOutput(send_index) => { Some(Connect(nodes.1, send_index, nodes.0, recv_index)) },
                    DropInput(i) => { Some(SelectInput(i)) },
                    _ => None,
                }
            },
            _ => None,
        }
    }
}

/// Return whether or not a given point is over a circle at a given point on a
/// Cartesian plane. We use this to determine whether the mouse is over the button.
pub fn is_over_circle(circ_center: [f64; 2], radius: f64, mouse_point: [f64; 2]) -> bool {
    // Offset vector from the center of the circle to the mouse.
    let offset = vecmath::vec2_sub(mouse_point, circ_center);

    vecmath::vec2_len(offset) <= radius / 2.0
}

pub fn is_over_rect(r: [f64; 4], m: [f64; 2]) -> bool {
    if m[0] >= r[0] && m[0] <= r[0]+r[2] && m[1] >= r[1] && m[1] <= r[1]+r[3] {
        true
    } else {
        false
    }
}
