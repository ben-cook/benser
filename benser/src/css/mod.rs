mod color;
mod parser;

pub use color::Color;
pub use parser::Parser;

#[derive(PartialEq, Debug)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(PartialEq, Debug)]
pub struct Rule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Selector {
    Simple(SimpleSelector),
}

#[derive(PartialEq, Eq, Debug)]
pub struct SimpleSelector {
    pub tag_name: Option<String>,
    pub id: Option<String>,
    pub class: Vec<String>,
}

#[derive(PartialEq, Debug)]
pub struct Declaration {
    pub name: String,
    pub value: Value,
}

#[derive(PartialEq, Clone, Debug)]
pub enum Value {
    Keyword(String),
    Length(f32, Unit),
    ColorValue(Color),
    // insert more values here
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Unit {
    Px,
    // insert more units here
}

pub type Specificity = (usize, usize, usize);

impl Selector {
    pub fn specificity(&self) -> Specificity {
        // http://www.w3.org/TR/selectors/#specificity
        let Selector::Simple(ref simple) = *self;
        let a = simple.id.iter().count();
        let b = simple.class.len();
        let c = simple.tag_name.iter().count();
        (a, b, c)
    }
}

impl Value {
    /// Return the size of a length in px, or zero for non-lengths.
    pub fn to_px(&self) -> f32 {
        match *self {
            Value::Length(f, Unit::Px) => f,
            _ => 0.0,
        }
    }
}
