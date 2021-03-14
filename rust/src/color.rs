use crate::externals::*;

/// A class for storing colors.
#[derive(Clone)]
pub struct Color {
	pub red : ColorMagnitude,
	pub green : ColorMagnitude,
	pub blue : ColorMagnitude,
	pub alpha : ColorMagnitude,
}

impl Color {
	pub fn new(red : ColorMagnitude, green : ColorMagnitude, blue : ColorMagnitude, alpha : ColorMagnitude) -> Color {
		Color { red, green, blue, alpha }
	}

	pub fn to_css(&self) -> String {
		format!("rgba({}, {}, {}, {})", self.red, self.green, self.blue, (self.alpha as f32) / 255.0)
	}
}
