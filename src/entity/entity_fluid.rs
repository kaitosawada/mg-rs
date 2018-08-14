use rust_fluid::lbm::*;
use cgmath;
use cgmath::prelude::*;
use entity;
use piston_window;
use render::{self};
use mesh;
use material::{self, Vertex};
use gfx;
use std;

fn make_flatmesh(width: f32, height: f32, div_w: u32, div_h: u32) -> mesh::Geometry<material::VertexTexture> {
    let mut vertices = Vec::with_capacity((div_h + 1) as usize * (div_w + 1) as usize);
    let dw = width / div_w as f32;
    let dh = height / div_h as f32;
    for z0 in 0..div_h + 1 {
        for x0 in 0..div_w + 1 {
            vertices.push(material::VertexTexture::new(
                [x0 as f32 * dw - (width / 2.0), 0.0, z0 as f32 * dh - (height / 2.0), ],
                [0.0, 0.0],
                [0.0, 1.0, 0.0, ],
            ));
        }
    }
    let mut faces = Vec::with_capacity(div_h as usize * div_w as usize * 6);
    for z0 in 0..div_h {
        faces.push(z0 * (div_w + 1));
        for x0 in 0..div_w + 1 {
            let offset = x0 + z0 * (div_w + 1);
            faces.push(offset);
            faces.push(offset + div_w + 1);
        }
        faces.push((z0 + 2) * (div_w + 1) - 1);
    }
    //let indices = glium::index::IndexBuffer::new(display, glium::index::PrimitiveType::TriangleStrip, &faces).unwrap();
    mesh::Geometry { vertices, indices: faces }
}

pub struct FluidEntity {
    x: u32,
    y: u32,
    height_mag: f32,
    width: f32,
    state: LBMState,
    geometry: mesh::Geometry<material::VertexTexture>,
    material: Box<material::MaterialTrait<material::VertexTexture>>,
    model_view: cgmath::Matrix4<f32>,
    time: u64,
}

impl FluidEntity {
    pub fn new(ctx: &mut render::RenderContext, x: u32, y: u32, scale: f32) -> Self {
        let dx = scale / x as f32;
        let mut state = LBMState::new(x as usize + 1, y as usize + 1);
        let (vertices, indices) = test_data();
        let geometry = make_flatmesh(x as f32 * dx, y as f32 * dx, x, y);
        let material = Box::new(material::MaterialPbrTex::new(
            ctx,
            0.5,
            0.5,
            [0.8, 0.9, 1.0],
            [0.0, 0.0, 0.1],
            0.1,
            gfx::Primitive::TriangleStrip,
            None,
            &std::path::Path::new(""),
        ).unwrap());
        state.init();
        FluidEntity {
            x,
            y,
            height_mag: scale / 4.0,
            width: scale,
            state,
            geometry,
            material,
            model_view: cgmath::Matrix4::from_scale(1.0),
            time: 0,
        }
    }
}

impl entity::Entity for FluidEntity {
    fn update(&mut self, dt: f64) {
        self.time += 1;
        self.state.wave(self.time as f64 * 0.01);
        self.state.update();

        let dx = self.width / self.x as f32;

        for (mut item, &x) in self.geometry.vertices.iter_mut().zip(self.state.get().iter()) {
            item.position[1] = x as f32 * self.height_mag;
        }
        let (a1, a2) = nabla(&self.state.get(), dx as f64 / self.height_mag as f64);
        for ((mut item, &x1), &x2) in self.geometry.vertices.iter_mut().zip(a1.iter()).zip(a2.iter()) {
            item.normal = cgmath::Vector3::new(-x1 as f32, -1.0, -x2 as f32).normalize().into();
        }
    }

    fn draw(&mut self, ctx: &mut render::RenderContext, encoder: &mut piston_window::GfxEncoder, dt: f64) {
        self.material.draw(
            ctx,
            encoder,
            &self.geometry,
            self.model_view.clone(),
        );
    }
}

fn test_data() -> (Vec<Vertex>, Vec<u32>) {
    let vertex_data = vec![
        //top (0, 0, 1)
        Vertex::new([-1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        Vertex::new([1.0, -1.0, 1.0], [0.0, 0.0, 1.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 0.0, 1.0]),
        //bottom (0.0, 0.0, -1.0)
        Vertex::new([1.0, 1.0, -1.0], [0.0, 0.0, -1.0]),
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 0.0, -1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0]),
        Vertex::new([1.0, -1.0, -1.0], [0.0, 0.0, -1.0]),
        //right (1.0, 0.0, 0.0)
        Vertex::new([1.0, -1.0, -1.0], [1.0, 0.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [1.0, 0.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [1.0, 0.0, 0.0]),
        Vertex::new([1.0, -1.0, 1.0], [1.0, 0.0, 0.0]),
        //left (-1.0, 0.0, 0.0)
        Vertex::new([-1.0, 1.0, 1.0], [-1.0, 0.0, 0.0]),
        Vertex::new([-1.0, -1.0, 1.0], [-1.0, 0.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0]),
        Vertex::new([-1.0, 1.0, -1.0], [-1.0, 0.0, 0.0]),
        //front (0.0, 1.0, 0.0)
        Vertex::new([-1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, -1.0], [0.0, 1.0, 0.0]),
        Vertex::new([1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        Vertex::new([-1.0, 1.0, 1.0], [0.0, 1.0, 0.0]),
        //back (0.0, -1.0, 0.0)
        Vertex::new([1.0, -1.0, 1.0], [0.0, -1.0, 0.0]),
        Vertex::new([-1.0, -1.0, 1.0], [0.0, -1.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0]),
        Vertex::new([1.0, -1.0, -1.0], [0.0, -1.0, 0.0]),
    ];

    let index_data = vec![
        0, 1, 2, 2, 3, 0, // top
        4, 6, 5, 6, 4, 7, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 14, 13, 14, 12, 15, // left
        16, 18, 17, 18, 16, 19, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data, index_data)
}