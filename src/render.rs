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

struct RenderParams {
    flat: bool,
    iter: i32,
}

struct RenderBuffers(glium::VertexBuffer<RayVertex>, glium::IndexBuffer<u8>);
pub struct Renderer {
    program: glium::Program,
    background: glium::texture::SrgbTexture2d,

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

        let prog = Shader::construct(args).compile(display);

        let bufs = {
            use glium::index::PrimitiveType::TrianglesList;
            RenderBuffers(glium::VertexBuffer::new(display, &VERTICES).unwrap(),
             glium::IndexBuffer::new(display, TrianglesList, &INDICES).unwrap())
        };

        Renderer { program: prog, background: bg, buffers: bufs,
            params: RenderParams::new(args) }
    }

    pub fn render(&self, mut target: Frame, t: f32) {
        use time::precise_time_ns;

        let start = precise_time_ns();

        target.clear_color(0., 0., 0., 0.0);

        let (width, height) = target.get_dimensions();

        let src = [-10.0 * t.sin(),0.0,-10.0 * t.cos()];
        let facing_mat = {
            use cgmath::*;
            let dir = vec3(t.sin(), 0.0f32, t.cos());
            let up = vec3(0.,1.,0.0f32);

            // cgmath returns a tranposed look_at matrix for some reason
            Into::<[[f32;3];3]>::into(cgmath::Matrix3::look_at(dir, up)
                .transpose())
        };

        let (num_iter, time_step) = {
            let distance = 15.0f32; /* 2 * 15R_s, deflection is minimal by then */
            let num_iter = self.params.iter; /* arbitrary */
            
            (num_iter, distance / (num_iter as f32))
        };

        let uniforms = uniform! {
            height_ratio: (height as f32) / (width as f32),
            fov_ratio: (f32::consts::PI * 2. / 3. / 2.).tan(), // pi/2, 90 deg
            src: src,
            facing: facing_mat,
            tex: self.background
                .sampled().wrap_function(glium::uniforms::SamplerWrapFunction::Repeat),
            NUM_ITER: num_iter,
            TIME_STEP: time_step,
            FLAT: self.params.flat,
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

        let end = precise_time_ns();
        println!("dt: {}ms", (end - start) as f32 / (1000000.0f32));
    }
}

impl RenderParams {
    fn new(args: &ArgMatches) -> Self {
        RenderParams {
            flat: args.is_present("flat"),
            iter: args.value_of("iter").unwrap().parse::<i32>().unwrap(),
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

