use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

#[macro_use] extern crate conrod;
extern crate deeplearn as dl;
extern crate time;
extern crate matrix;
extern crate piston;
extern crate graphics;
extern crate opengl_graphics;
extern crate glutin_window;
extern crate vecmath;

use conrod::{
    Theme,
    Ui,
    Widget,
};
use opengl_graphics::{GlGraphics, OpenGL, Texture};
use opengl_graphics::glyph_cache::GlyphCache;
use piston::input::*;
use piston::event_loop::*;
use piston::window::{WindowSettings, Size};
use glutin_window::GlutinWindow;

use dl_ui::DeepLabUi;

mod dl_ui;
mod graph_builder;
mod node;
mod node_block;
mod op;
mod var_store;

fn main() {
    let opengl = OpenGL::V3_2;
    let window = GlutinWindow::new(
        WindowSettings::new(
            "Deep Lab".to_string(),
            Size { width: 1280, height: 700 }
        )
        .exit_on_esc(true)
        .samples(4)
    ).unwrap();
    let window = Rc::new(RefCell::new(window));
    let mut gl = GlGraphics::new(opengl);

    let font_path = Path::new("./assets/fonts/NotoSans-Regular.ttf");
    let theme = Theme::default();
    let glyph_cache = GlyphCache::new(&font_path).unwrap();
    let mut ui = Ui::new(glyph_cache, theme);
    
    ///////////////////////////////////////////////////////////////////////////////////////

    let mut deep_ui = DeepLabUi::new();

    for event in window.clone().events().ups(60) {
        ui.handle_event(&event);
        event.update(|_| {
            ui.set_widgets(|ui| deep_ui.set_widgets(ui));
        });
        event.render(|args| {
            gl.draw(args.viewport(), |c, gl| {
                ui.draw(c, gl);
            });
        });
    }
}
