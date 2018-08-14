use render;
use piston_window;
use gfx;
use cgmath;

pub mod entity_obj;
pub mod entity_fluid;

pub trait Entity {
    fn update(&mut self, dt: f64);
    fn draw(&mut self, ctx: &mut render::RenderContext, encoder: &mut piston_window::GfxEncoder, dt: f64);
}