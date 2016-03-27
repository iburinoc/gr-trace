extern crate clap;
extern crate glium;

use clap::ArgMatches;

pub struct ShaderPair {
    vert_shader: String,
    frag_shader: String,
}

impl ShaderPair {
    pub fn construct(args: &ArgMatches) -> Self {
        ShaderPair { vert_shader: ShaderPair::construct_vert_shader(args),
                     frag_shader: ShaderPair::construct_frag_shader(args) }
    }

    fn construct_vert_shader(args: &ArgMatches) -> String {
        DEFAULT_VERT_SHADER.to_string()
    }

    fn construct_frag_shader(args: &ArgMatches) -> String {
        "".to_string()
    }

    pub fn compile<F>(self, display: &F) -> glium::Program
                  where F: glium::backend::Facade {
        let res = glium::Program::from_source(display,
                &self.vert_shader,
                &self.frag_shader,
                None);
        match res {
            Ok(t) => t,
            Err(e) => {
                panic!("{}", e);
            },
        }
    }
}

const DEFAULT_VERT_SHADER: &'static str = r#"

#version 330

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

"#;

const DEFAULT_FRAG_SHADER: &'static str = r#"

#version 330

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
    vec2 invert_coords = vec2(invert_x, y);

    vec2 dx1 = dFdx(tex_coords);
    vec2 dx2 = dFdx(invert_coords);

    vec2 dy1 = dFdy(tex_coords);
    vec2 dy2 = dFdy(invert_coords);

    vec2 dx = dot(dx1, dx1) < dot(dx2, dx2) ? dx1 : dx2;
    vec2 dy = dot(dy1, dy1) < dot(dy2, dy2) ? dy1 : dy2;

    /* force the LOD so that GLSL doesn't flip out on the discontinuity
       at the texture border */
    return textureGrad(bgtex, tex_coords, dx, dy);
}

const vec4 ZERO = vec4(0.0, 0.0, 0.0, 0.0);

const float TDIST = 20.0;
const float BTHRESHOLD = 0.9;
const float TTHRESHOLD = 0.95;
const float M_RAT = 8;

void main() {
    float cdist = 0.0;
    float ratio = 1.0;
    vec3 ndir = normalize(dir);

    /* closest approach to BH */
    float dist = length(cross(ndir, src));

    /* test iteration function */
    int num_iter = int((1 / (1 + dist * 0.2)) * NUM_ITER);

    float alpha_rem = 1.0;
    vec4 ccolor = vec4(0.0, 0.0, 0.0, 0.0);
    vec3 cdir = ndir; /* current direction */

    vec3 pos = src;
    vec3 h = cross(pos, cdir);
    float h2 = dot(h, h); /* angular momentum */

    while(cdist < TDIST) {
        vec3 npos = pos + cdir * TIME_STEP * ratio;
        cdist += TIME_STEP * ratio;
        if(!FLAT) {
            vec3 accel = -pos * 1.5 * h2 * pow(dot(pos, pos), -2.5);
            vec3 ncdir = cdir + accel * TIME_STEP * ratio;
            ncdir = normalize(ncdir);
            h = cross(pos, ncdir);
            h2 = dot(h, h);
            if(dot(ncdir, cdir) < BTHRESHOLD && ratio > (1/M_RAT)) {
                ratio *= 0.5;
            } else if(dot(ncdir, cdir) > TTHRESHOLD && ratio < M_RAT) {
                ratio *= 2;
            }
            cdir = ncdir;
        }

        /* check if its within a black hole */
        if(length(npos) <= R_s) {
            ccolor += alpha_rem * vec4(0.0, 0.0, 0.0, 1.0);
            alpha_rem *= 0.0;
        }

        pos = npos;
    }
/*
    for(int i = 0; i < num_iter; i++) {
        vec3 npos = pos + cdir * TIME_STEP;
        if(!FLAT) {
            vec3 accel = -pos * 1.5 * h2 * pow(dot(pos, pos), -2.5);
            cdir = cdir + accel * TIME_STEP;
            cdir = normalize(cdir);
            h = cross(pos, cdir);
            h2 = dot(h, h);
            //cdir = cdir + accel * TIME_STEP;
        }

        /* check if its within a black hole */
  /*      if(length(npos) <= R_s) {
            ccolor += alpha_rem * vec4(0.0, 0.0, 1.0, 1.0);
            alpha_rem *= 0.0;
        }

        pos = npos;
    }*/
    cdir = normalize(cdir);

    ccolor += alpha_rem * bg_tex(cdir);

    color = ccolor;
}

    "#;

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

