/// Something with an area.
pub trait Shape {
    /// The surface; see [`Rect::area`].
    fn area(&self) -> f64;
}

/// How wide a shape is scaled by default.
pub const SCALE: f64 = 2.0;

/// A rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(width: f64, height: f64) -> Self {
        Rect { width, height }
    }

    pub fn scaled(self) -> Self {
        Rect::new(self.width * SCALE, self.height * SCALE)
    }
}

impl Shape for Rect {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// How a rectangle is oriented.
#[derive(Debug, PartialEq)]
pub enum Kind {
    Square,
    Wide,
}

pub fn kind_of(rect: &Rect) -> Kind {
    if rect.width > rect.height {
        Kind::Wide
    } else {
        Kind::Square
    }
}

/// Adds up the areas; both sums are called `total` in their own scope.
pub fn total_area(shapes: &[&dyn Shape]) -> f64 {
    let mut total = 0.0;
    for shape in shapes {
        total += shape.area();
    }
    total
}

pub fn double(value: f64) -> f64 {
    let total = value * 2.0;
    total
}
