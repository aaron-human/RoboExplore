use generational_arena::Index;

use super::consts::*;
use super::range::*;
use super::vec2::*;
use super::line::*;
use super::line_segment::*;
use super::collider::*;

/// A 2D circle.
#[derive(Debug, Copy, Clone)]
pub struct Circle {
	pub center : Vec2, // The center.
	pub radius : f32, // The radius.
}

impl Circle {
	pub fn new(center : &Vec2, radius : f32) -> Circle {
		Circle { center: center.clone(), radius }
	}
}

impl<'l> Collider<'l, Line> for Circle {
	/// Deflects a collider's movement with the given obstacle.
	fn deflect_with(&self, movement : &Vec2, obstacle : &'l Line) -> Option<Deflection> {
		let mut deflection = Deflection{
			times: Range::empty(),
			normal: (&obstacle.delta).ortho_like(&self.center - &obstacle.origin),
			deflected: false, // Assume not deflected until go through that part.
			position: self.center.clone(),
			remainder: movement.clone(),
			source: Index::from_raw_parts(0, 0), // A generic index that will be replaced by the caller.
		};
		println!("normal: {:?}", &deflection.normal);

		// Push the start of the line out if it's too close.
		let mut ortho = (&self.center - &obstacle.origin).ext(&obstacle.delta);
		let mut ortho_dist = ortho.abs();
		let moved = if ortho_dist < self.radius {
			(&mut deflection.times).cover(0.0); // Since had to move out of line, will be in contact at least at the very start.
			deflection.position += (&deflection.normal).scale(self.radius - ortho_dist);
			// Recalculate the ortho and ortho_dist now that the starting point has moved.
			ortho = (&deflection.position - &obstacle.origin).ext(&obstacle.delta);
			ortho_dist = ortho.abs();
			true
		} else {
			false
		};
		println!("ortho: {:?}; moved = {:?}", ortho, moved);

		// Find if/when the movement would hit.
		let denom = movement.ext(&obstacle.delta);
		println!("denom: {:?}", denom);
		if denom.abs() < EPSILON && (ortho_dist - self.radius).abs() < EPSILON {
			println!("Found skimming hit.");
			// If start just touching and are moving parallel to the line, then it's skimming.
			deflection.times.make_all();
			return Some(deflection);
		}
		(&mut deflection.times).cover(Range::from_values(
			(-ortho - self.radius) / denom,
			(-ortho + self.radius) / denom,
		));

		// If not time between 0.0 and 1.0, then no hit happened.
		println!("times = {:?}", &deflection.times);
		if deflection.split_remainder() {
			println!("Gave up: all times invalid.");
			// Both out of range, then there was no hit.
			// And since pushing out the start always adds 0.0 to times, should always return 'None' here...
			return None;
		}

		// Then calculate the deflection. Always return Some at this point (did contact the line), but it won't always have `deflected` set to true.
		deflection.calc_deflection();
		Some(deflection)
	}
}

#[cfg(test)]
mod test_line_deflect {
	use super::*;
	use crate::{assert_about_eq, assert_vec2_about_eq, assert_lt, assert_gt}; // I have to export macros to the top-level module to be able to share them across modules in the same crate... Yeah, I can't imagine that leading to bad things in the Rust ecosystem! Why is this so difficult?

	#[test]
	fn no_hit_parallel() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			5.0,
		);
		let line = Line::new(
			&Vec2::new(10.0, 0.0),
			&Vec2::new(0.0, 10.0),
		);
		let result = circle.deflect_with(&Vec2::new(10.0, -10.0), &line);
		assert!(result.is_none());
	}

	#[test]
	fn no_hit_too_short() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			3.0,
		);
		let line = Line::new(
			&Vec2::new(10.0, 0.0),
			&Vec2::new(0.0, 10.0),
		);
		let result = circle.deflect_with(&Vec2::new(1.0, 1.0), &line);
		assert!(result.is_none());
	}

	#[test]
	fn no_hit_just_touch() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let line = Line::new(
			&Vec2::new(2.0, 10.0),
			&Vec2::new(2.0,-10.0),
		);
		let result = circle.deflect_with(&Vec2::new(1.0, 0.0), &line);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 1.0);
		assert_vec2_about_eq!(hit.normal, Vec2::new(-1.0, 0.0));
		assert_eq!(hit.deflected, false);
	}

	#[test]
	fn no_hit_skim() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let line = Line::new(
			&Vec2::new(-5.0, 1.0),
			&Vec2::new( 5.0, 1.0),
		);
		let result = circle.deflect_with(&Vec2::new(1.0, 0.0), &line);
		let hit = result.unwrap();
		assert_lt!(hit.times.min().unwrap(), 0.0);
		assert_gt!(hit.times.max().unwrap(),-1.0);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0,-1.0));
		assert_eq!(hit.deflected, false);
	}

	#[test]
	fn no_hit_away() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let line = Line::new(
			&Vec2::new(10.0, 0.0),
			&Vec2::new(0.0, 10.0),
		);
		let result = circle.deflect_with(&Vec2::new(-10.0, -10.0), &line);
		assert!(result.is_none());
	}

	#[test]
	fn no_hit_off_wall() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let line = Line::new(
			&Vec2::new(-10.0, -1.0),
			&Vec2::new( 10.0, -1.0),
		);
		let result = circle.deflect_with(&Vec2::new(0.0, 10.0), &line);
		let hit = result.unwrap();
		assert_lt!(hit.times.min().unwrap(), 0.0);
		assert_about_eq!(hit.times.max().unwrap(), 0.0);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, 1.0));
		assert_eq!(hit.deflected, false);
	}

	#[test]
	fn hit_stop() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let line = Line::new(
			&Vec2::new(2.0, 10.0),
			&Vec2::new(2.0,-10.0),
		);
		let result = circle.deflect_with(&Vec2::new(2.0, 0.0), &line);
		assert!(result.is_some());
		let hit = result.unwrap();
		assert_eq!(hit.times.min().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(-1.0, 0.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(1.0, 0.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0, 0.0));
	}

	#[test]
	fn hit_deflect() {
		let circle = Circle::new(
			&Vec2::new(0.0, 1.0),
			1.0,
		);
		let line = Line::new(
			&Vec2::new(-2.0, 10.0),
			&Vec2::new(-2.0,-10.0),
		);
		let result = circle.deflect_with(&Vec2::new(-2.0, -2.0), &line);
		assert!(result.is_some());
		let hit = result.unwrap();
		assert_eq!(hit.times.min().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(1.0, 0.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(-1.0, 0.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0,-1.0));
	}

	#[test]
	fn start_inside() {
		let circle = Circle::new(
			&Vec2::new(1.0, 0.5),
			2.0,
		);
		let line = Line::new(
			&Vec2::new(-5.0, 0.0),
			&Vec2::new( 5.0, 0.0),
		);
		let result = circle.deflect_with(&Vec2::new(1.0, 1.0), &line);
		assert!(result.is_some());
		let hit = result.unwrap();
		assert_lt!(hit.times.min().unwrap(), 0.0);
		assert_about_eq!(hit.times.max().unwrap(), 0.0);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, 1.0));
		assert_eq!(hit.deflected, false);
	}
}

impl<'l> Collider<'l, Vec2> for Circle {
	/// Deflects a collider's movement with the given obstacle.
	fn deflect_with(&self, movement : &Vec2, obstacle : &'l Vec2) -> Option<Deflection> {
		// Check if starting inside.
		let mut position = self.center.clone();
		{
			let mut outward = &self.center - obstacle;
			let push_out_distance = self.radius - (&outward).length();
			if 0.0 < push_out_distance {
				(&mut outward).set_length(push_out_distance);
				position += outward;
			}
		}

		// Find when it would hit (if ever).
		let start_offset = &position - obstacle;
		let mut deflection = Deflection{
			times: Range::from_quadratic_zeros(
				(movement).dot(movement),
				2.0 * (&start_offset).dot(movement),
				(&start_offset).dot(&start_offset) - self.radius * self.radius,
			),
			normal: Vec2::zero(),
			deflected: false, // Assume not deflected until go through that part.
			position,
			remainder: movement.clone(),
			source: Index::from_raw_parts(0, 0), // A generic index that will be replaced by the caller.
		};

		// If not time between 0.0 and 1.0, then no hit happened.
		println!("times = {:?}", &deflection.times);
		let bounded = (&deflection.times).intersect(Range::from_values(0.0, 1.0));
		if bounded.is_empty() {
			println!("Gave up: all times invalid.");
			// Both out of range, then there was no hit.
			// And since pushing out the start always adds 0.0 to times, should always return 'None' here...
			return None;
		}

		// TODO: Special handling if it's bascially at the end?
		// If not time between 0.0 and 1.0, then no hit happened and dflect from there.
		let time = bounded.min().unwrap();
		deflection.position += movement.scale(time);
		deflection.normal = (&deflection.position - obstacle).norm();
		// TODO: Factor out everything below here into a method on Delfection. Then let this code and the code for Line share that call.
		(&mut deflection.remainder).scale(1.0 - time);

		// Then calculate the deflection. Always return Some at this point (did contact the line), but it won't always have `deflected` set to true.
		deflection.calc_deflection();
		Some(deflection)
	}
}

#[cfg(test)]
mod test_point_deflect {
	use super::*;
	use crate::{assert_vec2_about_eq, assert_about_eq, assert_gt};

	#[test]
	fn no_hit_basic() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			5.0,
		);
		let point = Vec2::new(1.0, -6.0);
		let result = circle.deflect_with(&Vec2::new(1.0, 0.0), &point);
		assert!(result.is_none());
	}

	#[test]
	fn no_hit_too_short() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let point = Vec2::new(10.0, 0.0);
		let result = circle.deflect_with(&Vec2::new(1.0, 0.0), &point);
		assert!(result.is_none());
	}

	#[test]
	fn no_hit_moving_away() {
		let circle = Circle::new(
			&Vec2::new(0.0, 0.0),
			1.0,
		);
		let point = Vec2::new(1.0, 0.0);
		let result = circle.deflect_with(&Vec2::new(-1.0, 0.0), &point);
		let hit = result.unwrap();
		assert_eq!(hit.times.max().unwrap(), 0.0);
		assert_vec2_about_eq!(hit.normal, Vec2::new(-1.0, 0.0));
		assert_eq!(hit.deflected, false);
	}

	#[test]
	fn no_hit_orthogonal_skim() {
		let circle = Circle::new(
			&Vec2::new(-1.0, 1.0),
			1.0,
		);
		let point = Vec2::new(0.0, 2.0);
		let result = circle.deflect_with(&Vec2::new(2.0, 0.0), &point);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_about_eq!(hit.times.max().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, -1.0));
		assert_eq!(hit.deflected, false);
	}

	#[test]
	fn start_inside_no_move() {
		let circle = Circle::new(
			&Vec2::new(0.0, 1.0),
			1.0,
		);
		let point = Vec2::new(0.0, 1.5);
		let result = circle.deflect_with(&Vec2::new(0.0, 0.0), &point);
		let hit = result.unwrap();
		assert!(hit.times.contains(0.0));
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, -1.0));
		assert_eq!(hit.deflected, false);
		assert_vec2_about_eq!(hit.position, Vec2::new(0.0, 0.5));
	}

	#[test]
	fn hit_stop() {
		let circle = Circle::new(
			&Vec2::new(1.0, 1.0),
			1.0,
		);
		let point = Vec2::new(3.0, 1.0);
		let result = circle.deflect_with(&Vec2::new(2.0, 0.0), &point);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_about_eq!(hit.times.max().unwrap(), 1.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(-1.0, 0.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(2.0, 1.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0, 0.0));
	}

	#[test]
	fn hit_deflect() {
		let circle = Circle::new(
			&Vec2::new(1.0, 1.0),
			2.0_f32.sqrt(),
		);
		let point = Vec2::new(3.0, 0.0);
		let result = circle.deflect_with(&Vec2::new(2.0, 0.0), &point);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_gt!(      hit.times.max().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(-1.0, 1.0).norm());
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(2.0, 1.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.5, 0.5));
	}
}

/// For just the deflections that occur when the circle hits the straight parts of a line-segment's deflection geometry (i.e. parts between the end points as opposted to the rounded end-point caps).
fn deflect_with_line_segment_middle(circle : &Circle, movement : &Vec2, obstacle : &LineSegment) -> Option<Deflection> {
	let mut deflection = Deflection{
			times: Range::empty(),
			normal: (&obstacle.direction).ortho_like(&circle.center - &obstacle.start),
			deflected: false, // Assume not deflected until go through that part.
			position: circle.center.clone(),
			remainder: movement.clone(),
			source: Index::from_raw_parts(0, 0), // A generic index that will be replaced by the caller.
		};
		println!("normal: {:?}", &deflection.normal);

		// Push the start of the line out if it's too close.
		let starting_offset = &circle.center - &obstacle.start; // Diff
		let mut ortho = (&starting_offset).ext(&obstacle.direction); // Diff
		let mut ortho_dist = ortho.abs();
		let mut distance_along = starting_offset.dot(&obstacle.direction); // Diff
		let moved = if ortho_dist < circle.radius && 0.0 < distance_along && distance_along < obstacle.length { // Diff
			(&mut deflection.times).cover(0.0); // Since had to move out of line, will be in contact at least at the very start.
			deflection.position += (&deflection.normal).scale(circle.radius - ortho_dist);
			// Recalculate the ortho and ortho_dist now that the starting point has moved.
			ortho = (&deflection.position - &obstacle.start).ext(&obstacle.direction);
			ortho_dist = ortho.abs();
			true
		} else {
			false
		};
		println!("ortho: {:?}; moved = {:?}", ortho, moved);

		// TODO: Deduplicate the normal finding code. (Into where?)
		// Find if/when the movement would hit.
		let denom = movement.ext(&obstacle.direction);
		println!("denom: {:?}", denom);
		if denom.abs() < EPSILON && (ortho_dist - circle.radius).abs() < EPSILON {
			println!("Found skimming hit.");
			// If start just touching and are moving parallel to the line, then it's skimming.
			deflection.times.make_all();
			return Some(deflection);
		}
		(&mut deflection.times).cover(Range::from_values(
			(-ortho - circle.radius) / denom,
			(-ortho + circle.radius) / denom,
		));

		// TODO: Deduplicate the normal finding code. (Into where?)
		// If not time between 0.0 and 1.0, then no hit happened.
		println!("times = {:?}", &deflection.times);
		if deflection.split_remainder() {
			println!("Gave up: all times invalid.");
			// Both out of range, then there was no hit.
			// And since pushing out the start always adds 0.0 to times, should always return 'None' here...
			return None;
		}

		// Make sure the deflection occurs in between the end points. If not then there is no deflection.
		distance_along = (&deflection.position - &obstacle.start).dot(&obstacle.direction);
		if 0.0 > distance_along || distance_along > obstacle.length {
			// Hit outside of the line segment.
			println!("Gave up: beyond line segment edges @ {:?} vs [0.0 to {:?}].", distance_along, obstacle.direction);
			return if moved { Some(deflection) } else { None };
		}

		// Then calculate the deflection. Always return Some at this point (did contact the line), but it won't always have `deflected` set to true.
		deflection.calc_deflection();
		Some(deflection)
}

#[cfg(test)]
mod test_line_segment_middle_deflect { // Just testing things that are different from Line, since copied that collider to get things started...
	use super::*;
	use crate::{assert_vec2_about_eq, assert_about_eq};

	#[test]
	fn push_out() {
		let circle = Circle::new(
			&Vec2::new(-1.0, 1.0),
			2.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, 0.0), &Vec2::new(5.0, 0.0));
		let result = deflect_with_line_segment_middle(&circle, &Vec2::new(1.0, 1.0), &seg);
		let hit = result.unwrap();
		assert_eq!(hit.times.max().unwrap(), 0.0);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, 1.0));
		assert_eq!(hit.deflected, false);
		assert_vec2_about_eq!(hit.position, Vec2::new(-1.0, 2.0));
	}

	#[test]
	fn no_push_out() {
		let circle = Circle::new(
			&Vec2::new(-1.0, 1.0),
			2.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, 0.0), &Vec2::new(-15.0, 0.0));
		let result = deflect_with_line_segment_middle(&circle, &Vec2::new(1.0, 1.0), &seg);
		assert!(result.is_none());
	}

	#[test]
	fn hit_beyond_end_points() {
		let circle = Circle::new(
			&Vec2::new(-1.0, 3.0),
			1.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, 0.0), &Vec2::new(-15.0, 0.0));
		let result = deflect_with_line_segment_middle(&circle, &Vec2::new(0.0, -6.0), &seg);
		assert!(result.is_none());
	}

	#[test]
	fn hit_between_end_points() {
		let circle = Circle::new(
			&Vec2::new(-1.0, 2.0),
			1.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, 0.0), &Vec2::new(5.0, 0.0));
		let result = deflect_with_line_segment_middle(&circle, &Vec2::new(0.0, -2.0), &seg);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, 1.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(-1.0, 1.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0, 0.0));
	}
}

impl<'l> Collider<'l, LineSegment> for Circle {
	/// Deflects a collider's movement with the given obstacle.
	fn deflect_with(&self, movement : &Vec2, obstacle : &'l LineSegment) -> Option<Deflection> {
		let mut deflections = Vec::new();
		if let Some(deflection) = self.deflect_with(movement, &obstacle.start) {
			deflections.push(deflection);
		}
		if let Some(deflection) = deflect_with_line_segment_middle(self, movement, obstacle) {
			deflections.push(deflection);
		}
		if let Some(deflection) = self.deflect_with(movement, &obstacle.end) {
			deflections.push(deflection);
		}
		if let Some(mut total) = TotalDeflection::try_new(deflections) {
			Some(total.deflections.remove(0))
		} else {
			None
		}
	}
}

#[cfg(test)]
mod test_line_segment_deflect { // Testing lightly as there's a lot of code that's shared with already-tested code...
	use super::*;
	use crate::{assert_vec2_about_eq, assert_about_eq};

	#[test]
	fn complete_miss() {
		let circle = Circle::new(
			&Vec2::new(1.0, 1.0),
			1.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, -1.0), &Vec2::new(5.0, -1.0));
		let result = circle.deflect_with(&Vec2::new(1.0, 1.0), &seg);
		assert!(result.is_none());
	}

	#[test]
	fn hit_middle() {
		let circle = Circle::new(
			&Vec2::new(1.0, 1.0),
			1.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, -1.0), &Vec2::new(5.0, -1.0));
		let result = circle.deflect_with(&Vec2::new(0.0, -2.0), &seg);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(0.0, 1.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(1.0, 0.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0, 0.0));
	}

	#[test]
	fn hit_start() {
		let circle = Circle::new(
			&Vec2::new(-7.0, -1.0),
			1.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, -1.0), &Vec2::new(5.0, -1.0));
		let result = circle.deflect_with(&Vec2::new(2.0, 0.0), &seg);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(-1.0, 0.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(-6.0, -1.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0, 0.0));
	}

	#[test]
	fn hit_end() {
		let circle = Circle::new(
			&Vec2::new(7.0, -1.0),
			1.0,
		);
		let seg = LineSegment::new(&Vec2::new(-5.0, -1.0), &Vec2::new(5.0, -1.0));
		let result = circle.deflect_with(&Vec2::new(-2.0, 0.0), &seg);
		let hit = result.unwrap();
		assert_about_eq!(hit.times.min().unwrap(), 0.5);
		assert_vec2_about_eq!(hit.normal, Vec2::new(1.0, 0.0));
		assert_eq!(hit.deflected, true);
		assert_vec2_about_eq!(hit.position, Vec2::new(6.0, -1.0));
		assert_vec2_about_eq!(hit.remainder, Vec2::new(0.0, 0.0));
	}
}
