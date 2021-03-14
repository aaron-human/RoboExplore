use crate::externals::*;
use crate::geo::vec2::*;
use crate::color::*;

#[derive(Debug, Copy, Clone)]
pub enum TextAlignment {
	LEFT,
	CENTER,
	RIGHT,
	JUSTIFY,
}

impl TextAlignment {
	/// Converts it to the CSS string equivalent.
	pub fn to_css(&self) -> &str {
		match self {
			TextAlignment::LEFT => "left",
			TextAlignment::CENTER => "center",
			TextAlignment::RIGHT => "right",
			TextAlignment::JUSTIFY => "justify",
		}
	}
}

/// A way to describe CSS lengths that are useful to DisplayText objects.
#[derive(Debug, Copy, Clone)]
pub enum CssLength {
	/// Measured in terms of average character widths.
	CharWidth(f32),
	/// Measured in terms of average character heights.
	CharHeight(f32),
	/// Measured in terms of pixel sizes.
	Pixels(f32),
}

impl CssLength {
	/// Converts it to the CSS string equivalent.
	pub fn to_css(&self) -> String {
		match self {
			CssLength::CharWidth(magnitude) => format!("{}em", magnitude),
			CssLength::CharHeight(magnitude) => format!("{}ex", 2.0 * magnitude),
			CssLength::Pixels(magnitude) => format!("{}px", magnitude),
		}
	}
}

/// Common info about a piece of DisplayText.
pub struct DisplayText {
	id : DrawTextID,
	color : Color,
	alignment : TextAlignment,
	contents : String,
}

impl DisplayText {
	// TODO? Could implement an interface similar to DisplayBuffer (i.e. setting values doesn't immediately update, must run update() which scans though dirty flags).

	/// Creates a DisplayText positioned according to a point. See `addTextPoint` in `textDisplay.ts` for details.
	pub fn new_text_point(position : Vec2, horizontal : f32, vertical : f32, width : CssLength, height : CssLength, color : &Color, alignment : TextAlignment, text : &str) -> DisplayText {
		DisplayText {
			id: createTextPoint(position.x as i32, position.y as i32, horizontal, vertical, &width.to_css(), &height.to_css(), &color.to_css(), alignment.to_css(), text),
			color: color.clone(),
			alignment,
			contents: String::from(text),
		}
	}

	/// Sets the position of a DisplayText based on the point-centric style of positioning. See `positionTextPoint()` in `textDisplay.ts` for details.
	pub fn set_text_point_position(&mut self, position : &Vec2, horizontal : f32, vertical : f32, width : CssLength, height : CssLength) {
		setTextPointPosition(self.id, position.x as i32, position.y as i32, horizontal, vertical, &width.to_css(), &height.to_css());
	}

	/// Creates a new area to DisplayText. See `addTextArea` in `textDisplay.ts` for details.
	pub fn new_text_area(top : f32, left : f32, bottom : f32, right : f32, color : &Color, alignment : TextAlignment, text : &str) -> DisplayText {
		DisplayText {
			id: createTextArea(top, left, bottom, right, &color.to_css(), alignment.to_css(), text),
			color: color.clone(),
			alignment,
			contents: String::from(text),
		}
	}

	/// Sets the position of a DisplayText based on the text-area style of positioning. See `positionTextArea()` in `textDisplay.ts` for details.
	pub fn set_text_area_position(&mut self, top : f32, left : f32, bottom : f32, right : f32) {
		setTextAreaPosition(self.id, top, left, bottom, right);
	}

	/// Gets the text color of a DisplayText.
	pub fn get_color(&self) -> Color {
		self.color.clone()
	}

	/// Sets the text color of a DisplayText.
	pub fn set_color(&mut self, color : &Color) {
		self.color = color.clone();
		setTextValues(self.id, &self.color.to_css(), self.alignment.to_css(), &self.contents);
	}

	/// Gets the text alignment of a DisplayText.
	pub fn get_alignment(&self) -> TextAlignment {
		self.alignment.clone()
	}

	/// Sets the text alignment of a DisplayText.
	pub fn set_alignment(&mut self, alignment : TextAlignment) {
		self.alignment = alignment;
		setTextValues(self.id, &self.color.to_css(), self.alignment.to_css(), &self.contents);
	}

	/// Gets the (HTML) text of a DisplayText.
	pub fn get_text(&self) -> String {
		self.contents.clone()
	}

	/// Sets the (HTML) text of a DisplayText.
	pub fn set_text(&mut self, text : &str) {
		self.contents = text.to_string();
		setTextValues(self.id, &self.color.to_css(), self.alignment.to_css(), &self.contents);
	}

	/// Makes sure the text is shown.
	pub fn show(&mut self) {
		setDisplayTextVisibility(self.id, true);
	}

	/// Makes sure the text is hidden.
	pub fn hide(&mut self) {
		setDisplayTextVisibility(self.id, false);
	}
}
