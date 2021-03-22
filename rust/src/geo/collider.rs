use generational_arena::Index;

use std::f32::INFINITY;

use super::consts::*;
use super::range::*;
use super::vec2::*;

/// The result of deflecting a collider.
#[derive(Debug, Clone)]
pub struct Deflection {
	/// When in contact with the obstacle. Zero means at start. One means at end.
	pub times : Range,
	/// The surface normal. Should be pointing away from the surface. Must be unit length.
	pub normal : Vec2,
	/// Whether a deflection occurred. If false, then just passing back that was in contact with some surface.
	pub deflected : bool,

	/// The position of the collider when the deflection happened.
	pub position : Vec2,
	/// The remaining (deflected) movement.
	pub remainder : Vec2,

	/// A way to keep track of which piece of collision geometry caused this.
	pub source : Index,
}

pub trait Collider<'o, OBSTACLE> {
	/// Deflects a collider's movement with the given obstacle.
	fn deflect_with(&self, movement : &Vec2, obstacle : &'o OBSTACLE) -> Option<Deflection>;
}

impl Deflection {

	/// Completes a common part of deflection calculation: verifying that the `times` is between 0.0 and 1.0, and using it to split the movement between `position` and `remainder`.
	/// For this to work, `times` must be the final collision time value, `position` must be the starting position, and `remainder` must be the full movement.
	/// This will return if `times` was INVALID (i.e. not between 0.0 and 1.0). Should quit out if that's the case.
	pub fn split_remainder(&mut self) -> bool { // NOTE: This is tested through circle.rs
		let bounded = (&self.times).intersect(Range::from_values(0.0, 1.0));
		if !bounded.is_empty() {
			// Calculate the hit position if those times checked out.
			// Need this for the next code block to make sense.
			let time = bounded.min().unwrap();
			self.position += (&self.remainder).scale(time);
			(&mut self.remainder).scale(1.0 - time);
			false
		} else {
			true
		}
	}

	/// Completes calculating a deflection.
	/// This assumes that `split_remainder()` has basically been run. In other words: `normal` must be the surface-normal (toward the collider), and `remainder` must have been scaled to be the remainder of the movement.
	/// This will set `deflected` and `remainder` accordingly.
	pub fn calc_deflection(&mut self) { // NOTE: This is tested through circle.rs
		// TODO: Special handling if it's bascially at the end?
		// If moving in same direction as normal, then no hit happened, but skimmed, didn't hit.
		let coincidence = (&self.remainder).dot(&self.normal);
		if -EPSILON <= coincidence {
			println!("Gave up: coindicence = {:?}.", coincidence);
			self.deflected = false;
			// Positive or zero coincidence means moving away from wall or perpendicular to it.
		} else {
			// At this point you've definitely hit and deflected.
			println!("Deflected!");
			self.deflected = true;
			self.remainder += (&self.normal).scale(-coincidence); // TODO: This is redundant with limit_movement_with_normals()!
		}
	}
}

/// The result of combining one or more Deflection()s together.
#[derive(Debug)]
pub struct TotalDeflection {
	/// The final position of the collider after the collision.
	pub final_position : Vec2,
	/// All unique surface normals found at the point of collision.
	pub normals : Vec<Vec2>,
	/// All of the deflections that applied during the collision.
	/// This includes ones that were in contact but didn't alter the movement.
	///
	/// The first is always the one that's considered **the** deflection that occurred.
	pub deflections : Vec<Deflection>,
}

/// Limits the given input vector according to a set of normals passed in.
pub fn limit_movement_with_normals(movement : &Vec2, normals : &Vec<Vec2>) -> Vec2 {
	// Try limiting the remainder vector if there's more than one normal.
	// Do this mainly by checking if there are normals on the left and right of the remaining movement. That means drop the remaining movement.
	let mut on_pos : bool = false;
	let mut on_neg : bool = false;
	let mut result = movement.clone();
	for normal in normals {
		// Ignore if the normal is in the direction of movement.
		// But don't ignore if it's perpendicular.
		if movement.dot(normal) > EPSILON { continue; }
		// Check which side it is on.
		if 0.0 > movement.ext(normal) {
			on_pos = true;
		} else {
			on_neg = true;
		}
		// Remove the normal from it.
		result -= normal.scale(normal.dot(&result));
		if (on_pos && on_neg) {// || (result.length() < EPSILON) {
			return Vec2::new(0.0, 0.0);
		}
	}
	result
}

impl TotalDeflection {
	/// Combines multiple Deflections.
	/// Always yields the nearest. If there are multiple that fit that description, chooses first.
	/// If more than one unique normal applies at that time, then will try to apply the new ones. This will generally zero any movement toward two unique normals (not 100% sure if there's a better way).
	pub fn try_new(mut items : Vec<Deflection>) -> Option<TotalDeflection> {
		// First pass: find the Deflection with the earliest start time.
		let mut soonest_time : f32 = INFINITY;
		let mut soonest_index : usize = items.len();
		for index in 0..items.len() {
			let new = &items[index];
			if new.deflected {
				let mut new_time = new.times.min().unwrap();
				if 0.0 > new_time { new_time = 0.0; }
				if new_time < soonest_time {
					soonest_time = new_time;
					soonest_index = index;
				}
			}
		}

		// If no soonest, then no collision.
		if soonest_index == items.len() {
			return None;
		}

		// Then extract soonest_index, so can easily work on all the rest.
		let deflection = items.remove(soonest_index);

		// Second pass: remove all deflections that don't "apply" at the soonest_time.
		// Also setup a list of unqiue normals.
		let mut normals : Vec<Vec2> = Vec::new();
		normals.push(deflection.normal.clone());
		items.retain(|hit| {
			let keep = hit.times.contains(soonest_time);
			if keep {
				let new_normal = hit.normal.clone();
				let mut unique = true;
				for norm in &normals {
					if (norm - &new_normal).length() < EPSILON {
						unique = false;
						break;
					}
				}
				if unique {
					normals.push(new_normal);
				}
			}
			keep
		});

		// Always put the soonest_index first.
		let remainder = deflection.remainder.clone();
		items.insert(0, deflection);

		// Try limiting the remainder vector with the surface normal(s).
		let final_remainder = limit_movement_with_normals(&remainder, &normals);

		Some(TotalDeflection{
			final_position: items[0].position + final_remainder,
			normals,
			deflections: items,
		})
	}

}

#[cfg(test)]
mod test_combine_deflection {
	use super::*;

	#[test]
	fn no_hits() {
		let result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::empty(),
				normal: Vec2::zero(),
				deflected: false,
				position:  Vec2::zero(),
				remainder: Vec2::zero(),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		assert!(result.is_none());
	}

	#[test]
	fn pass_through() {
		let maybe_result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::from_value(0.5),
				normal: Vec2::new(1.0, 0.0),
				deflected: true,
				position:  Vec2::new(0.0, 1.0),
				remainder: Vec2::new(1.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		let result = maybe_result.unwrap();
		let hit = &result.deflections[0];
		assert_eq!(hit.times.min().unwrap(), 0.5);
		assert_eq!(hit.times.max().unwrap(), 0.5);
		assert_eq!(hit.normal.x, 1.0);
		assert_eq!(hit.normal.y, 0.0);
		assert_eq!(hit.deflected, true);
		assert_eq!(hit.position.x, 0.0);
		assert_eq!(hit.position.y, 1.0);
		assert_eq!(hit.remainder.x, 1.0);
		assert_eq!(hit.remainder.y, 1.0);
	}

	#[test]
	fn gets_closest() {
		let maybe_result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::from_value(0.5),
				normal: Vec2::new(1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::zero(),
				source: Index::from_raw_parts(0, 0),
			},
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(0.0, 1.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::zero(),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		let result = maybe_result.unwrap();
		let hit = &result.deflections[0];
		assert_eq!(hit.times.min().unwrap(), 0.5);
		assert_eq!(hit.times.max().unwrap(), 0.5);
	}

	#[test]
	fn two_normals() {
		let maybe_result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::new(0.0, 1.0),
				remainder: Vec2::new(1.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new( 0.0,-1.0),
				deflected: true,
				position:  Vec2::new(0.0, 1.0),
				remainder: Vec2::new(1.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		let result = maybe_result.unwrap();
		let remainder = result.final_position - result.deflections[0].position;
		assert_eq!(remainder.x, 0.0);
		assert_eq!(remainder.y, 0.0);
	}

	#[test]
	fn two_same_normals() {
		let maybe_result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(0.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(0.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		let result = maybe_result.unwrap();
		let remainder = result.final_position - result.deflections[0].position;
		assert_eq!(remainder.x, 0.0);
		assert_eq!(remainder.y, 1.0);
	}

	#[test]
	fn opposing_normals() { // Normals on opposite sides of the remaining movement.
		let maybe_result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(0.0, -1.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		let result = maybe_result.unwrap();
		let remainder = result.final_position - result.deflections[0].position;
		assert_eq!(remainder.x, 0.0);
		assert_eq!(remainder.y, 0.0);
	}

	#[test]
	fn similar_normals() { // Normals on same sides of the remaining movement.
		let maybe_result = TotalDeflection::try_new(vec![
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(0.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
			Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 1.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(0.0, 1.0),
				source: Index::from_raw_parts(0, 0),
			},
		]);
		let result = maybe_result.unwrap();
		let remainder = result.final_position - result.deflections[0].position;
		assert_eq!(remainder.x, 0.0);
		assert_eq!(remainder.y, 1.0);
	}
}
