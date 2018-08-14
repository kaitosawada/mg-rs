extern crate gfx_device_gl;

use cgmath;
use gfx::{self, traits::*};
use piston_window::{self, Window};
use super::world;
use shader_version::Shaders;
use shader_version::glsl::GLSL;
use camera_controllers::{CameraPerspective};
use material;

pub struct RenderContext{
    pub factory: gfx_device_gl::Factory,
    pub projection: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
    pub point_lights: Vec<material::PointLight>,
    pub output_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, gfx::format::Srgba8>,
    pub output_stencil: gfx::handle::DepthStencilView<gfx_device_gl::Resources, gfx::format::DepthStencil>,
}

impl RenderContext {
    pub fn new(window: &piston_window::PistonWindow) -> Self {
        let factory = window.factory.clone();
        let width = window.draw_size().width;
        let height = window.draw_size().height;

        let projection: cgmath::Matrix4<f32> = cgmath::Matrix4::from(CameraPerspective {
            fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
            aspect_ratio: (width as f32) / (height as f32)
        }.projection());

        let opengl = piston_window::OpenGL::V3_2;
        let glsl = opengl.to_glsl();

        RenderContext {
            factory,
            projection,
            view: cgmath::Matrix4::from_scale(1.0),
            point_lights: Vec::new(),
            output_color: window.output_color.clone(),
            output_stencil: window.output_stencil.clone(),
        }
    }

    pub fn resize(&mut self, window: &piston_window::PistonWindow, x: u32, y: u32) {
        self.projection = cgmath::Matrix4::from(CameraPerspective {
            fov: 90.0, near_clip: 0.1, far_clip: 1000.0,
            aspect_ratio: (x as f32) / (y as f32)
        }.projection());
        self.output_color = window.output_color.clone();
        self.output_stencil = window.output_stencil.clone();
    }

    pub fn draw_world<T: world::World>(&mut self, world: &mut T, window: &mut piston_window::PistonWindow, dt: f64) {
        window.encoder.clear(&window.output_color, [0.3, 0.3, 0.3, 1.0]);
        window.encoder.clear_depth(&window.output_stencil, 1.0);
        world.draw(self, &mut window.encoder, dt);
    }

    pub fn set_view(&mut self, view: cgmath::Matrix4<f32>) {
        self.view = view;
    }
}