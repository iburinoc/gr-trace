extern crate clap;
extern crate glium;

use clap::ArgMatches;

pub struct Shader {
    vert_shader: String,
    frag_shader: String,
}

impl Shader {
    pub fn construct(args: &ArgMatches) -> Self {
        Shader { vert_shader: Shader::construct_vert_shader(args),
                 frag_shader: Shader::construct_frag_shader(args) }
    }

    fn construct_vert_shader(args: &ArgMatches) -> String {
        DEFAULT_VERT_SHADER.to_string()
    }

    fn construct_frag_shader(args: &ArgMatches) -> String {
        frag_shader::gen_shader(args)
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
out vec3 dir_v;
out vec2 pos_v;

uniform float height_ratio; // height / width
uniform float fov_ratio; // tan(fov / 2)

uniform mat3 facing;

void main() {
    float x = pos.x * fov_ratio;
    float y = pos.y * fov_ratio * height_ratio;
    dir_v = facing * vec3(x, y, 1.0);
    pos_v = pos;

    gl_Position = vec4(pos, 0.0, 1.0);
}

"#;

#[allow(unused_variables)]
mod frag_shader {
    use clap::ArgMatches;
    pub fn gen_shader(args: &ArgMatches) -> String {
        format!(r#"
{preamble}

{bg_func}

{params}

void main() {{
    float alpha_rem = 1.0;
    vec4 ccolor = vec4(0.0, 0.0, 0.0, 0.0);
    vec3 dir = normalize(dir_v);
    vec3 pos = src;

    /* closest approach to BH */
    float min_dist = length(cross(dir, src));

    {loop_vars}

    {trace_vars}

    {loop_cond} {{
        vec3 npos, ndir;

        {update_func}

        {bh_check}
        {ad_check}

        pos = npos;
        dir = ndir;
    }}

    ccolor += alpha_rem * bg_col(dir);

    color = ccolor;
}}

    "#,
        
        preamble = PREAMBLE,
        bg_func = bg::func(args),
        params = trace::params(args),
        loop_vars = iter::vars(args),
        trace_vars = trace::vars(args),
        loop_cond = iter::cond(args),
        update_func = trace::update(args),
        bh_check = bh::check(args),
        ad_check = ad::check(args))
    }

    const PREAMBLE: &'static str = r#"
#version 330

#define M_PI (3.1415926535897932384626433832795)
/* we set constants to convenient values for now */
const float C = 1.0;
const float R_s = 1.0;
const float M = 0.5; /* must be R_s / 2 */
const float G = 1.0;

uniform vec3 src;

in vec3 dir_v;
in vec3 pos_v;
out vec4 color;

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

float ts_func(float ts, vec3 pos) {
    //float r2 = dot(pos,pos);
    return ts;
    //return ts * clamp(r2 / 2.25, 1.0, 10.0);
}
"#;

    mod bg {
        use clap::ArgMatches;
        enum Type {
            Black,
            Texture,
        }

        pub fn func(args: &ArgMatches) -> String {
            let s = args.value_of("bg").unwrap_or("img");
            BGS[(match s {
                "img" => Type::Texture,
                "black" => Type::Black,
                _ => panic!("Invalid bg type"),
            }) as usize].to_string()
        }

        const BGS: [&'static str; 2] = [
        r#"
vec4 bg_col(vec3 dir) {
    return vec4(0.0, 0.0, 0.0, 1.0);
}"#,
            r#"

uniform sampler2D bgtex;
vec4 bg_col(vec3 dir) {
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
}"#,
        ];
    }

    mod iter {
        use clap::ArgMatches;
        pub fn vars(args: &ArgMatches) -> String {
            "".to_string()
        }

        pub fn cond(args: &ArgMatches) -> String {
            r#"while(dot(pos, pos) <= 15.0 * 15.0 &&
                dot(pos, pos) >= 0.1 &&
                alpha_rem >= 0.01)"#.to_string()
        }
    }

    mod trace {
        use clap::ArgMatches;
        
        enum Type {
            Flat = 0,
            Verlet = 1,
        }

        fn get_type(args: &ArgMatches) -> Type {
            if args.is_present("flat") {
                Type::Flat
            } else {
                match args.value_of("scheme").unwrap_or("verlet") {
                    "verlet" => Type::Verlet,
                    s => panic!("invalid integration scheme: {}", s),
                }
            }
        }

        pub fn params(args: &ArgMatches) -> String {
            PARAMS[get_type(args) as usize].to_string()
        }

        pub fn update(args: &ArgMatches) -> String {
            UPDATES[get_type(args) as usize].to_string()
        }

        pub fn vars(args: &ArgMatches) -> String {
            VARS[get_type(args) as usize].to_string()
        }

        const VARS: [&'static str; 2] = [
            r#"
            float time_step;
            "#,
            r#"
            float time_step;
            vec3 h = cross(pos, dir);
            float h2 = dot(h, h);
            "#,
        ];

        const PARAMS: [&'static str; 2] = [
            r#"
            uniform float TIME_STEP;
        "#,
            r#"
            uniform float TIME_STEP;
        "#,
        ];

        const UPDATES: [&'static str; 2] = [
            r#"
            time_step = ts_func(TIME_STEP, pos);
            npos = pos + dir * time_step;
            ndir = dir;
        "#,
            r#"
            time_step = ts_func(TIME_STEP, pos);
            npos = pos + dir * time_step;
            vec3 accel = -pos * 1.5 * h2 * pow(dot(pos, pos), -2.5);
            ndir = dir + accel * time_step;
            if(dot(ndir, ndir) > 100.0) {
                /* experimental renormalization */
                ndir = normalize(ndir);
                h = cross(ndir, npos);
                h2 = dot(h, h);
            }
        "#,
        ];
    }

    mod bh {
        use clap::ArgMatches;
        pub fn check(args: &ArgMatches) -> String {
            FLAT_BH.to_string()
        }

        const FLAT_BH: &'static str = r#"
            float mindist2;
            {
                vec3 c = cross(npos, pos);
                vec3 d = pos - npos;
                mindist2 = dot(c, c) / dot(d, d);
            }
            //if(dot(npos, npos) <= R_s * R_s && dot(pos, pos) >= R_s * R_s) {
            if(mindist2 <= R_s * R_s) {
                ccolor += vec4(0.0, 0.0, 0.0, 1.0) * alpha_rem * 1.0;
                alpha_rem -= alpha_rem * 1.0;
            }
        "#;
    }

    mod ad {
        use clap::ArgMatches;
        pub fn check(args: &ArgMatches) -> String {
            NO_DISK.to_string()
        }

        const NO_DISK: &'static str = "";
    }

const DEFAULT_FRAG_SHADER: &'static str = r#"
#version 330

#define M_PI 3.1415926535897932384626433832795

in vec3 dir;

out vec4 color;

uniform sampler2D bgtex;
uniform int NUM_ITER;
uniform float TIME_STEP;


uniform bool FLAT;

uniform vec3 src;

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

}
/*const GR_SHADER: Shader = Shader {
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

