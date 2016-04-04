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

    #[allow(unused_variables)]
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

{trace_params}

{ad_params}

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
        trace_params = trace::params(args),
        ad_params = ad::params(args),
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

uniform sampler2D bg_tex;
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
    return textureGrad(bg_tex, tex_coords, dx, dy);
}"#,
        ];
    }

    mod iter {
        use clap::ArgMatches;
        pub fn vars(args: &ArgMatches) -> String {
            "".to_string()
        }

        pub fn cond(args: &ArgMatches) -> String {
            r#"float border = max(15.0 * 15.0, dot(src, src));
            while(dot(pos, pos) <= border &&
                alpha_rem >= 0.01)"#.to_string()
        }
    }

    mod trace {
        use clap::ArgMatches;
        
        enum Type {
            Flat = 0,
            Verlet = 1,
            Rk4 = 2,
        }

        fn get_type(args: &ArgMatches) -> Type {
            if args.is_present("flat") {
                Type::Flat
            } else {
                match args.value_of("method").unwrap_or("rk4") {
                    "verlet" => Type::Verlet,
                    "rk4" => Type::Rk4,
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

        const VARS: [&'static str; 3] = [
            r#"
            float time_step;
            "#,
            r#"
            float time_step;
            vec3 h = cross(pos, dir);
            float h2 = dot(h, h);
            "#,
            r#"
            float time_step;
            vec3 h = cross(pos, dir);
            float h2 = dot(h, h);
            "#,
        ];

        const PARAMS: [&'static str; 3] = [
            r#"
            uniform float TIME_STEP;
        "#,
            r#"
            uniform float TIME_STEP;
        "#,
            r#"
            uniform float TIME_STEP;

            vec3 accel(vec3 pos, float h2) {
                return -pos * 1.5 * h2 * pow(dot(pos, pos), -2.5);
            }
        "#,
        ];

        const UPDATES: [&'static str; 3] = [
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
            r#"
            time_step = ts_func(TIME_STEP, pos);
            {
                vec3 x1 = pos;
                vec3 v1 = dir;
                vec3 a1 = accel(x1, h2);

                vec3 x2 = pos + 0.5 * v1 * time_step;
                vec3 v2 = dir + 0.5 * a1 * time_step;
                vec3 a2 = accel(x2, h2);

                vec3 x3 = pos + 0.5 * v2 * time_step;
                vec3 v3 = dir + 0.5 * a2 * time_step;
                vec3 a3 = accel(x3, h2);

                vec3 x4 = pos + v3 * time_step;
                vec3 v4 = dir + a3 * time_step;
                vec3 a4 = accel(x4, h2);

                npos = pos + (time_step/6.0) * (v1 + 2*v2 + 2*v3 + v4);
                ndir = dir + (time_step/6.0) * (a1 + 2*a2 + 2*a3 + a4);
            }
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
            BH_CHECK.to_string()
        }

        const BH_CHECK: &'static str = r#"
            {
            float mindist2;
            float t;
            vec3 closest;
            {
                vec3 c = cross(npos, pos);
                vec3 d = pos - npos;
                mindist2 = dot(c, c) / dot(d, d);

                t = dot(pos, d) / dot(d, d);
                t = clamp(t, 0.0, 1.0);
                closest = pos + t * (npos - pos);

            }
            if(dot(closest, closest) <= R_s * R_s) {
                ccolor += vec4(0.0, 0.0, 0.0, 1.0) * alpha_rem * 1.0;
                alpha_rem -= alpha_rem * 1.0;
            }
            }
        "#;
    }

    mod ad {
        use clap::ArgMatches;
        enum Type {
            NoDisk = 0,
            White = 1,
            Tex = 2,
        }

        fn get_type(args: &ArgMatches) -> Type {
            match args.value_of("accdisk").unwrap_or("white") {
                "none" => Type::NoDisk,
                "white" => Type::White,
                "img" => Type::Tex,
                s => panic!("invalid accretion disk type: {}", s),
            }
        }

        pub fn check(args: &ArgMatches) -> String {
            CHECK.to_string()
        }

        pub fn params(args: &ArgMatches) -> String {
            let or: f32 = args.value_of("oradius").unwrap().parse().unwrap();
            let ir: f32 = args.value_of("iradius").unwrap().parse().unwrap();
            let extra = PARAMS[get_type(args) as usize].to_string();
            format!(r#"
                const float DISK_O_RAD = {};
                const float DISK_I_RAD = {};
                {}
                "#, or, ir, extra)
        }

        const CHECK: &'static str = r#"
            {
            float t = -pos.y / (npos.y - pos.y);
            if(t >= 0 && t <= 1) {
                vec3 p = pos + t * (npos - pos);
                float mag = length(p);
                if(mag >= DISK_I_RAD && mag <= DISK_O_RAD) {
                    vec4 col = ad_col(p, mag);
                    float rat = col.a;
                    ccolor += col * alpha_rem;
                    alpha_rem -= alpha_rem * rat;
                }
            }
            }
        "#;

        const PARAMS: [&'static str; 3] = [
            r#"
            vec4 ad_col(vec3 intersect, float mag) {
                return vec4(0.0, 0.0, 0.0, 0.0);
            }
        "#,
            r#"
            vec4 ad_col(vec3 intersect, float mag) {
                return vec4(1.0, 1.0, 1.0, 1.0);
            }
        "#,
            r#"
            uniform sampler2D ad_tex;
            vec4 ad_col(vec3 intersect, float mag) {
                float x = yaw_coord(intersect);
                float y = (DISK_O_RAD - mag) / (DISK_O_RAD - DISK_I_RAD);

                float invert_x = x - 0.5;
                invert_x = invert_x - sign(invert_x) * 0.5;

                vec2 c1 = vec2(x, y);
                vec2 c2 = vec2(invert_x, y);

                vec2 dx1 = dFdx(c1);
                vec2 dx2 = dFdx(c2);

                vec2 dy1 = dFdy(c1);
                vec2 dy2 = dFdy(c2);

                vec2 dx = dot(dx1, dx1) < dot(dx2, dx2) ? dx1 : dx2;
                vec2 dy = dot(dy1, dy1) < dot(dy2, dy2) ? dy1 : dy2;

                return textureGrad(ad_tex, c1, dx, dy);
                vec3 col = vec3(textureGrad(ad_tex, c1, dx, dy));
                float alpha = clamp(dot(col, col)/3.0, 0.0, 1.0);
                return vec4(col, alpha);
            }
        "#,
        ];
    }
}

