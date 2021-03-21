use crate::externals::*;
use crate::color::*;
use crate::camera::*;
use crate::mouse::*;
use crate::keyboard::*;
use crate::gamepad::*;
use crate::display_text::*;
use crate::tiled::*;
use crate::tiled_display::*;
use crate::tiled_geometry::*;
use crate::player::Player;

use crate::geo::vec2::*;
use crate::geo::line_segment::*;
use crate::geo::collision_system::*;

pub struct Game {
	camera : Camera,
	mouse : Mouse,
	keyboard : Keyboard,
	gamepad : Gamepad,
	#[allow(dead_code)] // This should be stored, so the background buffer isn't recycled...
	elapsed : f32,

	collision : CollisionSystem,

	#[allow(dead_code)] // This should be stored, so it's clear where the instructional text comes from...
	description : DisplayText,

	player : Player,

	#[allow(dead_code)] // This should be stored, so it's clear where the instructional text comes from...
	tiled_file : SharedTiledFile,
	tiled_display : TiledDisplay,
	tiled_geometry : TiledGeometry,
}


impl Game {
	pub fn new() -> Game {
		log("Setting up WASM game!");

		let description = DisplayText::new_text_area(
			0.80,
			0.05,
			0.95,
			0.95,
			&Color::new(0, 255, 0, 255),
			TextAlignment::JUSTIFY,
			"Hit the arrow keys or WASD to move around.<br>Click to show mouse button tracking.",
		);

		let mut tiled_file = SharedTiledFile::new();
		assert!(tiled_file.load("room.json").is_ok(), "Couldn't start loading 'room.json'!");

		Game {
			camera: Camera::new(),
			mouse: Mouse::new(),
			keyboard: Keyboard::new(),
			gamepad: Gamepad::new(),
			elapsed: 0.0,

			collision : CollisionSystem::new(),

			description,

			player : Player::new(),

			tiled_file,
			tiled_display : TiledDisplay::new(),
			tiled_geometry : TiledGeometry::new(),
		}
	}

	pub fn handle_tiled_file_loaded(&mut self, _url : &str, mut tiled_file : SharedTiledFile) {
		let file = tiled_file.get().unwrap();
		log(&format!("Point[0]: {:?}", file.get_points()[0].position));
		self.player.position = file.get_points()[0].position;
		self.tiled_display.load_from(&file);
		self.tiled_geometry.load_from(&file);
		for rect in self.tiled_geometry.get_collision_rects() {
			self.collision.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(
				&Vec2::new(rect.x_min(), rect.y_min()),
				&Vec2::new(rect.x_max(), rect.y_min()),
			)));
			self.collision.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(
				&Vec2::new(rect.x_min(), rect.y_max()),
				&Vec2::new(rect.x_max(), rect.y_max()),
			)));

			self.collision.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(
				&Vec2::new(rect.x_min(), rect.y_min()),
				&Vec2::new(rect.x_min(), rect.y_max()),
			)));
			self.collision.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(
				&Vec2::new(rect.x_max(), rect.y_min()),
				&Vec2::new(rect.x_max(), rect.y_max()),
			)));
		}

		self.player.gravity_acceleration.y = -800.0;
	}

	pub fn update(&mut self, elapsed_seconds : f32) {
		self.elapsed += elapsed_seconds;

		self.player.update(self.elapsed, elapsed_seconds, &self.keyboard, &self.gamepad, &self.collision, &self.tiled_geometry);
		self.camera.track_position(&self.player.position);
	}

	pub fn on_resize(&mut self, width : u32, height : u32) {
		self.camera.resize(width, height);
	}

	pub fn on_key_down(&mut self, key : String) {
		self.keyboard.on_down(key);
	}

	pub fn on_key_up(&mut self, key : String) {
		self.keyboard.on_up(key);
	}

	pub fn on_mouse_enter(&mut self) {
		self.mouse.on_enter();
	}

	pub fn on_mouse_update(&mut self, x : u32, y : u32, buttons : u8) {
		self.mouse.on_mouse_update(&self.camera, x, y, buttons);
	}

	pub fn on_mouse_leave(&mut self) {
		self.mouse.on_leave();
	}

	pub fn on_gamepad_changed(&mut self, _valid : bool, buttons : Vec<f32>, raw_analog_sticks : Vec<f32>) {
		// TODO: Some sort of "disconnect pause" via `_valid`?
		//log(&format!("Gamepad state: {:?} {:?} {:?}", valid, buttons, raw_analog_sticks));
		self.gamepad.update(buttons, raw_analog_sticks);
	}
}
