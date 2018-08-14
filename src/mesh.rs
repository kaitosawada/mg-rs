use gfx::{self, traits::*, pso};

pub struct Geometry<V: Pod> {
    pub vertices: Vec<V>,
    pub indices: Vec<u32>,
}