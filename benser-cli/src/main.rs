use std::fs::File;
use std::{error::Error, fs};

use benser::css::Parser as css_parser;
use benser::html::Parser as html_parser;
use benser::layout::{layout_tree, Dimensions};
use benser::painting;
use benser::style::style_tree;

use benser_cli::Args;
use clap::Parser;
use image::ImageFormat;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Read input files
    let css_source = fs::read_to_string(args.css_file)?;
    let html_source = fs::read_to_string(args.html_file)?;

    // Create a virtual viewport
    let mut viewport = Dimensions::default();
    if let Some(viewport_width) = args.viewport_width {
        viewport.content.width = viewport_width;
    } else {
        viewport.content.width = 800.0;
    }
    if let Some(viewport_height) = args.viewport_height {
        viewport.content.height = viewport_height;
    } else {
        viewport.content.height = 600.0;
    }

    // Parsing and rendering:
    let root_node = html_parser::parse(html_source);
    let stylesheet = css_parser::parse(css_source);
    let style_root = style_tree(&root_node, &stylesheet);
    let layout_root = layout_tree(&style_root, viewport);

    // Create the output file:
    File::create(&args.output).unwrap();

    // Write to the file:
    let canvas = painting::paint(&layout_root, viewport.content);
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let img = image::ImageBuffer::from_fn(w, h, move |x, y| {
        let color = canvas.pixels[(y * w + x) as usize];
        image::Rgba([color.r, color.g, color.b, color.a])
    });

    img.save_with_format(&args.output, ImageFormat::Png)?;

    Ok(())
}
