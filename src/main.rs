#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate clap;
extern crate image;
extern crate time;

use glium::glutin::{EventsLoop, VirtualKeyCode, WindowBuilder};

use std::collections::HashSet;

use time::precise_time_ns;

use clap::{App, Arg, ArgMatches};

use cgmath::{vec3, Euler, Matrix3, Vector3};

use std::fmt;

mod render;
mod shaders;

#[allow(dead_code)]
mod settings {
    pub const NAME: &'static str = "gr_trace";
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

fn main() {
    let args = arg_handle();
    let mut events_loop = EventsLoop::new();
    let window = build_display();
    let context = glium::glutin::ContextBuilder::new();

    let display = glium::Display::new(window, context, &events_loop).unwrap();

    let renderer = render::Renderer::new(&display, &args);

    let mut camera = Camera {
        pos: vec3(0.0, 0.0, -10.0f32),
        facing: Matrix3::look_at(vec3(0., 0., 1.), vec3(0., 1., 0.)),
    };

    let start = precise_time_ns();
    let mut prev = precise_time_ns();
    let mut keys = HashSet::new();
    loop {
        use time::precise_time_ns;

        let time = (precise_time_ns() - start) as f32 / 1000000000.0f32;
        renderer.render(display.draw(), &camera, time);
        display.finish();

        let time = precise_time_ns();
        let dt = (time - prev) as f32 / 1000000000.0f32;
        prev = time;

        let mut stop = false;
        events_loop.poll_events(|ev| {
            use glium::glutin::Event::WindowEvent;
            use glium::glutin::WindowEvent::*;
            match ev {
                WindowEvent { window_id: _, event } => match event {
                    CloseRequested => stop = true,
                    KeyboardInput { device_id: _, input } => {
                        if let Some(k) = input.virtual_keycode {
                            use glium::glutin::ElementState;
                            match input.state {
                                ElementState::Pressed => keys.insert(k),
                                ElementState::Released => keys.remove(&k),
                            };
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        });
        if stop {
            break;
        }

        camera.update(&keys, dt);
        println!("dt: {}ms cam: {}", dt * 1000.0f32, camera);

        if keys.contains(&VirtualKeyCode::Q) && keys.contains(&VirtualKeyCode::LWin) {
            break;
        }
    }
}

pub struct Camera {
    pos: Vector3<f32>,
    facing: Matrix3<f32>,
}

impl Camera {
    fn update(&mut self, keys: &HashSet<VirtualKeyCode>, dt: f32) {
        use cgmath::Rad ;
        use cgmath::SquareMatrix;
        use cgmath::Zero;

        let ang = Rad::<f32>(1f32 * dt);
        let mut dist = 0.2f32;

        let mut vert = 0.0;
        let mut hori = 0.0;
        let mut depth = 0.0;
        let mut yaw = Rad::zero();
        let mut pitch = Rad::zero();
        let mut roll = Rad::zero();
        for &k in keys {
            match k {
                VirtualKeyCode::W => vert += 1.0,
                VirtualKeyCode::S => vert -= 1.0,
                VirtualKeyCode::A => hori -= 1.0,
                VirtualKeyCode::D => hori += 1.0,
                VirtualKeyCode::Q => depth -= 1.0,
                VirtualKeyCode::E => depth += 1.0,
                VirtualKeyCode::LShift => dist = 2f32,
                VirtualKeyCode::I => pitch = pitch - ang,
                VirtualKeyCode::K => pitch = pitch + ang,
                VirtualKeyCode::J => yaw = yaw - ang,
                VirtualKeyCode::L => yaw = yaw + ang,
                VirtualKeyCode::U => roll = roll + ang,
                VirtualKeyCode::O => roll = roll - ang,
                _ => (),
            }
        }

        let mov = self.facing.invert().unwrap() * vec3(hori, vert, depth);
        let rot = Matrix3::from(Euler {x: pitch, y: yaw, z: roll});

        self.pos = self.pos + mov * dist * dt;
        self.facing = (self.facing.invert().unwrap() * rot).invert().unwrap();
    }
}

impl fmt::Display for Camera {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pos = self.pos;
        let fw = self.facing.z;
        let up = self.facing.y;
        write!(f, "pos: {:?} dir: {:?} up: {:?}", pos, fw, up)
    }
}

fn build_display() -> WindowBuilder {
    WindowBuilder::new()
        .with_dimensions(glium::glutin::dpi::LogicalSize::new(1024.0, 768.0))
        //.with_visibility(false)
        .with_title(format!("gr-trace"))
}

fn arg_handle<'a>() -> ArgMatches<'a> {
    App::new(settings::NAME)
        .version(settings::VERSION)
        .author("Sean Purcell <iburinoc@gmail.com>")
        .about("GPU General Relativity Ray Tracer")
        .arg(
            Arg::with_name("flat")
                .short("f")
                .long("flat")
                .help("Turns off relativistic distortion"),
        )
        .arg(
            Arg::with_name("iter")
                .short("i")
                .long("iter")
                .help("Sets the number of iterations to raytrace")
                .takes_value(true)
                .value_name("ITER_NUM")
                .default_value("1000"),
        )
        .arg(
            Arg::with_name("timestep")
                .short("t")
                .long("timestep")
                .help("Sets the length of each time step (where c = 1)")
                .takes_value(true)
                .value_name("TIME_STEP")
                .default_value("0.64"),
        )
        .arg(
            Arg::with_name("method")
                .short("m")
                .long("method")
                .help("Sets the integration method to use")
                .takes_value(true)
                .value_name("METHOD")
                .default_value("rk4")
                .possible_value("rk4")
                .possible_value("verlet")
                .possible_value("flat"),
        )
        .arg(
            Arg::with_name("bg")
                .short("b")
                .long("bg")
                .help("Sets the type of background used")
                .takes_value(true)
                .value_name("TYPE")
                .default_value("img")
                .possible_value("black")
                .possible_value("img"),
        )
        .arg(
            Arg::with_name("accdisk")
                .short("d")
                .long("disk")
                .help("Sets the type of accretion disk used")
                .takes_value(true)
                .value_name("TYPE")
                .default_value("dyno")
                .possible_value("none")
                .possible_value("white")
                .possible_value("img")
                .possible_value("dyno"),
        )
        .arg(
            Arg::with_name("iradius")
                .long("ir")
                .help("Sets the inner radius of the accretion disk")
                .takes_value(true)
                .value_name("RADIUS")
                .default_value("3"),
        )
        .arg(
            Arg::with_name("oradius")
                .long("or")
                .help("Sets the outer radius of the accretion disk")
                .takes_value(true)
                .value_name("RADIUS")
                .default_value("15"),
        )
        .arg(
            Arg::with_name("surface")
                .short("s")
                .long("surface")
                .help("Sets the type of black hole surface used")
                .takes_value(true)
                .value_name("SURFACE")
                .default_value("black")
                .possible_value("black")
                .possible_value("checkered"),
        )
        .arg(
            Arg::with_name("fov")
                .long("fov")
                .help("Sets the horizontal field of view (in degrees)")
                .takes_value(true)
                .value_name("FOV")
                .default_value("90"),
        )
        .arg(
            Arg::with_name("bgrat")
                .long("bgratio")
                .help("Sets the factor by which the background is dimmed")
                .takes_value(true)
                .value_name("FACTOR")
                .default_value("0.5"),
        )
        .arg(
            Arg::with_name("out")
                .short("O")
                .long("out")
                .help("Sets the output file")
                .value_name("FILE")
                .takes_value(true)
                .default_value("out.png"),
        )
        .get_matches()
}
/*
#[allow(dead_code)]
fn write_img(display: &GlutinFacade, path: &str) {
    let image: glium::texture::RawImage2d<u8> = display.read_front_buffer();
    let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image);
    let mut output = std::fs::File::create(&Path::new(path)).unwrap();
    image.save(&mut output, image::ImageFormat::PNG).unwrap();
}*/
