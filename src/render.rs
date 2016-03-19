extern crate glium;
extern crate clap; 
extern crate image;
extern crate cgmath;
extern crate time;

use glium::backend::glutin_backend::GlutinFacade;
use clap::ArgMatches;
use std::f32;

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
            glium::texture::SrgbTexture2d::new(display, im).unwrap()
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

        Renderer { program: prog, background: bg, buffers: bufs,
            params: RenderParams::new(args) }
    }

    pub fn render(&self, display: &GlutinFacade, t: f32) {
        use glium::Surface;
        use time::precise_time_ns;

        let start = precise_time_ns();

        let mut target = display.draw();

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
            let distance = 30.0f32; /* 2 * 15R_s, deflection is minimal by then */
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

struct ShaderPair {
    vert_shader: &'static str,
    frag_shader: &'static str,
}

const FLAT_SHADER: ShaderPair = ShaderPair {
    vert_shader: r#"

#version 140

in vec2 pos;
out vec3 dir;
out vec2 pos_v;

uniform float height_ratio; // height / width
uniform float fov_ratio; // tan(fov / 2)

uniform mat3 facing;

void main() {
    float x = pos.x * fov_ratio;
    float y = pos.y * fov_ratio * height_ratio;
    dir = facing * vec3(x, y, 1.0);
    pos_v = pos;

    gl_Position = vec4(pos, 0.0, 1.0);
}

    "#,
    frag_shader: r#"

#version 140

#define M_PI 3.1415926535897932384626433832795

in vec3 dir;

out vec4 color;

uniform sampler2D bgtex;
uniform int NUM_ITER;
uniform float TIME_STEP;

/* we set constants to convenient values for now */
const float C = 1.0;
const float R_s = 1.0;
const float M = 0.5; /* must be R_s / 2 */
const float G = 1.0;

uniform bool FLAT;

uniform vec3 src;

float atan2(float y, float x) {
    return x == 0.0 ? sign(y) * M_PI / 2 : atan(y, x);
}

float yaw(vec3 v) {
    return atan2(v.x, v.z);
}

float yaw_coord(vec3 v) {
    return (yaw(v) + M_PI) / (2. * M_PI);
}

float pitch(vec3 v) {
    return asin(v.y);
}

float pitch_coord(vec3 v) {
    return (pitch(v) + M_PI / 2.) / M_PI;
}

vec4 bg_tex(vec3 dir) {
    float x = yaw_coord(dir);
    float y = pitch_coord(dir);

    vec2 tex_coords = vec2(x, y);

    float invert_x = x - 0.5;
    invert_x = invert_x - sign(invert_x) * 0.5;

    vec2 dx1 = dFdx(tex_coords);
    vec2 dx2 = dFdx(vec2(invert_x, y));

    vec2 dx = dot(dx1, dx1) < dot(dx2, dx2) ? dx1 : dx2;

    /* force the LOD so that GLSL doesn't flip out on the discontinuity
       at the texture border */
    return textureGrad(bgtex, tex_coords, dx, dFdy(tex_coords));
}

const vec4 ZERO = vec4(0.0, 0.0, 0.0, 0.0);

void main() {
    vec3 ndir = normalize(dir);

    /* closest approach to BH */
    float dist = length(cross(ndir, src));

    float alpha_rem = 1.0;
    vec4 ccolor = vec4(0.0, 0.0, 0.0, 0.0);
    vec3 cdir = ndir; /* current direction */
    //if(dist < 3) {
        vec3 pos = src;
        vec3 h = cross(pos, cdir);
        float h2 = dot(h, h); /* angular momentum */


        for(int i = 0; i < NUM_ITER; i++) {
            vec3 npos = pos + cdir * TIME_STEP;
            if(!FLAT) {
                vec3 accel = -pos * 1.5 * h2 * pow(dot(pos, pos), -2.5);
                cdir = cdir + accel * TIME_STEP;
                if(0 == i % 100) {
                    cdir = normalize(cdir);
                    h = cross(pos, cdir);
                    h2 = dot(h, h);
                }
                //cdir = cdir + accel * TIME_STEP;
            }

            /* check if its within a black hole */
            if(length(npos) <= R_s) {
                ccolor += alpha_rem * vec4(0.0, 0.0, 1.0, 1.0);
                alpha_rem *= 0.0;
            }

            pos = npos;
        }
    //}
    cdir = normalize(cdir);

    ccolor += alpha_rem * bg_tex(cdir);

    color = ccolor;
}

    "#,
};

/*const GR_SHADER: ShaderPair = ShaderPair {
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
};*/

