extern crate genmesh;
extern crate obj;

use std;
use std::path::{Path, PathBuf};
use cgmath;
use entity;
use material::{self, Vertex, VertexTexture};
use gfx;
use mesh;
use render;
use piston_window;

pub struct Mesh<V: gfx::traits::Pod> {
    geometry: mesh::Geometry<V>,
    material: Box<material::MaterialTrait<V>>,
    model_view: cgmath::Matrix4<f32>,
}

pub struct EntityObj<V: gfx::traits::Pod> {
    position: cgmath::Vector3<f32>,
    model_view: cgmath::Matrix4<f32>,
    parts: Vec<Mesh<V>>,
}

impl EntityObj<VertexTexture> {
    pub fn from_obj(ctx: &mut render::RenderContext, name: &str) -> Self {
        let path =
            Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/models"))
                .join(name);
        let parts = load_wavefront(ctx, &path.as_path());
        let position = cgmath::Vector3::new(0.0, 0.0, 0.0);
        let model_view = cgmath::Matrix4::from_scale(0.01);
        EntityObj {
            position,
            model_view,
            parts,
        }
    }

    pub fn set_pos(&mut self, pos: cgmath::Vector3<f32>) {
        self.position = pos;
        self.model_view.w = pos.extend(1.0);
    }
}

impl<V: gfx::traits::Pod> entity::Entity for EntityObj<V> {
    fn update(&mut self, dt: f64) {}

    fn draw(&mut self, ctx: &mut render::RenderContext, encoder: &mut piston_window::GfxEncoder, dt: f64) {
        for part in self.parts.iter_mut() {
            part.material.draw(
                ctx,
                encoder,
                &part.geometry,
                self.model_view * part.model_view,
            );
        }
    }
}

pub fn convert_material(
    ctx: &mut render::RenderContext,
    m: &std::borrow::Cow<obj::Material>,
    path: &Path,
) -> Result<material::MaterialPbrTex, gfx::PipelineStateError<String>> {
    material::MaterialPbrTex::new(
        ctx,
        0.5,
        0.5,
        m.kd.unwrap_or([1.0, 1.0, 1.0]),
        [0.0, 0.0, 0.0],
        1.0,//m.d.unwrap_or(1.0 - m.tr.unwrap_or(0.0)),
        gfx::Primitive::TriangleList,
        m.map_kd.clone(),
        path,
    )
}

pub fn load_wavefront(ctx: &mut render::RenderContext, path: &Path) -> Vec<Mesh<VertexTexture>> {
    use self::genmesh::MapToVertices;
    let mut data: obj::Obj<obj::SimplePolygon> = obj::Obj::load(&path).unwrap();
    data.load_mtls();

    let mut vertex_data = Vec::new();

    for o in &data.objects {
        for g in &o.groups {
            let mut ofst = 0;
            let mut i: Vec<u32> = Vec::new();
            let p: Vec<VertexTexture> = g.polys
                .iter()
                .flat_map(|x: &obj::SimplePolygon| {
                    let v: Vec<material::VertexTexture> =
                        x.iter()
                            .map(|obj::IndexTuple(p, t, n)| {
                                material::VertexTexture::new(
                                    data.position[*p],
                                    t.map_or([0., 0.], |t| data.texture[t]),
                                    n.map_or([1., 0., 0.], |n| data.normal[n]))
                            })
                            .collect();
                    for ii in 0..v.len() as u32 - 2 {
                        i.push(ofst);
                        i.push(ofst + ii + 1);
                        i.push(ofst + ii + 2);
                    }
                    ofst += v.len() as u32;
                    v
                })
                .collect();
            let geometry = mesh::Geometry {
                vertices: p,
                indices: i,
            };
            //println!("{}", g.material.clone().unwrap().ni.unwrap_or(-1.0));
            let material =
                g.material
                    .clone()
                    .map(|x| { convert_material(ctx, &x, &data.path.as_path()).unwrap() })
                    .unwrap();
            let mesh = Mesh {
                geometry,
                material: Box::new(material),
                model_view: cgmath::Matrix4::from_scale(1.0),
            };
            vertex_data.push(mesh);
        }
    }
    vertex_data
}