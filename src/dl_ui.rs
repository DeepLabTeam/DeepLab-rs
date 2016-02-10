use std::rc::Rc;

use conrod::Ui;
use graphics::Context;
use matrix;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input;
use dl;

use super::graph_builder::{GraphAction, GraphBuilder};
use super::op::Operation;
use super::var_store::{VarStore, VarIndex};

pub struct Mouse {
    pub pos: [f64; 2],
    pub lmb: bool,
    pub rmb: bool,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            pos: [0.0; 2],
            lmb: false,
            rmb: false,
        }
    }
}

pub struct DeepLabUi {
    activation_blocks: [[Rc<Operation>; 2]; 2],
    graph: GraphBuilder,
    ctx: matrix::Context,

    place_op: Option<Rc<Operation>>,
    sel_var: Option<(VarIndex, Vec<usize>)>, // Selected variable
    mouse: Mouse,
}

impl DeepLabUi {
    pub fn new() -> DeepLabUi {
        let mat_mul = Rc::new(Operation::new("MatMul".to_string(), 2, 1,
            |ctx: &matrix::Context,
             graph: &mut dl::Graph,
             vars: &mut VarStore,
             _in: &[Option<VarIndex>],
             _out: &[VarIndex]| {
                let a = *_in[0].unwrap().get(&vars);
                let b = *_in[1].unwrap().get(&vars);
                let op = dl::op::MatMul::new(&ctx, a.shape, b.shape);
                let node = graph.add_node(ctx, op,
                                          vec![a.gpu.unwrap(), b.gpu.unwrap()],
                                          &[_out[0].get(vars).shape]);
                _out[0].get_mut(vars).gpu = Some(node.get(&graph).outputs[0]);
            }));
        let relu = Rc::new(Operation::new("ReLU".to_string(), 1, 1,
            |ctx: &matrix::Context,
             graph: &mut dl::Graph,
             vars: &mut VarStore,
             _in: &[Option<VarIndex>],
             _out: &[VarIndex]| {
                let a = *_in[0].unwrap().get(&vars);
                let op = dl::op::Relu::new();
                let node = graph.add_node(ctx, op,
                                          vec![a.gpu.unwrap()],
                                          &[_out[0].get(vars).shape]);
                _out[0].get_mut(vars).gpu = Some(node.get(&graph).outputs[0]);
            }));
        let mse = Rc::new(Operation::new("MSE".to_string(), 2, 1,
            |ctx: &matrix::Context,
             graph: &mut dl::Graph,
             vars: &mut VarStore,
             _in: &[Option<VarIndex>],
             _out: &[VarIndex]| {
                let a = *_in[0].unwrap().get(&vars);
                let b = *_in[1].unwrap().get(&vars);
                let out_shape = _out[0].get(vars).shape;
                let op = dl::op::Mse::new();
                let node = graph.add_node(ctx, op,
                                          vec![a.gpu.unwrap(), b.gpu.unwrap()],
                                          &[out_shape]);
                _out[0].get_mut(vars).gpu = Some(node.get(&graph).outputs[0]);
                let gradient = graph.add_gradient(ctx, node, 0);
                gradient.get(graph)
                        .set(ctx, &matrix::Matrix::from_vec(out_shape.0, out_shape.1, vec![-0.1]));
            }));
        let variable = Rc::new(Operation::new("Variable".to_string(), 0, 1,
            |ctx: &matrix::Context,
             graph: &mut dl::Graph,
             vars: &mut VarStore,
             _in: &[Option<VarIndex>],
             _out: &[VarIndex]| {
                 _out[0].get_mut(vars).gpu = Some(graph.add_variable(ctx, (1, 1), dl::init::Normal(0.5, 0.2)));
            }));
        DeepLabUi {
            activation_blocks: [[mat_mul.clone(), relu.clone()],
                                [variable.clone(), mse.clone()]],
            graph: GraphBuilder::new(),
            ctx: matrix::Context::new(),

            place_op: None,
            sel_var: None,
            mouse: Mouse::new(),
        }
    }

    pub fn event(&mut self, event: &input::Event) {
        use piston::input::*;
        event.mouse_cursor(|x, y| {
            self.mouse.pos = [x, y];
        });
        event.press(|button| {
            //use piston::input::Button;
            match button {
                Button::Keyboard(key) => println!("Pressed keyboard key '{:?}'", key),
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.mouse.lmb = true;
                        },
                        mouse::MouseButton::Right => {
                            self.mouse.rmb = true;
                            self.place_op = None;
                        },
                        _ => { },
                    }
                },
                _ => { },
            }
        });
        event.release(|button| {
            //use piston::input::Button;
            match button {
                Button::Keyboard(key) => println!("Released keyboard key '{:?}'", key),
                Button::Mouse(button) => {
                    match button {
                        mouse::MouseButton::Left => {
                            self.mouse.lmb = false;
                            if let Some(ref place_op) = self.place_op {
                                self.graph.add_node("asdf".to_string(), self.mouse.pos, place_op.clone());
                            }
                        },
                        mouse::MouseButton::Right => {
                            self.mouse.rmb = false;
                        }
                        _ => { },
                    }
                },
                _ => { },
            }
        });
        if let Some(graph_action) = self.graph.event(event, &self.mouse) {
            match graph_action {
                GraphAction::SelectNode(n) => { },
                GraphAction::SelectVariable(v) => { self.sel_var = Some((v, vec![0, 0])); },
            }
        }
    }

    pub fn draw(&self, c: Context, gl: &mut GlGraphics) {
        self.graph.draw(&c, gl);
    }

    pub fn set_widgets<'a>(&mut self, ui: &mut Ui<GlyphCache<'a>>) {
        use conrod::{color, Button, Canvas, Colorable, Frameable, Labelable, Positionable, Sizeable, Slider, Tabs, Text, Widget, WidgetMatrix};

        // Construct our main `Canvas` tree.
        Canvas::new().flow_down(&[
            (UPPER, Canvas::new().flow_down(&[
                (TOOL_BAR, Canvas::new().color(color::rgb(0.7, 0.7, 0.7)).length(48.0).flow_right(&[
                    (BUILD_BTN_AREA, Canvas::new().color(color::rgb(0.7, 0.7, 0.7))),
                    (RUN_BTN_AREA, Canvas::new().color(color::rgb(0.7, 0.7, 0.7))),
                ])),
                (GRAPH_AREA, Canvas::new().color(color::rgb(1.0, 1.0, 0.8))),
            ])),
            (LOWER, Canvas::new().color(color::rgb(1.0, 0.8, 1.0)).scroll_kids_vertically().flow_right(&[
                (BLOCKS, Canvas::new().color(color::rgb(0.8, 1.0, 1.0)).pad_bottom(10.0)),
                (RELU_B, Canvas::new().color(color::rgb(0.8, 1.0, 0.8)).pad_bottom(10.0)),
                (VAR_MANIP, Canvas::new().color(color::rgb(0.8, 0.2, 0.8)).pad_bottom(10.0).pad_left(10.0)),
            ])),
        ]).set(MASTER, ui);

        /*Text::new("Fancy Neural Net").color(color::rgb(0.0, 0.0, 0.0))
                                     .font_size(48)
                                     .middle_of(GRAPH_AREA)
                                     .set(TITLE, ui);*/

        Button::new().rgb(0.3, 0.3, 0.8)
                     .label("Build")
                     .middle_of(BUILD_BTN_AREA)
                     .react(|| {
                         self.graph.gpu_build(&self.ctx);
                     }).set(BUILD_BTN, ui);

        Button::new().rgb(0.3, 0.3, 0.8)
                     .label("Run")
                     .middle_of(RUN_BTN_AREA)
                     .react(|| {
                         self.graph.graph.run(&self.ctx);
                     }).set(RUN_BTN, ui);

        let footer_wh = ui.wh_of(BLOCKS).unwrap();
        WidgetMatrix::new(2, 2)
            .w_h(footer_wh[0], footer_wh[1])
            .mid_top_of(BLOCKS)
            .each_widget(|n, col, row| {
                let op: Rc<Operation> = {
                    let _row: &[Rc<Operation>; 2] = &self.activation_blocks[row];
                    let op: &Rc<Operation> = &_row[col];
                    op.clone()
                };
                Button::new()
                    .rgb(0.3, 0.8, 0.3)
                    .label(op.name.clone().as_ref())
                    .react(|| {
                        self.place_op = Some(op);
                    })
            }).set(ACTIVATION_BLOCK_MATRIX, ui);

        // Build the variable manipulator
        if let Some((sel_var, ref coords)) = self.sel_var {
            let gpu_var = sel_var.get(&self.graph.vars).gpu.unwrap();
            let var_val = *gpu_var.get(&self.graph.graph).get(&self.ctx).get(coords[0], coords[1]);
            let var_rows = gpu_var.get(&self.graph.graph).get(&self.ctx).rows();
            let var_cols = gpu_var.get(&self.graph.graph).get(&self.ctx).columns();

            Slider::new(var_val, 0.0, 1.0)
                .w_h(30.0, 150.0)
                .mid_left_of(VAR_MANIP)
                .rgb(0.5, 0.3, 0.6)
                .frame(1.0)
                .react(|val| {
                    // TODO
                }).set(VAR_SLIDER, ui);

            WidgetMatrix::new(var_rows, var_cols)
                .w_h(100.0, 100.0)
                .middle_of(VAR_MANIP)
                .each_widget(|n, col, row| {
                    Button::new()
                        .rgb(0.3, 0.8, 0.3)
                        .react(|| {
                            // TODO
                        })
                }).set(VAR_MATRIX, ui);
        }
    }

    pub fn on_key_pressed(&mut self, key: input::Key) {
        match key {
            _ => { },
        }
    }

    pub fn on_key_released(&mut self, key: input::Key) {
        match key {
            _ => { },
        }
    }
}

widget_ids! {
    // Canvas IDs
    MASTER,
    TOOL_BAR,
    BUILD_BTN_AREA,
    RUN_BTN_AREA,
    GRAPH_AREA,
    UPPER,
    LOWER,

    // Widget IDs
    BUILD_BTN,
    RUN_BTN,
    NODE,
    BLOCKS,
    ACTIVATION_BLOCK_MATRIX,
    RELU_B,
    TITLE,
    MATMUL,
    RELU,

    // Variable manipulator
    VAR_MANIP,
    VAR_SLIDER,
    VAR_MATRIX,
    VAR_TRAINABLE,
    VAR_DATASET,
}
