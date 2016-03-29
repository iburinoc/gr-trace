#[macro_use]
extern crate glium;
extern crate clap;
extern crate image;
extern crate cgmath;
extern crate time;

use glium::glutin::WindowBuilder;
use glium::DisplayBuild;
use glium::backend::glutin_backend::GlutinFacade;

use std::path::Path;

use clap::{Arg, App, ArgMatches};

mod render;
mod shaders;


#[allow(dead_code)]
mod settings {
    pub const NAME: &'static str = "gr-trace";
    pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
}

fn main() {
    let args = arg_handle();
    let display = build_display().build_glium().unwrap();

    let renderer = render::Renderer::new(&display, &args);

    let mut f: f32 = 0.2;//std::f32::consts::PI;
    loop {
        use time::precise_time_ns;

        let start = precise_time_ns();
        f += 0.002;
        renderer.render(display.draw(), f);
        display.finish();
        let end = precise_time_ns();
        println!("dt: {}ms", (end - start) as f32 / (1000000.0f32));

        for ev in display.poll_events() {
            use glium::glutin::Event::*;
            match ev {
                Closed => return,
				_ => ()
			}
		}
        //std::thread::sleep(std::time::Duration::from_millis(1000));
	}
}

fn build_display<'a>() -> WindowBuilder<'a> {
    WindowBuilder::new()
		.with_dimensions(1024, 768)
        //.with_visibility(false)
		.with_title(format!("gr-trace"))
}

fn arg_handle<'a>() -> ArgMatches<'a> {
    App::new(settings::NAME)
        .version(settings::VERSION)
        .author("Sean Purcell <iburinoc@gmail.com>")
        .about("GPU General Relativity Ray Tracer")
        .arg(Arg::with_name("flat")
            .short("f")
            .long("flat")
            .help("Turns off relativistic distortion"))
        .arg(Arg::with_name("iter")
            .short("i")
            .long("iter")
            .help("Sets the number of iterations to raytrace")
            .takes_value(true)
            .value_name("ITER_NUM")
            .default_value("1000"))
        .arg(Arg::with_name("timestep")
            .short("t")
            .long("timestep")
            .help("Sets the length of each time step (where c = 1)")
            .takes_value(true)
            .value_name("TIME_STEP")
            .default_value("0.08"))
        .arg(Arg::with_name("method")
            .short("m")
            .long("method")
            .help("Sets the integration method to use")
            .takes_value(true)
            .value_name("METHOD")
            .default_value("rk4")
                .possible_value("rk4")
                .possible_value("verlet")
                .possible_value("flat"))
        .arg(Arg::with_name("out")
            .short("o")
            .long("out")
            .help("Sets the output file")
            .value_name("FILE")
            .takes_value(true)
            .default_value("out.png"))
        .arg(Arg::with_name("bg")
            .short("b")
            .long("bg")
            .help("Sets the type of background used")
            .takes_value(true)
            .value_name("TYPE")
            .default_value("img")
                .possible_value("black")
                .possible_value("img"))
        .get_matches()
}

fn write_img(display: &GlutinFacade, path: &str) {
    let image: glium::texture::RawImage2d<u8> = display.read_front_buffer();
    let image = image::ImageBuffer::from_raw(image.width, image.height, image.data.into_owned()).unwrap();
    let image = image::DynamicImage::ImageRgba8(image);
    let mut output = std::fs::File::create(&Path::new(path)).unwrap();
    image.save(&mut output, image::ImageFormat::PNG).unwrap();
}

