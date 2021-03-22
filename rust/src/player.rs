
use crate::geo::mat4::Mat4;
use crate::geo::vec2::*;
use crate::geo::vec3::Vec3;
use crate::geo::consts::EPSILON;
use crate::geo::collider::limit_movement_with_normals;

use crate::externals::log;

use crate::display_texture::DisplayTexture;
use crate::display_buffer::{DisplayBuffer, DisplayBufferType};
use crate::geo::collision_system::CollisionSystem;
use crate::keyboard::*;
use crate::gamepad::*;
use crate::tiled_geometry::TiledGeometry;

/// The max number of physics iterations the player.
const PHYSICS_ITERATION_MAX : usize = 5;

/// The radius of the player's (circle) collider.
const PLAYER_RADIUS : f32 = 8.0;
/// How fast the player moves in pixels per second.
const PLAYER_SPEED : f32 = 120.0;

/// Max track snap distance.
const MAX_TRACK_SNAP_DISTANCE : f32 = 3.0;
/// The starting speed when kicking off a track vertically.
const TRACK_KICK_VERTICAL_START_SPEED : f32 = 320.0;
/// The starting speed when kicking off a track horizontally.
const TRACK_KICK_HORIZONTAL_START_SPEED : f32 = 120.0;
/// How long before the track kick velocity zeros (in seconds).
const TRACK_KICK_TIME : f32 = 1.0;

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

	/// Whether the jump input has been "used up" and should be ignored until it's released.
	jump_input_used : bool,
	/// Whether the track snap input has been "used up" and should be ignored until it's released.
	track_input_used : bool,

	/// Whether the the player is on the track.
	on_track : bool,

	/// Current acceleration due to gravity.
	pub gravity_acceleration : Vec2,
	/// The current velocity due to gravity.
	gravity_velocity : Vec2,
	/// Whether was on ground last update.
	on_ground : bool,

	/// The velocity due to jumping.
	jump_velocity : Vec2,
	/// The time when the current jump started. Negative means no jump.
	jump_start_time : f32,
	/// The starting height of the current jump.
	jump_start_height : f32,
	/// Whether the current jump is just done. This is mainly a way for jumps to be cut short.
	jump_done : bool,

	/// The initial velocity from kiching off the track.
	kick_start_velocity : Vec2,
	/// When the last track kick happened.
	kick_start_time : f32,

	/// The display buffer for the player.
	display : DisplayBuffer,
	/// The texture used to draw the player.
	#[allow(dead_code)] // This should be stored, so it's clear where the instructional text comes from...
	texture : DisplayTexture,
	/// Whether the sprite should be looking to the right.
	aiming_right : bool,
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

			jump_input_used : false,
			track_input_used : false,

			on_track : false,

			gravity_acceleration : Vec2::new(0.0, 0.0),
			gravity_velocity : Vec2::new(0.0, 0.0),
			on_ground : false,

			jump_velocity : Vec2::new(0.0, 0.0),
			jump_start_time : -1.0,
			jump_start_height : 0.0,
			jump_done : true,

			kick_start_velocity : Vec2::new(0.0, 0.0),
			kick_start_time : -1.0,

			display : display_buffer,
			texture,
			aiming_right : true,
		}
	}

	/// Calculate the needed velocity to get to some height given the current height and vertical velocity.
	fn calc_jump_velocity(&self, target_height : f32) -> f32 {
		(2.0 * self.gravity_acceleration.length() * target_height).abs().sqrt()
	}

	/// The fuction that updates the player's position and movement.
	pub fn update(&mut self, current_time : f32, elapsed_seconds : f32, keyboard : &Keyboard, gamepad : &Gamepad, collision : &CollisionSystem, geometry : &TiledGeometry) {

		let debug = keyboard.is_down(Key::DEBUG);

		let gravity_active = EPSILON < self.gravity_acceleration.length() && !self.on_track;

		// Handle the player's inputs.
		let mut input_direction = gamepad.direction();
		let mut input_scale = input_direction.x.abs().max(input_direction.y.abs());
		if keyboard.is_down(Key::UP) {
			input_direction.y += 1.0;
			input_scale = 1.0;
		}
		if keyboard.is_down(Key::LEFT) {
			input_direction.x -= 1.0;
			input_scale = 1.0;
		}
		if keyboard.is_down(Key::DOWN) {
			input_direction.y -= 1.0;
			input_scale = 1.0;
		}
		if keyboard.is_down(Key::RIGHT) {
			input_direction.x += 1.0;
			input_scale = 1.0;
		}
		if EPSILON < input_direction.length() {
			(&mut input_direction).norm();
		}

		// Generate a sane movement the player is trying to add to the movement based on the above input(s).
		let input_movement = if 0.0 < input_direction.length() {
			(&mut input_direction).norm();
			input_direction.set_length(input_scale * elapsed_seconds * PLAYER_SPEED)
		} else {
			Vec2::new(0.0, 0.0)
		};

		// Handle gravity acceleration.
		if gravity_active {
			self.gravity_velocity += self.gravity_acceleration * elapsed_seconds;
		}

		// Handle jumping.
		// This overrides gravity.
		let gravity_direction = self.gravity_acceleration.norm();
		let jump_pressed = gamepad.is_down(Button::A) || keyboard.is_down(Key::UP);
		if jump_pressed && gravity_active {
			let height = -self.position.dot(gravity_direction);
			if self.on_ground && !self.jump_input_used {
				// Start jumping.
				// Start by killing off gravity, so it doesn't start "ahead" an iteration.
				self.gravity_velocity.x = 0.0;
				self.gravity_velocity.y = 0.0;

				self.jump_velocity = gravity_direction.set_length(-self.calc_jump_velocity(MIN_JUMP_HEIGHT));
				self.jump_start_time = current_time;
				self.jump_start_height = height;
				self.jump_done = false;
				self.jump_input_used = true;
			} else if !self.jump_done {
				let jump_elapsed_time : f32 = current_time - self.jump_start_time;
				if jump_elapsed_time < MAX_JUMP_TIME {
					// Then continue to push the jump up.
					let jump_percent : f32 = 1.0_f32.min(jump_elapsed_time / MAX_JUMP_TIME);
					let target_jump_height = jump_percent * (MAX_JUMP_HEIGHT - MIN_JUMP_HEIGHT) + MIN_JUMP_HEIGHT;
					// Because some integration has already occurred with a lower target jump height, must "correct" against that.
					// Do so by increasing the desired height depending on how far the current height is from where it would be if had started with the "right" velocity to hit the current target_jump_height.
					let current_height = height - self.jump_start_height;
					let ideal_current_height = self.calc_jump_velocity(target_jump_height) * jump_elapsed_time - 0.5 * self.gravity_acceleration.length() * jump_elapsed_time * jump_elapsed_time;
					let height_correction = 0.0f32.max(ideal_current_height - current_height);
					// Then setup the jump value.
					self.jump_velocity = gravity_direction.set_length(-self.calc_jump_velocity(target_jump_height + height_correction));
				}
			}
		}
		if !jump_pressed {
			self.jump_start_time = -1.0;
			self.jump_start_height = 0.0;
			self.jump_done = true;
			self.jump_input_used = false;
		}

		// Handle track jumping.
		let kick_velocity = if self.on_track && jump_pressed && !self.jump_input_used {
			self.on_track = false;
			let mut kick_direction = input_direction.clone();
			if EPSILON > kick_direction.length() {
				kick_direction.y = 1.0; // Default to straight up if nothing else.
			}
			{
				let vertical = kick_direction.dot(&gravity_direction);
				let ortho = gravity_direction.ortho();
				let horizontal = kick_direction.dot(&ortho);
				self.kick_start_velocity =
					gravity_direction * vertical * TRACK_KICK_VERTICAL_START_SPEED +
					ortho * horizontal * TRACK_KICK_HORIZONTAL_START_SPEED;
			}
			self.kick_start_time = current_time;
			self.jump_input_used = true;
			self.kick_start_velocity
		} else {
			let elapsed = current_time - self.kick_start_time;
			if elapsed < TRACK_KICK_TIME && EPSILON < self.kick_start_velocity.length() {
				let percent = 0.0f32.max((TRACK_KICK_TIME - elapsed) / TRACK_KICK_TIME);
				self.kick_start_velocity * percent
			} else {
				self.kick_start_velocity.x = 0.0;
				self.kick_start_velocity.y = 0.0;
				Vec2::new(0.0, 0.0)
			}
		};

		// Now repeatedly alternate between collision detection and responding by modifying forces.
		let track_pressed = gamepad.is_down(Button::R) || keyboard.is_down(Key::SPACE);
		let mut remainder_percent = 1.0;
		let mut normals : Vec<Vec2> = Vec::new();
		self.on_ground = false; // Off the ground until proven otherwise.
		for _iteration in 0..PHYSICS_ITERATION_MAX {
			// First calculate the projected movement.
			let mut total_movement = (self.gravity_velocity + self.jump_velocity + kick_velocity) * elapsed_seconds;
			total_movement.x += input_movement.x;
			if self.on_track {
				total_movement.y += input_movement.y;
			}
			total_movement *= remainder_percent;
			if debug { log(&format!("total_movement: {:?}", total_movement)); }

			// Remove any movement that goes against a surface normal from the previous iteration.
			total_movement = limit_movement_with_normals(&total_movement, &normals);
			normals.clear();
			if debug { log(&format!("total_movement after normals: {:?}", total_movement)); }

			// Give up early if (basically) no movement left.
			if EPSILON > total_movement.length() {
				break;
			}

			// Check how that works with collision.
			let maybe_collision = collision.collide_circle_step(
				&self.position,
				PLAYER_RADIUS,
				&total_movement,
			);/*
			let maybe_collision = {
				let possible = collision.collide_circle_step(
					&self.position,
					PLAYER_RADIUS,
					&total_movement,
				);

				// Ignore collisions that deflect by basically zero.
				if let Some(collision) = &possible {
					let original_final = self.position + total_movement;
					if (original_final - collision.final_position).length() < EPSILON {
						None
					} else {
						possible
					}
				} else {
					possible
				}
			};*/
			if debug { log(&format!("collision: {:?}", maybe_collision)); }

			// If there is a collision, interact with it.
			let mut safe_movement = total_movement.clone();
			if let Some(collision) = &maybe_collision {
				let safe_percent = 0.0f32.max(collision.deflections[0].times.min().unwrap());
				if debug { log(&format!("Safe percent: {}", safe_percent)); }
				safe_movement *= safe_percent;
				remainder_percent *= 1.0 - safe_percent;

				// Save the normals.
				normals = collision.normals.clone();

				// See how the collision might update the on_ground and hit_ceiling flags.
				let mut on_ground = false;
				let mut hit_ceiling = false;
				let threshold = -0.65 * self.gravity_acceleration.length();
				// Threshold is below "sqrt(2) / 2" (0.7071) so can handle anything within 45 degrees.
				for deflection in &collision.deflections {
					let coincidence = deflection.normal.dot(self.gravity_acceleration);
					if threshold > coincidence {
						on_ground = true;
					}
					if -threshold < coincidence {
						hit_ceiling = true;
					}
				}
				if on_ground {
					if debug { log("On ground!"); }
					self.gravity_velocity.x = 0.0;
					self.gravity_velocity.y = 0.0;
					self.jump_velocity.x = 0.0;
					self.jump_velocity.y = 0.0;
					self.kick_start_velocity.x = 0.0;
					self.kick_start_velocity.y = 0.0;
				}
				if hit_ceiling {
					self.gravity_velocity.x = 0.0; // Might remove this part?
					self.gravity_velocity.y = 0.0;
					self.jump_velocity.x = 0.0;
					self.jump_velocity.y = 0.0;
					self.kick_start_velocity.y = 0.0;
					self.jump_done = true;
				}
				self.on_ground |= on_ground;
			}

			// If the player is trying to snap, then try to collide any safe movement with tracks to see if can snap.
			// Also check if the starting position is just close enough.
			if track_pressed && !self.track_input_used {
				if !self.on_track {
					// Try snapping if possible.
					let closest = geometry.get_closest_track_point(&self.position);
					if MAX_TRACK_SNAP_DISTANCE >= (closest - self.position).length() {
						self.position = closest;
						self.gravity_velocity.x = 0.0;
						self.gravity_velocity.y = 0.0;
						self.jump_velocity.x = 0.0;
						self.jump_velocity.y = 0.0;
						self.kick_start_velocity.x = 0.0;
						self.kick_start_velocity.y = 0.0;
						self.track_input_used = true;
						self.on_track = true;
						break; // Ignore any movement after that.
					} else if let Some(intersection) = geometry.collide_moving_point_with_track(&self.position, &safe_movement) {
						let used_percent = (intersection - self.position).length() / safe_movement.length();
						self.position = intersection;
						safe_movement *= 1.0 - used_percent;
						self.gravity_velocity.x = 0.0;
						self.gravity_velocity.y = 0.0;
						self.jump_velocity.x = 0.0;
						self.jump_velocity.y = 0.0;
						self.kick_start_velocity.x = 0.0;
						self.kick_start_velocity.y = 0.0;
						self.track_input_used = true;
						self.on_track = true;
						// Don't break, allow any remaining movement to be worked out.
					}
				} else {
					self.track_input_used = true;
					self.on_track = false;
				}
			}
			if !track_pressed {
				self.track_input_used = false;
			}
			// Then limit movement if on a track.
			if self.on_track {
				// TODO? Could make sure didn't "jump a gap" here?
				let updated_end = geometry.get_closest_track_point(&(self.position + safe_movement));
				safe_movement = updated_end - self.position;
			}

			// Handle moving with collision.
			self.position += safe_movement;

			if 0.0 > safe_movement.x {
				self.aiming_right = false;
			}
			if 0.0 < safe_movement.x {
				self.aiming_right = true;
			}

			// If no collision happened, then this is done.
			if maybe_collision.is_none() {
				break;
			}

			if PHYSICS_ITERATION_MAX-1 == _iteration {
				log("Hit player physics iteration max!");
			}
		}

		// Store the new position.
		{
			let mut transform = Mat4::new();
			transform.translate_before(&Vec3::new(self.position.x, self.position.y, 0.0));
			if !self.aiming_right {
				transform.scale_before(&Vec3::new(-1.0, 1.0, 1.0));
			}
			self.display.set_transform(&transform);
		}
	}
}
