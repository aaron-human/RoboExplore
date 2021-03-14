use std::f32::INFINITY;

use super::consts::*;
use super::range::*;
use super::vec2::*;

/// The result of deflecting a collider.
#[derive(Debug, Clone)]
pub struct Deflection {
	pub times : Range, // When in contact with the obstacle. Zero means at start. One means at end.
	pub normal : Vec2, // The surface normal. Should be pointing away from the surface. Must be unit length.
	pub deflected : bool, // Whether a deflection occurred. If false, then just passing back that was in contact with some surface.
	pub position : Vec2, // The position of the collider when the deflection happened.
	pub remainder : Vec2, // The remaining (deflected) movement.
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
			self.remainder += (&self.normal).scale(-coincidence);
		}
	}

	/// Combines multiple Deflections.
	/// Always yields the nearest. If there are multiple that fit that description, chooses first.
	/// If more than one unique normal applies at that time, then will try to apply the new ones. This will generally zero any movement toward two unique normals (not 100% sure if there's a better way).
	pub fn combine(mut items : Vec<Option<Deflection>>) -> Option<Deflection> {
		// First pass: find the Deflection with the earliest start time.
		let mut soonest_time : f32 = INFINITY;
		let mut soonest_index : usize = items.len();
		for index in 0..items.len() {
			if let Some(ref new) = items[index] {
				if !new.deflected {
					continue;
				} else {
					let mut new_time = new.times.min().unwrap();
					if 0.0 > new_time { new_time = 0.0; }
					if new_time < soonest_time {
						soonest_time = new_time;
						soonest_index = index;
					}
				}
			}
		}

		if soonest_index == items.len() {
			return None;
		}

		// Second pass: find all normals that apply to the soonest_time.
		let mut normals : Vec<Vec2> = Vec::new();
		for item in &items {
			if let Some(ref hit) = item {
				if hit.times.contains(soonest_time) {
					println!("Contained!");
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
			}
		}

		// Try limiting the remainder vector if there's more than one normal.
		// Do this mainly by checking if there are normals on the left and right of the remaining movement. That means drop the remaining movement.
		let mut on_pos : bool = false;
		let mut on_neg : bool = false;
		let mut deflection = items.remove(soonest_index).unwrap();
		for normal in &normals {
			// Ignore if the normal is in the direction of movement.
			// But don't ignore if it's perpendicular.
			if deflection.remainder.dot(normal) > EPSILON { continue; }
			// Check which side it is on.
			if 0.0 > deflection.remainder.ext(normal) {
				on_pos = true;
			} else {
				on_neg = true;
			}
			if on_pos && on_neg {
				(&mut deflection.remainder).scale(0.0);
				break;
			}
		}
		Some(deflection)
	}
}

#[cfg(test)]
mod test_combine_deflection {
	use super::*;

	#[test]
	fn no_hits() {
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::empty(),
				normal: Vec2::zero(),
				deflected: false,
				position:  Vec2::zero(),
				remainder: Vec2::zero(),
			}),
			None,
		]);
		assert!(result.is_none());
	}

	#[test]
	fn pass_through() {
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::from_value(0.5),
				normal: Vec2::new(1.0, 0.0),
				deflected: true,
				position:  Vec2::new(0.0, 1.0),
				remainder: Vec2::new(1.0, 1.0),
			}),
		]);
		let hit = result.unwrap();
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
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::from_value(0.5),
				normal: Vec2::new(1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::zero(),
			}),
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(0.0, 1.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::zero(),
			}),
		]);
		let hit = result.unwrap();
		assert_eq!(hit.times.min().unwrap(), 0.5);
		assert_eq!(hit.times.max().unwrap(), 0.5);
	}

	#[test]
	fn two_normals() {
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::new(0.0, 1.0),
				remainder: Vec2::new(1.0, 1.0),
			}),
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new( 0.0,-1.0),
				deflected: true,
				position:  Vec2::new(0.0, 1.0),
				remainder: Vec2::new(1.0, 1.0),
			}),
		]);
		let hit = result.unwrap();
		assert_eq!(hit.remainder.x, 0.0);
		assert_eq!(hit.remainder.y, 0.0);
	}

	#[test]
	fn two_same_normals() {
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
			}),
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
			}),
		]);
		let hit = result.unwrap();
		assert_eq!(hit.remainder.x, 1.0);
		assert_eq!(hit.remainder.y, 1.0);
	}

	#[test]
	fn opposing_normals() { // Normals on opposite sides of the remaining movement.
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
			}),
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(0.0, -1.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
			}),
		]);
		let hit = result.unwrap();
		assert_eq!(hit.remainder.x, 0.0);
		assert_eq!(hit.remainder.y, 0.0);
	}

	#[test]
	fn similar_normals() { // Normals on same sides of the remaining movement.
		let result = Deflection::combine(vec![
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 0.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
			}),
			Some(Deflection {
				times: Range::from_value(0.9),
				normal: Vec2::new(-1.0, 1.0),
				deflected: true,
				position:  Vec2::zero(),
				remainder: Vec2::new(1.0, 1.0),
			}),
		]);
		let hit = result.unwrap();
		assert_eq!(hit.remainder.x, 1.0);
		assert_eq!(hit.remainder.y, 1.0);
	}
}