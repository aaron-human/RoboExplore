use crate::externals::*;
use crate::geo::vec2::*;
use crate::geo::vec3::*;
use crate::geo::mat4::*;
use crate::geo::bounds2::*;

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
		let mut display = Mat4::new();
		display.scale_before(&Vec3::new(
			2.0 / (width as f32),
			2.0 / (height as f32),
			1.0,
		));
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
}
