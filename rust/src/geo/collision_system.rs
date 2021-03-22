use generational_arena::{Arena, Index};

use crate::externals::log;

use super::consts::*;
use super::vec2::*;
use super::line::*;
use super::line_segment::*;
use super::circle::*;
use super::collider::*;

/// The types of obstacles that a Circle() collider can hit.
pub enum CircleObstacle {
	LineSegment(LineSegment),
	Line(Line),
	Point(Vec2),
	Circle(Circle),
}

/// A general object representing a specific piece of collision geometry.
pub struct CollisionObstacle {
	/// The CircleObstacle that is what's collided against.
	pub geometry : CircleObstacle,
	/// Whether this obstacle should be collided against.
	pub active : bool,
}

/// The max number of iterations that collisions are allowed to go through.
const COLLISION_ITERATION_MAX : usize = 5;

/// An easy way to collide a Circle() collider against multiple other objects.
/// Will probably eventually also store a broad-phase collision filterer.
pub struct CollisionSystem {
	/// All the obstacles being collided with.
	pub obstacles : Arena<CollisionObstacle>,
}

impl CollisionSystem {
	/// Creates a new (empty) instance.
	pub fn new() -> CollisionSystem {
		CollisionSystem {
			obstacles: Arena::new(),
		}
	}

	/// Adds the given obstacle to the collidable geometry.
	pub fn add_obstacle(&mut self, obstacle : CircleObstacle) -> Index {
		self.obstacles.insert(CollisionObstacle{
			geometry : obstacle,
			active : true,
		})
	}

	/// Let users easily enable/disable a specific obstacle.
	pub fn set_enabled(&mut self, index : Index, enabled : bool) {
		self.obstacles.get_mut(index).unwrap().active = enabled;
	}

	/// Collides a circle with the stored collision geometry, and returns the updated movement vector.
	pub fn collide_circle(&self, position_ : &Vec2, radius : f32, movement_ : &Vec2) -> Vec<TotalDeflection> {
		let mut movement = movement_.clone();
		let mut position = position_.clone();
		let mut result : Vec<TotalDeflection> = Vec::new();
		for _iteration in 0..COLLISION_ITERATION_MAX {
			if let Some(total_deflection) = self.collide_circle_step(&position, radius, &movement) {
				let collision = &total_deflection.deflections[0];
				position = collision.position;
				movement = total_deflection.final_position - collision.position;
				result.push(total_deflection);
				if movement.length() < EPSILON {
					return result;
				}
			} else {
				return result;
			}
		}
		log("Hit collision iteration max!");
		return result;
	}

	/// Perform one round of collision detection and send all the information to the caller.
	pub fn collide_circle_step(&self, position : &Vec2, radius : f32, movement : &Vec2) -> Option<TotalDeflection> {
		let circle = Circle::new(position, radius);
		let mut hits : Vec<Deflection> = Vec::new();
		for (index, generic_obstacle) in &self.obstacles {
			if !generic_obstacle.active { continue; }
			let maybe_deflection = match &generic_obstacle.geometry {
				CircleObstacle::LineSegment(segment) => { (&circle).deflect_with(movement, segment) },
				CircleObstacle::Line(line)           => { (&circle).deflect_with(movement, line) },
				CircleObstacle::Point(position)      => { (&circle).deflect_with(movement, position) },
				CircleObstacle::Circle(obstacle) => {
					let augmented = Circle::new(&circle.center, circle.radius + obstacle.radius);
					(&augmented).deflect_with(movement, &obstacle.center)
				},
			};
			if let Some(mut deflection) = maybe_deflection {
				deflection.source = index;
				hits.push(deflection);
			}
		}

		TotalDeflection::try_new(hits)
	}
}

#[cfg(test)]
mod test_collision_system {
	use super::*;
	use crate::assert_vec2_about_eq; // I have to export macros to the top-level module to be able to share them across modules in the same crate... Yeah, I can't imagine that leading to bad things in the Rust ecosystem! Why is this so difficult?

	#[test]
	fn line_segment_stop() { // Make sure the line segment works.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(2.0, 2.0), &Vec2::new(2.0, -2.0))));
		let result = system.collide_circle(&Vec2::new(0.0, 1.0), 1.0, &Vec2::new(2.0, 0.0));
		assert_eq!(result.len(), 1);
		assert_vec2_about_eq!(result[0].final_position, Vec2::new(1.0, 1.0));
	}

	#[test]
	fn line_stop() { // Make sure the line works.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::Line(Line::new(&Vec2::new(2.0, 2.0), &Vec2::new(2.0, -2.0))));
		let result = system.collide_circle(&Vec2::new(0.0, 10.0), 1.0, &Vec2::new(2.0, 0.0));
		assert_eq!(result.len(), 1);
		assert_vec2_about_eq!(result[0].final_position, Vec2::new(1.0, 10.0));
	}

	#[test]
	fn point_stop() { // Make sure the point works.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::Point(Vec2::new(0.0, 3.0)));
		let result = system.collide_circle(&Vec2::new(0.0, 1.0), 1.0, &Vec2::new(0.0, 2.0));
		assert_eq!(result.len(), 1);
		assert_vec2_about_eq!(result[0].final_position, Vec2::new(0.0, 2.0));
	}

	#[test]
	fn acute_corner() { // Make sure going into a corner halts movement. And can then leave.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(-2.0, 2.0), &Vec2::new(2.0,-2.0))));
		system.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(-2.0,-2.0), &Vec2::new(2.0,-2.0))));
		const RADIUS : f32 = 1.0;
		let start = Vec2::new(-2.0, 0.0);
		let mut collisions = system.collide_circle(&start, RADIUS, &Vec2::new(6.0, 0.0));
		assert!(0 < collisions.len());
		let stuck = collisions.last().unwrap().final_position;

		collisions = system.collide_circle(&stuck, RADIUS, &Vec2::new(2.0, -1.0));
		assert!(0 < collisions.len());
		let final_position = collisions.last().unwrap().final_position;
		assert_vec2_about_eq!(final_position, stuck);

		let freedom = Vec2::new(-2.0, 1.0);
		collisions = system.collide_circle(&stuck, RADIUS, &freedom);
		assert_eq!(collisions.len(), 0);
	}
}
