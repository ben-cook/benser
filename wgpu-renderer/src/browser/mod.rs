mod state;

use crate::args::Args;
use benser::css::Parser as css_parser;
use benser::style::style_tree;
use html::parser::Parser as html_parser;
use state::State;
use std::fs;
use winit::event::WindowEvent;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run(args: Args) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("benser")
        .build(&event_loop)
        .unwrap();

    let css_source = fs::read_to_string(args.css_file).unwrap();
    let html_source = fs::read_to_string(args.html_file).unwrap();

    let root_node = html_parser::from_string(&html_source).run();
    let stylesheet = css_parser::parse(&css_source);
    let style_root = style_tree(&root_node, &stylesheet);

    let mut state: State<'_> = State::new(window, style_root).await;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Reconfigure the surface if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.window_size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            state.window().request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window().id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
