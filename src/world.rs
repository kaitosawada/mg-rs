use entity;
use piston_window;
use render;

pub trait World {
    fn add_entity<T: 'static + entity::Entity>(&mut self, entity: Box<T>);
    fn draw(&mut self, renderer: &mut render::RenderContext, encoder: &mut piston_window::GfxEncoder, dt: f64);
    fn update(&mut self, dt: f64);
}

pub struct BasicWorld {
    entities: Vec<Box<entity::Entity>>
}

impl BasicWorld {
    pub fn new() -> Self {
        BasicWorld {
            entities: Vec::new(),
        }
    }
}

impl World for BasicWorld {
    fn add_entity<T: 'static + entity::Entity>(&mut self, entity: Box<T>) {
        self.entities.push(entity);
    }

    fn draw(&mut self, renderer: &mut render::RenderContext, encoder: &mut piston_window::GfxEncoder, dt: f64) {
        for i in self.entities.iter_mut() {
            i.draw(renderer, encoder, dt)
        }
    }

    fn update(&mut self, dt: f64) {
        for mut i in &mut self.entities {
            i.update(dt);
        }
    }
}