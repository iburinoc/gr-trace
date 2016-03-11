#[macro_use]
extern crate glium;
extern crate clap;

use glium::glutin::WindowBuilder;
use glium::DisplayBuild;

use clap::{Arg, App, ArgMatches};

mod settings;

fn main() {
	use glium::DisplayBuild;

    let args = arg_handle();
	let display = build_display().build_glium().unwrap();

    println!("{}", settings::NAME);

	loop {
		for ev in display.wait_events() {
			match ev {
				glium::glutin::Event::Closed => return,
				_ => ()
			}
		}
	}
}

fn build_display<'a>() -> WindowBuilder<'a> {
    WindowBuilder::new()
		.with_dimensions(1024, 768)
		.with_title(format!("gr-trace"))
}

fn arg_handle<'a>() -> ArgMatches<'a> {
    App::new(settings::NAME)
        .version(settings::VERSION)
        .author("Sean Purcell <iburinoc@gmail.com>")
        .about("GPU General Relativity Ray Tracer")
        .get_matches()
}

