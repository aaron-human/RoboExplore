use crate::externals::*;
use crate::geo::vec2::*;
use crate::geo::vec3::*;
use crate::geo::mat4::*;
use crate::geo::bounds2::*;

/// What percent of the screen is reserved (tracked positions aren't allowed in it).
const TRACK_MARGIN_PERCENT : f32 = 0.5;

pub struct Camera {
	pub center : Vec3,
	screen_width : u32,
	screen_height : u32,
}

impl Camera {
	pub fn new() -> Camera {
		Camera {
			center: Vec3::zero(),
			screen_width: 1,
			screen_height: 1,
		}
	}

	/// Resizes the screen.
	pub fn resize(&mut self, width : u32, height : u32) {
		self.screen_width = width;
		self.screen_height = height;
		self.set_transform();
	}

	fn set_transform(&mut self) {
		let mut display = Mat4::new();
		let mut translation = &self.center * -1.0;
		// Keep things pixel perfect even with odd widths/heights
		if 1 == self.screen_width  % 2 { translation.x -= 0.5; }
		if 1 == self.screen_height % 2 { translation.y -= 0.5; }
		display.scale_before(&Vec3::new(
			2.0 / (self.screen_width  as f32),
			2.0 / (self.screen_height as f32),
			1.0,
		)).translate_before(&translation);
		setDisplayTransform(display.export());
	}

	/// Gets the size of the screen.
	pub fn size(&self) -> Vec3 {
		Vec3::new(self.screen_width as f32, self.screen_height as f32, 0.0)
	}

	/// Gets the game world bounds.
	pub fn bounds(&self) -> Bounds2 {
		Bounds2::from_centered_rect(&Vec2::new(self.center.x, self.center.y), self.screen_width as f32, self.screen_height as f32)
	}

	/// Converts a (cartesian) position on the screen to a position in game.
	pub fn to_game_space(&self, screen_position : &Vec3) -> Vec3 {
		Vec3 {
			x: screen_position.x - ((self.screen_width  / 2) as f32) + self.center.x,
			y:-screen_position.y + ((self.screen_height / 2) as f32) + self.center.y,
			z: self.center.z,
		}
	}

	/// Track the given location with this camera.
	pub fn track_position(&mut self, position : &Vec2) {
		let percent = (1.0 - TRACK_MARGIN_PERCENT) / 2.0;
		let max_x_distance = (self.screen_width  as f32) * percent;
		let max_y_distance = (self.screen_height as f32) * percent;
		let mut changed = false;
		if (self.center.x - position.x).abs() > max_x_distance {
			if self.center.x < position.x {
				self.center.x = position.x - max_x_distance;
			} else {
				self.center.x = position.x + max_x_distance;
			}
			self.center.x = self.center.x.floor();
			changed = true;
		}
		if (self.center.y - position.y).abs() > max_y_distance {
			if self.center.y < position.y {
				self.center.y = position.y - max_y_distance;
			} else {
				self.center.y = position.y + max_y_distance;
			}
			self.center.y = self.center.y.floor();
			changed = true;
		}
		if changed {
			self.set_transform();
		}
	}
}
