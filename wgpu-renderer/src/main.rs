use clap::Parser;
use wgpu_renderer::{args::Args, browser, file_output};

fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Some(ref path) = args.output {
        pollster::block_on(file_output::run(args))
    } else {
        pollster::block_on(browser::run(args));
    }
}
