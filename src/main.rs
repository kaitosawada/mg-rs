extern crate piston_window;
extern crate camera_controllers;
extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate shader_version;
extern crate rust_fluid;
#[macro_use]
extern crate gfx_macros;
#[macro_use]
extern crate conrod;
extern crate gfx_device_gl;

use piston_window::*;
use camera_controllers::{
    FirstPersonSettings,
    FirstPerson,
};
use world::World;
use std::path::Path;
use std::fs::File;

mod world;
mod render;
mod entity;
mod material;
mod mesh;
mod ui_handler;

fn main() {
    const WIDTH: u32 = 1024;
    const HEIGHT: u32 = 768;
    let mut window: PistonWindow =
        WindowSettings::new("piston: cube", [WIDTH, HEIGHT])
            .exit_on_esc(true)
            .samples(4)
            .opengl(OpenGL::V3_2)
            .build()
            .unwrap();
    let mut capture = true;
    window.set_capture_cursor(true);

    let mut ui = ui_handler::UIHandler::new(&mut window, WIDTH, HEIGHT);

    let mut ctx = render::RenderContext::new(&window);
    let mut first_person = FirstPerson::new(
        [0.5, 0.5, 1.0],
        FirstPersonSettings::keyboard_wasd(),
    );
    first_person.velocity = 2.0;

    let mut world = world::BasicWorld::new();
    let fluid = entity::entity_fluid::FluidEntity::new(&mut ctx, 300, 300, 2.0);
    world.add_entity(Box::new(fluid));
    let mut obj1 =
        entity::entity_obj::EntityObj::from_obj(
            &mut ctx,
            "Eames_chair_DSW/Eames_chair_DSW.obj");
    obj1.set_pos(cgmath::Vector3::new(1.0, 0.5, 2.0));
    world.add_entity(Box::new(obj1));

    while let Some(e) = window.next() {
        if capture {
            first_person.event(&e);
        }
        ui.update(&mut window, &e);
        use piston_window::Event::*;
        use piston_window::Loop::*;
        use piston_window::Input::*;
        use piston_window::Key;
        use piston_window::Button::*;
        match e {
            Loop(Render(RenderArgs { ext_dt, .. })) => {
                ctx.set_view(cgmath::Matrix4::from(first_person.camera(ext_dt).orthogonal()));
                window.draw_3d(&e, |window| {
                    ctx.draw_world(&mut world, window, ext_dt);
                });
            }
            Loop(Update(UpdateArgs { dt })) => {
                world.update(dt)
            }
            Input(Resize(x, y)) => {
                ctx.resize(&window, x, y);
            }
            Input(Button(ButtonArgs {
                             state,
                             button,
                             ..
                         })) => {
                match button {
                    Keyboard(key) => {
                        match key {
                            Key::E => {
                                if state == ButtonState::Release {
                                    capture = !capture;
                                    window.set_capture_cursor(capture);
                                }
                            }
                            _ => ()
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}