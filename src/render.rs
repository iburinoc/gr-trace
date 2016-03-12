extern crate glium;
extern crate clap; 
extern crate image;

use glium::backend::glutin_backend::GlutinFacade;
use clap::ArgMatches;

struct RenderBuffers(glium::VertexBuffer<RayVertex>, glium::IndexBuffer<u8>);
pub struct Renderer {
    program: glium::Program,
    background: glium::texture::Texture2d,

    buffers: RenderBuffers,
}

impl Renderer {
    pub fn new(display: &GlutinFacade, args: &ArgMatches) -> Self {
        let bg = {
            use std::io::Cursor;
            //FIXME: insert alternate bg images here

            let im = image::load(
                        Cursor::new(&include_bytes!("../resources/bg.jpg")[..]),
                        image::JPEG).unwrap().to_rgba();

            let imdim = im.dimensions();
            let im = glium::texture::RawImage2d::from_raw_rgba_reversed(
                        im.into_raw(), imdim);
            glium::texture::Texture2d::new(display, im).unwrap()
        };

        let prog = {
            //FIXME: add option for different shaders
            let shaders = &FLAT_SHADER;

            glium::Program::from_source(display,
                shaders.vert_shader,
                shaders.frag_shader,
                None).unwrap() 
        };

        let bufs = {
            use glium::index::PrimitiveType::TrianglesList;
            RenderBuffers(glium::VertexBuffer::new(display, &VERTICES).unwrap(),
             glium::IndexBuffer::new(display, TrianglesList, &INDICES).unwrap())
        };

        Renderer { program: prog, background: bg, buffers: bufs }
    }

    pub fn render(&self, display: &GlutinFacade) {
        use glium::Surface;

        let mut target = display.draw();

        target.clear_color(0., 0., 0., 1.0);

        target.draw(&self.buffers.0, &self.buffers.1, &self.program,
                    &glium::uniforms::EmptyUniforms,
                    &Default::default()).unwrap();

        target.finish().unwrap();
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

struct ShaderPair {
    vert_shader: &'static str,
    frag_shader: &'static str,
}

const FLAT_SHADER: ShaderPair = ShaderPair {
    vert_shader: r#"

#version 140

in vec2 pos;
out vec2 pos_v;

void main() {
    pos_v = pos;

    gl_Position = vec4(pos, 0.0, 1.0);
}

    "#,
    frag_shader: r#"

#version 140

in vec2 pos_v;

out vec4 color;

void main() {
    float red = (pos_v.x + 1) / 2.;
    float green = (pos_v.y + 1) / 2.;
    color = vec4(red, green, 0.0, 1.0);
}

    "#,
};

const GR_SHADER: ShaderPair = ShaderPair {
    vert_shader: r#"

#version 140

in vec2 pos;
out vec2 pos_v;

void main() {
    pos_v = pos;

    gl_Position = vec4(pos, 0.0, 1.0);
}

    "#,
    frag_shader: r#"

#version 140

in vec2 pos_v;

out vec4 color;

void main() {
    float red = (pos_v.x + 1) / 2.;
    float green = (pos_v.y + 1) / 2.;
    color = vec4(red, green, 0.0, 1.0);
}

    "#,
};

