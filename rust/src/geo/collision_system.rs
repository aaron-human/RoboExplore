
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

/// The max number of iterations that collisions are allowed to go through.
const COLLISION_ITERATION_MAX : usize = 5;

/// An easy way to collide a Circle() collider against multiple other objects.
/// Will probably eventually also store a broad-phase collision filterer.
pub struct CollisionSystem {
	obstacles : Vec<CircleObstacle>,
}

impl CollisionSystem {
	/// Creates a new (empty) instance.
	pub fn new() -> CollisionSystem {
		CollisionSystem {
			obstacles: Vec::new(),
		}
	}

	/// Adds the given obstacle to the collidable geometry.
	pub fn add_obstacle(&mut self, obstacle : CircleObstacle) {
		self.obstacles.push(obstacle);
	}

	/// Collides a circle with the stored collision geometry, and returns the updated movement vector.
	pub fn collide_circle(&self, position : &Vec2, radius : f32, movement_ : &Vec2) -> Vec2 {
		let mut movement = movement_.clone();
		let mut circle = Circle::new(position, radius);
		for _iteration in 0..COLLISION_ITERATION_MAX {
			let mut hits : Vec<Option<Deflection>> = Vec::new();
			for generic_obstacle in &self.obstacles {
				match generic_obstacle {
					CircleObstacle::LineSegment(segment) => { hits.push((&circle).deflect_with(&movement, segment)); },
					CircleObstacle::Line(line)           => { hits.push((&circle).deflect_with(&movement, line)); },
					CircleObstacle::Point(position)      => { hits.push((&circle).deflect_with(&movement, position)); },
					CircleObstacle::Circle(obstacle) => {
						let augmented = Circle::new(&circle.center, circle.radius + obstacle.radius);
						hits.push((&augmented).deflect_with(&movement, &obstacle.center));
					},
				}
			}

			match Deflection::combine(hits) {
				Option::Some(collision) => {
					//log(&format!("Collision: {:?} + {:?}", circle, movement));
					circle.center = collision.position;
					movement = collision.remainder;
					if movement.length() < EPSILON {
						return circle.center - position;
					}
				},
				Option::None => {
					//log(&format!("No hit: {:?} + {:?}", circle, movement));
					return circle.center + movement - position;
				},
			}
		}
		log("Hit collision iteration max!");
		circle.center + movement - position
	}
}

#[cfg(test)]
mod test_collision_system {
	use super::*;
	use crate::{assert_about_eq, assert_vec2_about_eq}; // I have to export macros to the top-level module to be able to share them across modules in the same crate... Yeah, I can't imagine that leading to bad things in the Rust ecosystem! Why is this so difficult?

	#[test]
	fn line_segment_stop() { // Majke sure the line segment works.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(2.0, 2.0), &Vec2::new(2.0, -2.0))));
		let result = system.collide_circle(&Vec2::new(0.0, 1.0), 1.0, &Vec2::new(2.0, 0.0));
		assert_vec2_about_eq!(result, Vec2::new(1.0, 0.0));
	}

	#[test]
	fn line_stop() { // Majke sure the line works.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::Line(Line::new(&Vec2::new(2.0, 2.0), &Vec2::new(2.0, -2.0))));
		let result = system.collide_circle(&Vec2::new(0.0, 10.0), 1.0, &Vec2::new(2.0, 0.0));
		assert_vec2_about_eq!(result, Vec2::new(1.0, 0.0));
	}

	#[test]
	fn point_stop() { // Majke sure the point works.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::Point(Vec2::new(0.0, 3.0)));
		let result = system.collide_circle(&Vec2::new(0.0, 1.0), 1.0, &Vec2::new(0.0, 2.0));
		assert_vec2_about_eq!(result, Vec2::new(0.0, 1.0));
	}

	#[test]
	fn acute_corner() { // Make sure going into a corner halts movement. And can then leave.
		let mut system = CollisionSystem::new();
		system.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(-2.0, 2.0), &Vec2::new(2.0,-2.0))));
		system.add_obstacle(CircleObstacle::LineSegment(LineSegment::new(&Vec2::new(-2.0,-2.0), &Vec2::new(2.0,-2.0))));
		const RADIUS : f32 = 1.0;
		let start = Vec2::new(-2.0, 0.0);
		let mut delta = system.collide_circle(&start, RADIUS, &Vec2::new(6.0, 0.0));
		let stuck = &start + &delta;
		delta = system.collide_circle(&stuck, RADIUS, &Vec2::new(2.0, -1.0));
		println!("Delta = {:?} @ {:?}", delta, stuck);
		assert_about_eq!(delta.length(), 0.0);
		let freedom = Vec2::new(-2.0, 1.0);
		let freed = system.collide_circle(&stuck, RADIUS, &freedom);
		assert_vec2_about_eq!(freed, freedom);
	}
}
