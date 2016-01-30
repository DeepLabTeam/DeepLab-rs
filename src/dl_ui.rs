use std::rc::Rc;

use conrod::Ui;
use graphics::Context;
use matrix;
use opengl_graphics::GlGraphics;
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input;
use dl;

use super::graph_builder::GraphBuilder;
use super::node_block::NodeBlock;
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
    mouse: Mouse,
}

impl DeepLabUi {
    pub fn new() -> DeepLabUi {
        let mat_mul = Rc::new(Operation::new("MatMul".to_string(), 2, 1,
            Box::new(|ctx: &matrix::Context,
                      graph: &mut dl::Graph,
                      vars: &VarStore,
                      _in: &[Option<VarIndex>],
                      _out: &[VarIndex]| {
                let a = *_in[0].unwrap().get(&vars);
                let b = *_in[1].unwrap().get(&vars);
                let op = Box::new(dl::op::MatMul::new(&ctx,
                                                      a.shape,
                                                      b.shape));
                graph.add_node(&ctx, op,
                               vec![a.gpu.unwrap(), b.gpu.unwrap()],
                               &[_out[0].get(vars).shape])
            })));
        DeepLabUi {
            activation_blocks: [[mat_mul.clone(), mat_mul.clone()],
                                [mat_mul.clone(), mat_mul.clone()]],
            graph: GraphBuilder::new(),
            ctx: matrix::Context::new(),

            place_op: None,
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
        self.graph.event(event, &self.mouse);
    }

    pub fn draw(&self, c: Context, gl: &mut GlGraphics) {
        self.graph.draw(&c, gl);
        /*self.v48_graph.draw(c.trans(ui.win_w - 405.0, 5.0), gl, &mut *ui.glyph_cache.borrow_mut());
        self.a24_graph.draw(c.trans(ui.win_w - 405.0, 185.0), gl, &mut *ui.glyph_cache.borrow_mut());
        self.v12_graph.draw(c.trans(ui.win_w - 405.0, 365.0), gl, &mut *ui.glyph_cache.borrow_mut());
        self.motor_temp_graph.draw(c.trans(ui.win_w - 405.0, 545.0), gl, &mut *ui.glyph_cache.borrow_mut());*/
    }

    pub fn set_widgets<'a>(&mut self, ui: &mut Ui<GlyphCache<'a>>) {
        use conrod::{color, Button, Canvas, Colorable, Labelable, Positionable, Sizeable, Tabs, Text, Widget, WidgetMatrix};

        // Construct our main `Canvas` tree.
        Canvas::new().flow_down(&[
            (UPPER, Canvas::new().color(color::rgb(1.0, 1.0, 0.8)).pad_bottom(20.0)),
            (LOWER, Canvas::new().color(color::rgb(1.0, 0.8, 1.0)).scroll_kids_vertically().flow_right(&[
                (BLOCKS, Canvas::new().color(color::rgb(0.8, 1.0, 1.0)).pad_bottom(10.0)),
                (RELU_B, Canvas::new().color(color::rgb(0.8, 1.0, 0.8)).pad_bottom(10.0)),
            ])),
        ]).set(MASTER, ui);

        Text::new("Fancy Neural Net").color(color::rgb(0.0, 0.0, 0.0))
                                     .font_size(48)
                                     .middle_of(UPPER)
                                     .set(TITLE, ui);

        NodeBlock::new().color(color::rgb(1.0, 0.0, 0.0))
                        .w_h(64.0, 64.0)
                        .x_y(10.0, 10.0)
                        .floating(true)
                        .react(|| println!("Click"))
                        //.mid_top_of(UPPER)
                        .set(NODE, ui);

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

        // Time delay
        /*Text::new("Time Delay: 0s")
            .xy((-ui.win_w / 2.0) + 70.0, (ui.win_h / 2.0) - 150.0)
            .font_size(18)
            .color(self.bg_color.plain_contrast())
            .set(TIME_DELAY, ui);*/
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
    UPPER,
    LOWER,

    // Widget IDs
    NODE,
    BLOCKS,
    ACTIVATION_BLOCK_MATRIX,
    RELU_B,
    TITLE,
    MATMUL,
    RELU,
}
