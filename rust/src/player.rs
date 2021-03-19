
use crate::geo::mat4::Mat4;
use crate::geo::vec2::*;
use crate::geo::vec3::Vec3;
use crate::geo::consts::EPSILON;

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
	/// Current acceleration due to gravity.
	pub gravity_acceleration : Vec2,
	/// The current velocity due to gravity.
	gravity_velocity : Vec2,
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
			gravity_acceleration : Vec2::new(0.0, 0.0),
			gravity_velocity : Vec2::new(0.0, 0.0),
			display : display_buffer,
			texture
		}
	}

	pub fn update(&mut self, elapsed_seconds : f32, keyboard : &Keyboard, collision : &CollisionSystem) {
		// Handle the player's inputs.
		let mut input_movement = Vec2::zero();
		if keyboard.is_down(Key::UP) {
			input_movement.y += 1.0;
		}
		if keyboard.is_down(Key::LEFT) {
			input_movement.x -= 1.0;
		}
		if keyboard.is_down(Key::DOWN) {
			input_movement.y -= 1.0;
		}
		if keyboard.is_down(Key::RIGHT) {
			input_movement.x += 1.0;
		}
		if 0.0 < input_movement.length() {
			(&mut input_movement).set_length(elapsed_seconds * PLAYER_SPEED);
		}

		// Handle gravity.
		if EPSILON < self.gravity_acceleration.length() {
			self.gravity_velocity += self.gravity_acceleration * elapsed_seconds;
		}
		let gravity_movement = self.gravity_velocity * elapsed_seconds;

		// Then see how that movement works out with collision.
		let total_movement = input_movement + gravity_movement;
		if EPSILON < total_movement.length() {
			// Get collision information.
			let collisions = collision.collide_circle(
				&self.position,
				PLAYER_RADIUS,
				&total_movement,
			);

			// Stop gravity if on the ground.
			{
				let mut hit_ground = false;
				let threshold = -0.9 * self.gravity_acceleration.length();
				for collision in &collisions {
					for deflection in &collision.deflections {
						if threshold > deflection.normal.dot(self.gravity_acceleration) {
							hit_ground = true;
							break;
						}
					}
					if hit_ground { break; }
				}
				if hit_ground {
					self.gravity_velocity.x = 0.0;
					self.gravity_velocity.y = 0.0;
				}
			}

			// Handle moving with collision.
			if let Some(collision) = collisions.last() {
				self.position = collision.final_position;
			} else {
				self.position += total_movement;
			}
			self.display.set_transform(Mat4::new().translate_before(&Vec3::new(self.position.x, self.position.y, 0.0)));
		}
	}
}
