extern crate glium;
extern crate clap; 
extern crate image;
extern crate cgmath;
extern crate time;

use glium::backend::Facade;
use glium::{Surface, Frame};
use clap::ArgMatches;
use std::f32;

use shaders::Shader;

use ::Camera;

struct RenderParams {
    iter: i32,
    time_step: f32,
}

struct RenderBuffers(glium::VertexBuffer<RayVertex>, glium::IndexBuffer<u8>);
pub struct Renderer {
    program: glium::Program,
    background: glium::texture::SrgbTexture2d,
    disk: glium::texture::SrgbTexture2d,

    buffers: RenderBuffers,

    params: RenderParams,
}

impl Renderer {
    pub fn new<F>(display: &F, args: &ArgMatches) -> Self
                  where F: Facade {
        let bg = {
            use std::io::Cursor;
            //FIXME: insert alternate bg images here

            let bytes = if cfg!(debug_assertions) {
                &include_bytes!("../resources/bg-small.jpg")[..]
            } else {
                &include_bytes!("../resources/bg.jpg")[..]
            };
            let im = image::load(Cursor::new(bytes),
                                 image::JPEG).unwrap().to_rgba();

            let imdim = im.dimensions();
            let im = glium::texture::RawImage2d::from_raw_rgba_reversed(
                        im.into_raw(), imdim);
            glium::texture::SrgbTexture2d::new(display, im).unwrap()
        };

        let ad = {
            use std::io::Cursor;

            let bytes = &include_bytes!("../resources/ad.jpg")[..];
            let im = image::load(Cursor::new(bytes),
                                 image::JPEG).unwrap().to_rgba();

            let imdim = im.dimensions();
            let im = glium::texture::RawImage2d::from_raw_rgba_reversed(
                        im.into_raw(), imdim);
            glium::texture::SrgbTexture2d::new(display, im).unwrap()
        };

        let prog = Shader::construct(args).compile(display);

        let bufs = {
            use glium::index::PrimitiveType::TrianglesList;
            RenderBuffers(glium::VertexBuffer::new(display, &VERTICES).unwrap(),
             glium::IndexBuffer::new(display, TrianglesList, &INDICES).unwrap())
        };

        Renderer { program: prog, background: bg, disk: ad,
            buffers: bufs, params: RenderParams::new(args) }
    }

    pub fn render(&self, mut target: Frame, camera: &Camera) {

        target.clear_color(0., 0., 0., 0.0);

        let (width, height) = target.get_dimensions();

/*        let src = (-1.2 * t.sin(),0.0,-1.2 * t.cos());
        let facing_mat = {
            use cgmath::*;
            let dir = vec3(t.cos(), 0.0f32, t.sin());
            let up = vec3(0.,1.,0.0f32);

            // cgmath returns a tranposed look_at matrix for some reason
            Into::<[[f32;3];3]>::into(cgmath::Matrix3::look_at(dir, up)
                .transpose())
        };
*/

        let (src, facing_mat) = {
            use cgmath::Matrix;

            let src = Into::<[f32;3]>::into(camera.pos);
            let facing_mat = Into::<[[f32;3];3]>::into(camera.facing.transpose());

            (src, facing_mat)
        };
        let uniforms = uniform! {
            height_ratio: (height as f32) / (width as f32),
            fov_ratio: (f32::consts::PI * 2. / 3. / 2.).tan(), // pi/2, 90 deg
            src: src,
            facing: facing_mat,
            bg_tex: self.background
                .sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Repeat),
            ad_tex: self.disk
                .sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Clamp),
            NUM_ITER: self.params.iter,
            TIME_STEP: self.params.time_step,
        };

        let params = glium::DrawParameters {
            blend: glium::Blend {
                color: glium::BlendingFunction::AlwaysReplace,
                alpha: glium::BlendingFunction::AlwaysReplace,
                constant_value: (0.0, 0.0, 0.0, 0.0),
            },
            .. Default::default()
        };

        target.draw(&self.buffers.0, &self.buffers.1, &self.program,
                    &uniforms, &params).unwrap();

        target.finish().unwrap();
    }
}

impl RenderParams {
    fn new(args: &ArgMatches) -> Self {
        RenderParams {
            iter: args.value_of("iter").unwrap().parse::<i32>().unwrap(),
            time_step: args.value_of("timestep").unwrap().parse::<f32>().unwrap(),
        }
    }
}

#[derive(Copy, Clone)]
struct RayVertex {
    pos: (f32, f32),
}

implement_vertex!(RayVertex, pos);

const VERTICES: [RayVertex; 4] = [
    RayVertex { pos: (-1., -1.) },
    RayVertex { pos: (-1.,  1.) },
    RayVertex { pos: ( 1.,  1.) },
    RayVertex { pos: ( 1., -1.) },
];

const INDICES: [u8; 6] = [
    0, 1, 2,
    0, 2, 3
];

