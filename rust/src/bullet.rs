use crate::geo::consts::*;
use crate::geo::vec2::*;
use crate::geo::vec3::*;
use crate::geo::mat4::*;
use crate::geo::circle::*;
use crate::color::*;
use crate::geo::collision_system::*;
use crate::display_buffer::*;

pub struct Bullet {
	shape : Circle,
	velocity : Vec2,
	draw : DisplayBuffer,
}

impl Bullet {
	/// Creates a new bullet.
	pub fn new(position : &Vec2, radius : f32, velocity : &Vec2) -> Bullet {
		let mut draw = DisplayBuffer::new(DisplayBufferType::SOLIDS);
		{
			let mut editor = draw.make_editor();
			editor.add_circle(Vec3::zero(), radius, 7, &Color::new(255, 0, 0, 255));
		}
		draw.set_transform(Mat4::new().translate_before(&Vec3::new(position.x, position.y, 0.0)));
		Bullet{
			shape: Circle::new(position, radius),
			velocity: velocity.clone(),
			draw,
		}
	}

	/// Updates the bullet. Returns if the bullet should stay alive.
	pub fn update(&mut self, elapsed_seconds : f32, collision : &CollisionSystem) -> bool {
		// TODO: Make the below more efficient.
		let mut movement = self.velocity.scale(elapsed_seconds);
		let collisions = collision.collide_circle(&self.shape.center, self.shape.radius, &movement);
		if let Some(collision) = collisions.last() {
			let new_position = collision.final_position;
			let new_movement = new_position - self.shape.center;
			if EPSILON < (&new_movement - movement).length() {
				return false;
			}
			movement = new_movement;
		}
		self.shape.center += movement;
		self.draw.set_transform(self.draw.get_transform().translate_before(&Vec3::new(movement.x, movement.y, 0.0)));
		true
	}

	// TODO: implement a drop function that deletes the display buffer.
}
