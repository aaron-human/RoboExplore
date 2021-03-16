use crate::externals::*;
use crate::static_singletons::is_browser_little_endian;

pub trait ColorExportable {
	fn raw_export(&self, output : &mut Vec<ColorMagnitude>);
}

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

impl ColorExportable for Color {
	fn raw_export(&self, output : &mut Vec<ColorMagnitude>) {
		output.push(self.red);
		output.push(self.green);
		output.push(self.blue);
		output.push(self.alpha);
	}
}

/// A class for storing texture positions as colors.
#[derive(Clone)]
pub struct TexturePositionAsColor {
	pub x : u16,
	pub y : u16,
}

impl TexturePositionAsColor {
	pub fn new(x : u16, y : u16) -> TexturePositionAsColor {
		TexturePositionAsColor { x, y }
	}
}

impl ColorExportable for TexturePositionAsColor {
	fn raw_export(&self, output : &mut Vec<ColorMagnitude>) {
		let x_pieces;
		let y_pieces;
		// Unfortunately browsers use the system's endian to handle WebGL arrays.
		// So must pack the bytes according to that.
		if is_browser_little_endian() {
			x_pieces = self.x.to_le_bytes();
			y_pieces = self.y.to_le_bytes();
		} else {
			x_pieces = self.x.to_be_bytes();
			y_pieces = self.y.to_be_bytes();
		}
		output.push(x_pieces[0]);
		output.push(x_pieces[1]);
		output.push(y_pieces[0]);
		output.push(y_pieces[1]);
	}
}