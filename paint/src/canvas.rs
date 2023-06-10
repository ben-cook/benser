use std::iter;

use benser::css::Color;

use super::DisplayCommand;

pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    // Create a blank canvas
    pub fn new(width: usize, height: usize) -> Canvas {
        let white = Color::new(255, 255, 255, 255);
        Canvas {
            pixels: iter::repeat(white).take(width * height).collect(),
            width,
            height,
        }
    }

    pub fn paint_item(&mut self, item: &DisplayCommand) {
        match item {
            &DisplayCommand::SolidColor(color, rect) => {
                // Clip the rectangle to the canvas boundaries.
                let x0 = rect.x.clamp(0.0, self.width as f32) as usize;
                let y0 = rect.y.clamp(0.0, self.height as f32) as usize;
                let x1 = (rect.x + rect.width).clamp(0.0, self.width as f32) as usize;
                let y1 = (rect.y + rect.height).clamp(0.0, self.height as f32) as usize;

                for y in y0..y1 {
                    for x in x0..x1 {
                        // TODO: alpha compositing with existing pixel
                        self.pixels[x + y * self.width] = color;
                    }
                }
            }
        }
    }
}
