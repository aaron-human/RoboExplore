use core::cell::Cell;
use std::ptr;

use crate::externals::*;
use crate::geo::vec3::*;
use crate::geo::mat4::*;
use crate::color::*;
use crate::display_buffer::*;
use crate::display_texture::*;
use crate::camera::*;
use crate::mouse::*;
use crate::keyboard::*;
use crate::display_text::*;
use crate::bullet::*;
use crate::tiled::*;
use crate::tiled_display::*;

use crate::geo::vec2::*;
use crate::geo::circle::*;
use crate::geo::line_segment::*;
use crate::geo::collision_system::*;

/// How fast the player moves in pixels per second.
const PLAYER_SPEED : f32 = 90.0;

pub struct Game {
	camera : Camera,
	mouse : Mouse,
	keyboard : Keyboard,
	player : DisplayBuffer,
	player_position : Vec3,
	mouse_draw : DisplayBuffer,
	#[allow(dead_code)] // This should be stored, so the background buffer isn't recycled...
	background : DisplayBuffer,
	elapsed : f32,

	collision : CollisionSystem,

	#[allow(dead_code)] // This should be stored, so it's clear where the instructional text comes from...
	description : DisplayText,
	middle_click_tracker : ClickTracker,
	right_click_tracker : ClickTracker,

	bullets : Vec<Bullet>,

	texture : DisplayTexture,
	images : DisplayBuffer,
	texture2 : DisplayTexture,
	images2 : DisplayBuffer,
}

const PLAYER_RADIUS : f32 = 16.0;

struct ClickTracker {
	display : DisplayBuffer,
	text : DisplayText,
}

impl ClickTracker {
	fn new(tag : &str) -> ClickTracker {
		let mut display = DisplayBuffer::new(DisplayBufferType::LINES);
		{
			let color = Color::new(255, 255, 255, 255);
			const SIZE : f32 = 25.0;
			let mut editor = display.make_editor();
			editor.add_polygon(
				&vec![
					Vec3::new(0.0, 0.0, 0.0),
					Vec3::new(SIZE / 2.0, SIZE, 0.0),
					Vec3::new(-SIZE / 2.0, SIZE, 0.0),
				],
				&color,
			);
		}
		display.hide();

		let mut text = DisplayText::new_text_point(
			Vec2::new(100.0, 100.0),
			0.5, 0.5,
			CssLength::CharWidth(10.0), CssLength::CharHeight(1.0),
			&Color::new(255, 0, 0, 255),
			TextAlignment::CENTER,
			tag
		);
		text.hide();
		ClickTracker{ display, text }
	}

	pub fn move_to(&mut self, position : &Vec3) {
		self.display.set_transform(Mat4::new().translate_before(position));
		self.display.show();
		self.text.set_text_point_position(
			&Vec2::new(position.x, position.y),
			0.5, 0.0,
			CssLength::CharWidth(10.0), CssLength::CharHeight(1.0),
		);
		self.text.show();
	}
}

/// A horrible hack to quickly get the TiledDisplay working.
static mut TILED_DISPLAY : *mut TiledDisplay = ptr::null_mut();

pub fn test_do_load(mut file : Cell<TiledFile>) {
	let display;
	unsafe {
		TILED_DISPLAY = Box::into_raw(Box::new(TiledDisplay::new()));
		display = &mut *TILED_DISPLAY;
	}
	log(&format!("Drawing {} layers", file.get_mut().layer_count()));
	display.load_from(file.get_mut());
	log("Tiled loading done.");
}

impl Game {
	pub fn new() -> Game {
		//let line = Line::new(&Vec2::new(-5.0, 5.0), &Vec2::new(5.0, 5.0));
		//log(&format!("Line: {:?}", line));

		log("Setting up WASM game!");
		let mut player_draw = DisplayBuffer::new(DisplayBufferType::LINES);
		{
			let color = Color::new(128, 128, 255, 255);
			let mut editor = player_draw.make_editor();
			editor.add_circle(
				Vec3::zero(),
				PLAYER_RADIUS,
				25,
				&color,
			);
			editor.add_lines(
				vec![
					Vec3::new(-PLAYER_RADIUS, 0.0, 0.0),
					Vec3::new( PLAYER_RADIUS, 0.0, 0.0),
				],
				&color,
			);

			editor.add_lines(
				vec![
					Vec3::new(0.0,-PLAYER_RADIUS, 0.0),
					Vec3::new(0.0, PLAYER_RADIUS, 0.0),
				],
				&color,
			);
		}

		let mut background_draw = DisplayBuffer::new(DisplayBufferType::LINES);
		let mut collision = CollisionSystem::new();
		{
			let color = Color::new(0, 255, 0, 255);
			let outside = vec![
				Vec3::new(-100.0, 200.0, 0.0 ),
				Vec3::new(-100.0,-100.0, 0.0 ),
				Vec3::new( 100.0,-100.0, 0.0 ),
				Vec3::new( 100.0,  50.0, 0.0 ),
				Vec3::new(  50.0, 100.0, 0.0 ),
				Vec3::new(  50.0, 200.0, 0.0 ),
			];
			let mut editor = background_draw.make_editor();
			editor.add_polygon(
				&outside,
				&color,
			);
			for index in 1..outside.len() {
				collision.add_obstacle(CircleObstacle::LineSegment(
					LineSegment::new(
						&Vec2::new(outside[index-1].x, outside[index-1].y),
						&Vec2::new(outside[index  ].x, outside[index  ].y),
					)
				));
			}
			collision.add_obstacle(CircleObstacle::LineSegment(
				LineSegment::new(
					&Vec2::new(outside[outside.len()-1].x, outside[outside.len()-1].y),
					&Vec2::new(outside[              0].x, outside[              0].y),
				)
			));
			editor.add_lines(
				vec![
					Vec3::new(-100.0, -75.0, 0.0),
					Vec3::new( -50.0, -25.0, 0.0),
				],
				&color,
			);
			collision.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(-100.0, -75.0), &Vec2::new(-50.0, -25.0))));

			editor.add_circle(
				Vec3::new(-25.0, 100.0, 0.0),
				32.0,
				16,
				&color,
			);
			collision.add_obstacle(CircleObstacle::Circle(Circle::new(&Vec2::new(-25.0, 100.0), 32.0)));
		}

		let description = DisplayText::new_text_area(
			0.80,
			0.05,
			0.95,
			0.95,
			&Color::new(0, 255, 0, 255),
			TextAlignment::JUSTIFY,
			"Hit the arrow keys or WASD to move around.<br>Click to show mouse button tracking.",
		);

		let mut images_buffer = DisplayBuffer::new(DisplayBufferType::IMAGES);
		let mut images_texture = DisplayTexture::new();
		images_texture.load_from_url("roomTiles.png");
		{
			let mut editor = images_buffer.make_editor();
			editor.add_image(
				&Vec2::new(0.0, 0.0),
				&Vec2::new(128.0, 128.0),
				&Vec3::new(-20.0, 0.0, 0.0),
			);
			editor.add_image(
				&Vec2::new(0.0, 0.0),
				&Vec2::new(10.0, 10.0),
				&Vec3::new(-30.0, 5.0, 0.1),
			);
		}
		images_buffer.set_texture(&images_texture);
		images_buffer.hide();

		let mut images_buffer2 = DisplayBuffer::new(DisplayBufferType::IMAGES);
		let mut images_texture2 = DisplayTexture::new();
		images_texture2.load_from_url("player.png");
		{
			let mut editor = images_buffer2.make_editor();
			editor.add_image(
				&Vec2::new(0.0, 0.0),
				&Vec2::new(128.0, 128.0),
				&Vec3::new(30.0, 0.0, 0.0),
			);
			editor.add_image(
				&Vec2::new(0.0, 0.0),
				&Vec2::new(10.0, 10.0),
				&Vec3::new(50.0, 25.0, 0.1),
			);
		}
		images_buffer2.set_texture(&images_texture2);

		load_tiled_file("room.json",  test_do_load);

		Game {
			camera: Camera::new(),
			mouse: Mouse::new(),
			keyboard: Keyboard::new(),
			player: player_draw,
			player_position: Vec3::new(0.0, 0.0, 0.0),
			mouse_draw: DisplayBuffer::new(DisplayBufferType::LINES),
			background: background_draw,
			elapsed: 0.0,

			collision,

			description,
			middle_click_tracker: ClickTracker::new("middle"),
			right_click_tracker: ClickTracker::new("right"),

			bullets : Vec::new(),

			images: images_buffer,
			texture: images_texture,

			images2: images_buffer2,
			texture2: images_texture2,
		}
	}

	pub fn update(&mut self, elapsed_seconds : f32) {
		self.elapsed += elapsed_seconds;
		let mut movement = Vec3::zero();
		if self.keyboard.is_down(Key::UP) {
			movement.y += 1.0;
		}
		if self.keyboard.is_down(Key::LEFT) {
			movement.x -= 1.0;
		}
		if self.keyboard.is_down(Key::DOWN) {
			movement.y -= 1.0;
		}
		if self.keyboard.is_down(Key::RIGHT) {
			movement.x += 1.0;
		}
		if 0.0 < movement.length() {
			movement.set_length(elapsed_seconds * PLAYER_SPEED);

			let new_movement = self.collision.collide_circle(
				&Vec2::new(self.player_position.x, self.player_position.y),
				PLAYER_RADIUS,
				&Vec2::new(movement.x, movement.y),
			);
			movement.x = new_movement.x;
			movement.y = new_movement.y;

			self.player_position += movement;
			self.player.set_transform(Mat4::new().translate_before(&self.player_position));
		}

		{
			let mut index = 0;
			while index < self.bullets.len() {
				if !self.bullets[index].update(elapsed_seconds, &self.collision) {
					self.bullets.swap_remove(index);
				} else {
					index += 1;
				}
			}
		}

		if self.mouse.has_changed_since() {
			let bounds = self.camera.bounds();
			let position = self.mouse.position();
			{
				let mut editor = self.mouse_draw.make_editor();
				editor.clear();
				let color = Color::new(255, 255, 255, 255);
				editor.add_lines(
					vec![
						Vec3::new(bounds.x_min(), position.y, 0.0),
						Vec3::new(bounds.x_max(), position.y, 0.0),
					],
					&color,
				);
				editor.add_lines(
					vec![
						Vec3::new(position.x, bounds.y_min(), 0.0),
						Vec3::new(position.x, bounds.y_max(), 0.0),
					],
					&color,
				);
			}

			if self.mouse.is_button_down(MouseButton::MIDDLE) {
				self.middle_click_tracker.move_to(&self.mouse.position());
			}
			if self.mouse.is_button_down(MouseButton::RIGHT) {
				self.right_click_tracker.move_to(&self.mouse.position());
			}
		}

		if self.mouse.is_button_down(MouseButton::LEFT) {
			//self.left_click_tracker.move_to(&self.mouse.position());

			let mut velocity = self.mouse.position() - &self.player_position;
			velocity.set_length(300.0);
			self.bullets.push(Bullet::new(
				&Vec2::new(self.player_position.x, self.player_position.y),
				5.0,
				&Vec2::new(velocity.x, velocity.y),
			));
		}
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
}
