
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

/// The min time to hold the jump to get the max height (in seconds).
const MAX_JUMP_TIME : f32 = 0.2;
/// The min jump height.
const MIN_JUMP_HEIGHT : f32 = 16.0;
/// The min jump height.
const MAX_JUMP_HEIGHT : f32 = 64.0 + 4.0;

/// The player's data.
pub struct Player {
	/// The player's position. This is the center of the player.
	pub position : Vec2,

	/// Current acceleration due to gravity.
	pub gravity_acceleration : Vec2,
	/// The current velocity due to gravity.
	gravity_velocity : Vec2,
	/// Whether was on ground last update.
	on_ground : bool,

	/// The time when the current jump started. Negative means no jump.
	jump_start_time : f32,
	/// The starting height of the current jump.
	jump_start_height : f32,

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
			on_ground : false,

			jump_start_time : -1.0,
			jump_start_height : 0.0,

			display : display_buffer,
			texture
		}
	}

	/// Calculate the needed velocity to get to some height given the current height and vertical velocity.
	fn calc_jump_velocity(&self, current_height : f32, target_height : f32) -> f32 {
		(2.0 * self.gravity_acceleration.length() * (current_height - target_height)).abs().sqrt()
	}

	pub fn update(&mut self, current_time : f32, elapsed_seconds : f32, keyboard : &Keyboard, collision : &CollisionSystem) {
		// Handle the player's inputs.
		let mut input_movement = Vec2::zero();
		if keyboard.is_down(Key::UP) {
			//input_movement.y += 1.0;

		}
		if keyboard.is_down(Key::LEFT) {
			input_movement.x -= 1.0;
		}
		if keyboard.is_down(Key::DOWN) {
			//input_movement.y -= 1.0;
		}
		if keyboard.is_down(Key::RIGHT) {
			input_movement.x += 1.0;
		}
		if 0.0 < input_movement.length() {
			(&mut input_movement).set_length(elapsed_seconds * PLAYER_SPEED);
		}

		let gravity_active = EPSILON < self.gravity_acceleration.length();

		// Handle gravity.
		if gravity_active {
			self.gravity_velocity += self.gravity_acceleration * elapsed_seconds;
		}

		// Handle jumping.
		// This overrides gravity.
		let gravity_direction = self.gravity_acceleration.norm();
		if keyboard.is_down(Key::UP) && gravity_active {
			let height = self.position.dot(&gravity_direction);
			if 0.0 > self.jump_start_time && self.on_ground {
				// Start jumping.
				self.gravity_velocity = gravity_direction.set_length(-self.calc_jump_velocity(0.0, MIN_JUMP_HEIGHT));
				self.jump_start_time = current_time;
				self.jump_start_height = height;
			} else if 0.0 < self.jump_start_time {
				let jump_elapsed_time : f32 = current_time - self.jump_start_time;
				if jump_elapsed_time < MAX_JUMP_TIME {
					// Then continue to push the jump up.
					let jump_percent : f32 = 1.0_f32.min(jump_elapsed_time / MAX_JUMP_TIME);
					let target_jump_height = jump_percent * (MAX_JUMP_HEIGHT - MIN_JUMP_HEIGHT) + MIN_JUMP_HEIGHT;
					let target_velocity = self.calc_jump_velocity(
						(height - self.jump_start_height).abs(),
						target_jump_height,
					);
					self.gravity_velocity = gravity_direction.set_length(-target_velocity);
				}
			}
		} else {
			self.jump_start_time = -1.0;
		}

		// Then see how that movement works out with collision.
		let total_movement = input_movement + self.gravity_velocity * elapsed_seconds;
		if EPSILON < total_movement.length() {
			// Get collision information.
			let collisions = collision.collide_circle(
				&self.position,
				PLAYER_RADIUS,
				&total_movement,
			);

			// Stop gravity if on the ground.
			{
				self.on_ground = false;
				let threshold = -0.9 * self.gravity_acceleration.length();
				for collision in &collisions {
					for deflection in &collision.deflections {
						if threshold > deflection.normal.dot(self.gravity_acceleration) {
							self.on_ground = true;
							break;
						}
					}
					if self.on_ground { break; }
				}
				if self.on_ground {
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
