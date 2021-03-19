
use crate::geo::mat4::Mat4;
use crate::geo::vec2::*;
use crate::geo::vec3::Vec3;

use crate::externals::log;

use crate::display_texture::DisplayTexture;
use crate::display_buffer::{DisplayBuffer, DisplayBufferType};
use crate::geo::collision_system::CollisionSystem;
use crate::keyboard::*;

const PLAYER_RADIUS : f32 = 8.0;
/// How fast the player moves in pixels per second.
const PLAYER_SPEED : f32 = 100.0;

/// The player's data.
pub struct Player {
	/// The player's position. This is the center of the player.
	pub position : Vec2,
	/// The display buffer for the player.
	display : DisplayBuffer,
	/// The texture used to draw the player.
	#[allow(dead_code)] // This should be stored, so it's clear where the instructional text comes from...
	texture : DisplayTexture,
}

impl Player {
	pub fn new() -> Player {
		let mut texture = DisplayTexture::new();
		texture.load_from_url("player.png");
		let mut display_buffer = DisplayBuffer::new(DisplayBufferType::IMAGES);
		{
			let mut editor = display_buffer.make_editor();
			editor.add_image(
				&Vec2::new(0.0, 0.0),
				&Vec2::new(16.0, 16.0),
				&Vec3::new(-8.0, -8.0, 0.0),
			);
		}
		display_buffer.set_texture(&texture);
		Player {
			position : Vec2::new(0.0, 0.0),
			display : display_buffer,
			texture
		}
	}

	pub fn update(&mut self, elapsed_seconds : f32, keyboard : &Keyboard, collision : &CollisionSystem) {
		//
		let mut movement = Vec2::zero();
		if keyboard.is_down(Key::UP) {
			movement.y += 1.0;
		}
		if keyboard.is_down(Key::LEFT) {
			movement.x -= 1.0;
		}
		if keyboard.is_down(Key::DOWN) {
			movement.y -= 1.0;
		}
		if keyboard.is_down(Key::RIGHT) {
			movement.x += 1.0;
		}
		if 0.0 < movement.length() {
			let distance = elapsed_seconds * PLAYER_SPEED;
			(&mut movement).set_length(distance);

			let collisions = collision.collide_circle(
				&self.position,
				PLAYER_RADIUS,
				&movement,
			);
			if let Some(collision) = collisions.last() {
				self.position = collision.final_position;
			} else {
				self.position += movement;
			}
			self.display.set_transform(Mat4::new().translate_before(&Vec3::new(self.position.x, self.position.y, 0.0)));
		}
	}
}
