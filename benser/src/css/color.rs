#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn as_float(&self) -> [f32; 4] {
        [self.r.into(), self.g.into(), self.b.into(), self.a.into()]
    }
}
