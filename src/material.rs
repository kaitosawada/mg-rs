extern crate image;

use std;
use gfx::{self, traits::*, pso};
use render;
use mesh::Geometry;
use piston_window;
use cgmath::{self, prelude::*};
use shader_version::Shaders;
use shader_version::glsl::GLSL;
use gfx_macros;
use gfx_device_gl;

#[derive(VertexData, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    pub fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Vertex {
            position,
            normal,
        }
    }
}

#[derive(VertexData, Clone, Copy)]
pub struct VertexTexture {
    pub position: [f32; 3],
    pub texture: [f32; 2],
    pub normal: [f32; 3],
}

impl VertexTexture {
    pub fn new(position: [f32; 3], texture: [f32; 2], normal: [f32; 3]) -> Self {
        VertexTexture {
            position,
            texture,
            normal,
        }
    }
}

gfx_pipeline!( pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    u_model_view_proj: gfx::Global<[[f32; 4]; 4]> = "u_model_view_proj",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "f_color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
    gfx::preset::depth::LESS_EQUAL_WRITE,
});

#[derive(ConstantBuffer, Clone, Copy)]
pub struct DirectionalLight {
    color: [f32; 4],
    direction: [f32; 4],
}

impl DirectionalLight {
    pub fn transform(&self, mat: cgmath::Matrix3<f32>) -> Self {
        DirectionalLight {
            direction: (mat * cgmath::Vector4::from(self.direction).truncate()).extend(0.0).into(),
            color: self.color,
        }
    }
}

#[derive(ConstantBuffer, Clone, Copy)]
pub struct PointLight {
    position: [f32; 4],
    color: [f32; 4],
    distance_decay: [f32; 4],
}

impl PointLight {
    pub fn transform(&self, mat: cgmath::Matrix4<f32>) -> Self {
        PointLight {
            position: (mat * cgmath::Vector4::from(self.position)).into(),
            color: self.color,
            distance_decay: self.distance_decay,
        }
    }
}

#[derive(ConstantBuffer, Clone, Copy)]
pub struct SpotLight {
    position: [f32; 4],
    direction: [f32; 4],
    color: [f32; 4],
    distance_decay_coneCos_penumbraCos: [f32; 4],
}

gfx_pipeline!( pipe_pbr {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    model_view_matrix: gfx::Global<[[f32; 4]; 4]> = "model_view_matrix",
    projection_matrix: gfx::Global<[[f32; 4]; 4]> = "projection_matrix",
    normal_matrix: gfx::Global<[[f32; 3]; 3]> = "normal_matrix",
    metallic: gfx::Global<f32> = "metallic",
    roughness: gfx::Global<f32> = "roughness",
    albedo: gfx::Global<[f32; 3]> = "albedo",
    emissive: gfx::Global<[f32; 3]> = "emissive",
    opacity: gfx::Global<f32> = "opacity",
    d_lights: gfx::ConstantBuffer<DirectionalLight> = "d_lights",
    p_lights: gfx::ConstantBuffer<PointLight> = "p_lights",
    s_lights: gfx::ConstantBuffer<SpotLight> = "s_lights",
    d_num: gfx::Global<i32> = "numDirectionalLights",
    p_num: gfx::Global<i32> = "numPointLights",
    s_num: gfx::Global<i32> = "numSpotLights",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "f_color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
    gfx::preset::depth::LESS_EQUAL_WRITE,
});

gfx_pipeline!( pipe_pbr_tex {
    vbuf: gfx::VertexBuffer<VertexTexture> = (),
    model_view_matrix: gfx::Global<[[f32; 4]; 4]> = "model_view_matrix",
    projection_matrix: gfx::Global<[[f32; 4]; 4]> = "projection_matrix",
    normal_matrix: gfx::Global<[[f32; 3]; 3]> = "normal_matrix",
    metallic: gfx::Global<f32> = "metallic",
    roughness: gfx::Global<f32> = "roughness",
    albedo: gfx::Global<[f32; 3]> = "albedo",
    has_kdmap: gfx::Global<i32> = "has_kdmap",
    emissive: gfx::Global<[f32; 3]> = "emissive",
    opacity: gfx::Global<f32> = "opacity",
    d_lights: gfx::ConstantBuffer<DirectionalLight> = "d_lights",
    p_lights: gfx::ConstantBuffer<PointLight> = "p_lights",
    s_lights: gfx::ConstantBuffer<SpotLight> = "s_lights",
    d_num: gfx::Global<i32> = "numDirectionalLights",
    p_num: gfx::Global<i32> = "numPointLights",
    s_num: gfx::Global<i32> = "numSpotLights",
    t_color: gfx::TextureSampler<[f32; 4]> = "t_color",
    out_color: gfx::RenderTarget<::gfx::format::Srgba8> = "f_color",
    out_depth: gfx::DepthTarget<::gfx::format::DepthStencil> =
    gfx::preset::depth::LESS_EQUAL_WRITE,
});

pub trait MaterialTrait<V: Pod> {
    fn draw(&mut self,
            ctx: &mut render::RenderContext,
            encoder: &mut piston_window::GfxEncoder,
            geometry: &Geometry<V>,
            model_matrix: cgmath::Matrix4<f32>,
    );
}

pub struct Material<D, R>
    where D: gfx::pso::PipelineData<R>,
          R: gfx::Resources,
{
    pso: gfx::PipelineState<R, D::Meta>,
    data: D,
}

pub type MaterialBasic = Material<pipe::Data<gfx_device_gl::Resources>, gfx_device_gl::Resources>;
pub type MaterialPbr = Material<pipe_pbr::Data<gfx_device_gl::Resources>, gfx_device_gl::Resources>;
pub type MaterialPbrTex = Material<pipe_pbr_tex::Data<gfx_device_gl::Resources>, gfx_device_gl::Resources>;

impl MaterialPbr {
    pub fn new(
        ctx: &mut render::RenderContext,
        metallic: f32,
        roughness: f32,
        albedo: [f32; 3],
        emissive: [f32; 3],
        opacity: f32,
        primitive: gfx::Primitive,
    ) -> Result<Self, gfx::PipelineStateError<String>> {
        let glsl = piston_window::OpenGL::V3_2.to_glsl();
        let set = ctx.factory.create_shader_set(
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../assets/pbr_150_vert.glsl"))
                .get(glsl).unwrap().as_bytes(),
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../assets/pbr_150_frag.glsl"))
                .get(glsl).unwrap().as_bytes(),
        )?;
        let pso = ctx.factory.create_pipeline_state(
            &set,
            primitive,
            gfx::state::Rasterizer::new_fill(),
            pipe_pbr::new(),
        )?;

        let vbuf = ctx.factory.create_vertex_buffer(&Vec::<Vertex>::new());
        let data = pipe_pbr::Data {
            vbuf: vbuf.clone(),
            model_view_matrix: cgmath::Matrix4::from_scale(1.0).into(),
            projection_matrix: cgmath::Matrix4::from_scale(1.0).into(),
            normal_matrix: cgmath::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0).into(),
            metallic,
            roughness,
            albedo,
            emissive,
            opacity,
            d_lights: ctx.factory.create_constant_buffer(4),
            p_lights: ctx.factory.create_constant_buffer(4),
            s_lights: ctx.factory.create_constant_buffer(4),
            d_num: 1,
            p_num: 0,
            s_num: 0,
            out_color: ctx.output_color.clone(),
            out_depth: ctx.output_stencil.clone(),
        };
        Ok(Material {
            pso,
            data,
        })
    }
}

impl MaterialTrait<Vertex> for MaterialPbr {
    fn draw(&mut self,
            ctx: &mut render::RenderContext,
            encoder: &mut piston_window::GfxEncoder,
            geometry: &Geometry<Vertex>,
            model_matrix: cgmath::Matrix4<f32>,
    ) {
        let (vbuf, slice) = ctx.factory.create_vertex_buffer_with_slice
        (&geometry.vertices, geometry.indices.as_slice());
        let mv_mat = ctx.view * model_matrix;
        let n_mat: cgmath::Matrix3<f32> =
            cgmath::Matrix3::from_cols(mv_mat.x.truncate(), mv_mat.y.truncate(), mv_mat.z.truncate())
                .transpose()
                .invert()
                .unwrap_or(cgmath::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0));

        self.data.vbuf = vbuf.clone();
        self.data.model_view_matrix = mv_mat.into();
        self.data.projection_matrix = ctx.projection.into();
        self.data.normal_matrix = n_mat.into();
        encoder.update_buffer(&self.data.d_lights, &[DIR_LIGHT.transform(n_mat)], 0);
        encoder.update_buffer(&self.data.p_lights, &[], 0);
        encoder.update_buffer(&self.data.s_lights, &[], 0);
        encoder.draw(&slice, &self.pso, &self.data);
    }
}

impl MaterialPbrTex {
    pub fn new(
        ctx: &mut render::RenderContext,
        metallic: f32,
        roughness: f32,
        albedo: [f32; 3],
        emissive: [f32; 3],
        opacity: f32,
        primitive: gfx::Primitive,
        map_kd: Option<std::string::String>,
        path: &std::path::Path,
    ) -> Result<Self, gfx::PipelineStateError<String>> {
        let glsl = piston_window::OpenGL::V3_2.to_glsl();
        let set = ctx.factory.create_shader_set(
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../assets/pbr_tex_150_vert.glsl"))
                .get(glsl).unwrap().as_bytes(),
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../assets/pbr_tex_150_frag.glsl"))
                .get(glsl).unwrap().as_bytes(),
        )?;
        let pso = ctx.factory.create_pipeline_state(
            &set,
            primitive,
            gfx::state::Rasterizer::new_fill(),
            pipe_pbr_tex::new(),
        )?;
        let default =
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/default.png"));
        let texture_view =
            load_texture(&mut ctx.factory,
                         map_kd
                             .clone()
                             .map(|x| { path.join(x) })
                             .unwrap_or(default),
            );

        let sinfo = gfx::texture::SamplerInfo::new(
            gfx::texture::FilterMethod::Bilinear,
            gfx::texture::WrapMode::Clamp);

        let vbuf = ctx.factory.create_vertex_buffer(&Vec::<VertexTexture>::new());

        let data = pipe_pbr_tex::Data {
            vbuf: vbuf.clone(),
            model_view_matrix: cgmath::Matrix4::from_scale(1.0).into(),
            projection_matrix: cgmath::Matrix4::from_scale(1.0).into(),
            normal_matrix: cgmath::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0).into(),
            metallic,
            roughness,
            albedo,
            has_kdmap: if map_kd.is_some() {1} else {0},
            emissive,
            opacity,
            d_lights: ctx.factory.create_constant_buffer(4),
            p_lights: ctx.factory.create_constant_buffer(4),
            s_lights: ctx.factory.create_constant_buffer(4),
            d_num: 0,
            p_num: 1,
            s_num: 0,
            t_color: (texture_view.clone(), ctx.factory.create_sampler(sinfo)),
            out_color: ctx.output_color.clone(),
            out_depth: ctx.output_stencil.clone(),
        };

        Ok(Material {
            pso,
            data,
        })
    }
}

impl MaterialTrait<VertexTexture> for MaterialPbrTex {
    fn draw(&mut self,
            ctx: &mut render::RenderContext,
            encoder: &mut piston_window::GfxEncoder,
            geometry: &Geometry<VertexTexture>,
            model_matrix: cgmath::Matrix4<f32>,
    ) {
        let (vbuf, slice) = ctx.factory.create_vertex_buffer_with_slice
        (&geometry.vertices, geometry.indices.as_slice());
        let mv_mat = ctx.view * model_matrix;
        let n_mat: cgmath::Matrix3<f32> =
            cgmath::Matrix3::from_cols(mv_mat.x.truncate(), mv_mat.y.truncate(), mv_mat.z.truncate())
                .transpose()
                .invert()
                .unwrap_or(cgmath::Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0));

        self.data.vbuf = vbuf.clone();
        self.data.model_view_matrix = mv_mat.into();
        self.data.projection_matrix = ctx.projection.into();
        self.data.normal_matrix = n_mat.into();
        encoder.update_buffer(&self.data.d_lights, &[DIR_LIGHT.transform(n_mat)], 0).unwrap();
        encoder.update_buffer(&self.data.p_lights, &[POINT_LIGHT.transform(mv_mat)], 0).unwrap();
        encoder.update_buffer(&self.data.s_lights, &[], 0).unwrap();
        encoder.draw(&slice, &self.pso, &self.data);
    }
}

const DIR_LIGHT: DirectionalLight = DirectionalLight {
    direction: [0.57735026919, -0.57735026919, 0.57735026919, 0.0],
    color: [1.0, 1.0, 1.0, 1.0],
};

const POINT_LIGHT: PointLight = PointLight {
    position: [0.0, -0.01, 0.01, 0.0],
    color: [1.0, 1.0, 1.0, 1.0],
    distance_decay: [10.0, 0.8, 0.0, 0.0],
};

impl MaterialBasic {
    pub fn new(ctx: &mut render::RenderContext) -> Result<Self, gfx::PipelineStateError<String>> {
        let glsl = piston_window::OpenGL::V3_2.to_glsl();
        let set = ctx.factory.create_shader_set(
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../assets/cube_150_vert.glsl"))
                .get(glsl).unwrap().as_bytes(),
            Shaders::new()
                .set(GLSL::V1_50, include_str!("../assets/cube_150_frag.glsl"))
                .get(glsl).unwrap().as_bytes(),
        )?;
        let pso = ctx.factory.create_pipeline_state(
            &set,
            gfx::Primitive::TriangleStrip,
            gfx::state::Rasterizer::new_fill(),
            pipe::new(),
        )?;
        let vbuf = ctx.factory.create_vertex_buffer(&Vec::<Vertex>::new());

        let data = pipe::Data {
            vbuf: vbuf.clone(),
            u_model_view_proj: cgmath::Matrix4::from_scale(1.0).into(),
            out_color: ctx.output_color.clone(),
            out_depth: ctx.output_stencil.clone(),
        };
        Ok(Material {
            pso,
            data,
        })
    }
}

impl MaterialTrait<Vertex> for MaterialBasic {
    fn draw(&mut self,
            ctx: &mut render::RenderContext,
            encoder: &mut piston_window::GfxEncoder,
            geometry: &Geometry<Vertex>,
            model_matrix: cgmath::Matrix4<f32>,
    ) {
        let (vbuf, slice) = ctx.factory.create_vertex_buffer_with_slice
        (&geometry.vertices, geometry.indices.as_slice());
        let matrix = ctx.projection * ctx.view * model_matrix;
        self.data.vbuf = vbuf;
        self.data.u_model_view_proj = matrix.into();
        encoder.draw(&slice, &self.pso, &self.data);
    }
}

fn load_texture(factory: &mut gfx_device_gl::Factory, path: std::path::PathBuf) -> gfx::handle::ShaderResourceView<gfx_device_gl::Resources, [f32; 4]> {
    let img = image::open(path.as_path()).unwrap().to_rgba();
    let (width, height) = img.dimensions();
    let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
    let mipmap = gfx::texture::Mipmap::Provided;
    let (_, view) = factory.create_texture_immutable_u8::<gfx::format::Srgba8>(kind, mipmap, &[&img]).unwrap();
    view
}