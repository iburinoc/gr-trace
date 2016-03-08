#[macro_use]
extern crate glium;

fn main() {
	use glium::DisplayBuild;

	let display = glium::glutin::WindowBuilder::new()
		.with_dimensions(1024, 768)
		.with_title(format!("Hello world"))
		.build_glium()
		.unwrap();

	loop {
		for ev in display.poll_events() {
			match ev {
				glium::glutin::Event::Closed => return,
				_ => ()
			}
		}
	}
}
