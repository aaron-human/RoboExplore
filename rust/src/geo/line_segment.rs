use super::consts::*;
use super::common::*;
use super::vec2::*;
use super::range::*;
use super::bounds2::*;

/// A line segment.
#[derive(Debug, Clone)]
pub struct LineSegment {
	pub start : Vec2, // One of the end points.
	pub end : Vec2,   // The other end point.
	pub length : f32,
	pub direction : Vec2, // The direction from start to end. Always unit length, unless the line is just a point (then it's zero length as that makes intersection checking easier).
} // TODO: Make the above not pub... (Don't want a user would modifing them!)

/// All possible results of two line segments intersecting.
#[derive(Debug)]
pub enum LineSegmentIntersection {
	Point(Vec2), // A single point.
	Many(LineSegment), // A range of points because the line segments share an entire line-segment worth of points.
	None, // No intersection.
}

impl LineSegmentIntersection {
	/// A simple way to check for "no intersection". Mainly for the automated tests.
	pub fn is_none(&self) -> bool {
		match self {
			LineSegmentIntersection::None => true,
			_ => false,
		}
	}
}

impl LineSegment {
	/// Creates a line segment with the given end points.
	pub fn new(start : &Vec2, end : &Vec2) -> LineSegment {
		let delta = end - start;
		let mut length = delta.length();
		if length < EPSILON { length = 0.0; }
		LineSegment {
			start: start.clone(),
			end: end.clone(),
			length,
			direction: if 0.0 == length { Vec2::zero() } else { delta.norm() },
		}
	}

	/// Check if two lines overlap. Tries to be efficient and doesn't find where they overlap.
	pub fn check_if_intersects_with_line_segment(&self, other : &LineSegment) -> bool {
		// If the bounding boxes don't even overlap, then they definitely don't intersect.
		if !Bounds2::from_points(&self.start, &self.end).overlaps(&Bounds2::from_points(&other.start, &other.end)) { return false; }
		// If each pair of end points is on either side of the opposite line, then they intersect.
		let other_start_to_self_start = &self.start - &other.start;
		let other_start_to_self_end   = &self.end   - &other.start;
		let self_start_to_other_start = &other.start - &self.start;
		let self_start_to_other_end   = &other.end   - &self.start;
		let self_start_side   = sign(other.direction.ext(&other_start_to_self_start));
		let self_end_side     = sign(other.direction.ext(&other_start_to_self_end));
		let other_start_side  = sign(self.direction.ext( &self_start_to_other_start));
		let other_end_side    = sign(self.direction.ext( &self_start_to_other_end));
		// Note having one zero should be fine: it just means one of the end points is on the other line.
		if self_start_side != self_end_side && other_start_side != other_end_side { return true; }
		// One last way could be intersecting: if both lines are colinear. At that point all "sides" would be 0.
		if 0.0 == self_start_side && 0.0 == self_end_side && 0.0 == other_start_side && 0.0 == other_end_side {
			// At this point, use dot product to see if any of the start/end points are between the other line's.
			let mut along;
			along = other.direction.dot(&other_start_to_self_start);
			if -EPSILON < along && along - other.length < EPSILON { return true; }
			along = other.direction.dot(&other_start_to_self_end);
			if -EPSILON < along && along - other.length < EPSILON { return true; }
			along = self.direction.dot( &self_start_to_other_start);
			if -EPSILON < along && along - self.length < EPSILON { return true; }
			along = self.direction.dot( &self_start_to_other_end);
			if -EPSILON < along && along - self.length < EPSILON { return true; }
		}
		false // If all else fails, then they're not intersecting.
	}

	/// Gets the shortest distance to a point from somewhere on this line segment.
	pub fn shortest_distance_to_point(&self, point : &Vec2) -> f32 {
		let offset = point - &self.start;
		let along = self.direction.dot(&offset);
		println!("along: {:?} vs {:?}", along, self.length);
		if -EPSILON < along && along - self.length < EPSILON {
			self.direction.ext(&offset).abs()
		} else {
			// Must be one of the end points.
			(if 0.0 > along { offset } else { point - &self.end }).length()
		}
	}

	/// Find the intersection between two line segments (if one exists).
	pub fn find_intersection_with_line_segment(&self, other : &LineSegment) -> LineSegmentIntersection {
		// If the bounding boxes don't even overlap, then they definitely don't intersect.
		if !Bounds2::from_points(&self.start, &self.end).overlaps(&Bounds2::from_points(&other.start, &other.end)) { return LineSegmentIntersection::None; }
		// To find the probable point of intersection get the perpendicular distance from this line segment to the other's starting point.
		// Then convert the other line segment's "direction" into a value that decides how quickly it moves toward/away from this line segment when tranveling from its start to end.
		// Use that to figure out where the segments would have to intersect.
		let start_offset = &other.start - &self.start;
		let start_perp_dist = self.direction.ext(&start_offset);
		let perp_direction = self.direction.ext(&other.direction);
		// If the lines are parallel, things degenerate quickly.
		if perp_direction.abs() < EPSILON {
			// If they're not fully colinear lines, then no intersection.
			if EPSILON < start_perp_dist.abs() {
				return LineSegmentIntersection::None;
			}
			// Otherwise the lines are on the same infinite line, and must overlap because their bounding boxes do.
			// Get the signed direction to all start/end points using this.start using this.direction.
			let self_range = Range::from_values(0.0, self.length); // self.start is obviously at 0.0, self.end is at self.length since self.direction is unit length.
			let other_range = Range::from_values(
				self.direction.dot(start_offset),
				self.direction.dot(&other.end - &self.start),
			);
			let overlap = self_range.intersect(other_range);
			let hit_start = self.direction.scale(overlap.min().unwrap()) + &self.start;
			let hit_end   = self.direction.scale(overlap.max().unwrap()) + &self.start;
			return if (&hit_end - &hit_start).length() < EPSILON {
				LineSegmentIntersection::Point(hit_start)
			} else {
				LineSegmentIntersection::Many(LineSegment::new(&hit_start, &hit_end))
			}
		}
		// Otherwise, they're not parallel, and there's one (possible) point of intersection where: 0 = start_perp_dist + perp_direction * t
		let t = -start_perp_dist / perp_direction;
		// If the time is negative, then it's before the other line segment's start, so no intersection.
		if t < -EPSILON {
			return LineSegmentIntersection::None;
		}
		let possible = other.direction.scale(t) + &other.start;
		// Last check to see if the point is between the start and end of both line segments.
		let self_along  = self.direction.dot( &possible - &self.start);
		let other_along = other.direction.dot(&possible - &other.start);
		if -EPSILON > self_along || EPSILON < self_along - self.length || -EPSILON > other_along || EPSILON < other_along - other.length {
			return LineSegmentIntersection::None; // Past one of the end points.
		}
		// At this point, it's definitely a valid intersection.
		// Do a little snapping (to end points) if it goes past them.
		if 0.0 <= self_along - self.length {
			LineSegmentIntersection::Point(self.end.clone())
		} else if 0.0 >= self_along {
			LineSegmentIntersection::Point(self.start.clone())
		} else if 0.0 <= other_along - other.length {
			LineSegmentIntersection::Point(other.end.clone())
		} else if 0.0 >= other_along {
			LineSegmentIntersection::Point(other.start.clone())
		} else {
			LineSegmentIntersection::Point(possible)
		}
	}

	/// Gets the end point that doesn't match the one passed in.
	pub fn get_other_end_point<'a>(&'a self, check : &Vec2) -> &'a Vec2 {
		if (self.start - check).length() < EPSILON {
			&self.end
		} else {
			&self.start
		}
	}
}

#[cfg(test)]
mod test_intersection {
	use super::*;
	use crate::{assert_about_eq, assert_vec2_about_eq};

	#[test]
	fn overlaps() {
		// Bounding boxes don't overlap.
		assert!(!LineSegment::new(
			&Vec2::new(0.0, 1.0),
			&Vec2::new(1.0, 0.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(2.0, 3.0),
			&Vec2::new(3.0, 2.0),
		)));

		// Normal perpendicular where points are clearly on opposite sides of other line.
		assert!(LineSegment::new(
			&Vec2::new( 0.0, 1.0),
			&Vec2::new( 1.0, 0.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new( 2.0, 2.0),
			&Vec2::new(-2.0,-2.0),
		)));

		// T-shape, where one point is on the other line. Swap around for a 4 possible point orderings.
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 5.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(0.0, 3.0),
			&Vec2::new(2.0, 3.0),
		)));
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 5.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(2.0, 3.0),
			&Vec2::new(0.0, 3.0),
		)));
		assert!(LineSegment::new(
			&Vec2::new(0.0, 3.0),
			&Vec2::new(2.0, 3.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 5.0),
		)));
		assert!(LineSegment::new(
			&Vec2::new(2.0, 3.0),
			&Vec2::new(0.0, 3.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 5.0),
		)));

		// Then try colinear with all 4 possible combinations.
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 3.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(0.0, 0.0),
			&Vec2::new(2.0, 2.0),
		)));
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 3.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(2.0, 2.0),
			&Vec2::new(0.0, 0.0),
		)));
		assert!(LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(0.0, 0.0),
			&Vec2::new(2.0, 2.0),
		)));
		assert!(LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(2.0, 2.0),
			&Vec2::new(0.0, 0.0),
		)));

		// Try a degenerate case where one line is a point. Have it: on the line, beyond the end point of the line, and way off the line.
		assert!(LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(2.0, 2.0),
			&Vec2::new(2.0, 2.0),
		)));
		assert!(!LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(4.0, 4.0),
			&Vec2::new(4.0, 4.0),
		)));
		assert!(!LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(6.0, 3.0),
			&Vec2::new(6.0, 3.0),
		)));

		// Check that the "on same side" checking is right.
		assert!(!LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(10.0, 10.0),
			).check_if_intersects_with_line_segment(&LineSegment::new(
			&Vec2::new(5.5, 4.5),
			&Vec2::new(6.0, 4.0),
		)));
	}

	#[test]
	fn check_shortest_distance() {
		let line = LineSegment::new(&Vec2::new(1.0, 1.0), &Vec2::new(3.0, 3.0));
		assert_about_eq!(2.0_f32.sqrt(), line.shortest_distance_to_point(&Vec2::new(3.0, 1.0)));
		assert_about_eq!(1.0, line.shortest_distance_to_point(&Vec2::new(4.0, 3.0)));
		assert_about_eq!(1.0, line.shortest_distance_to_point(&Vec2::new(1.0, 0.0)));
		assert_about_eq!(2.0_f32.sqrt(), line.shortest_distance_to_point(&Vec2::new(0.0, 0.0)));
	}

	/// Asserts that a LineSegmentIntersection is a Point() type, and that the point passes a assert_vec2_about_eq!().
	macro_rules! assert_intersection_is_point {
		( $result:expr , $point:expr ) => {
			let result = $result;
			if let LineSegmentIntersection::Point(hit) = result {
				assert_vec2_about_eq!(hit, $point);
			} else {
				panic!("Expected single point intersection but got {:?}", result);
			}
		};
	}

	/// Simple white-box testing for the find_intersection_with_line_segment() function.
	#[test]
	fn check_find_intersection_with_line_segment() {
		// Check when the boudning boxes don't even overlap.
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(0.0, 0.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(1.0,-1.0),
			&Vec2::new(5.0,-5.0),
			)).is_none()
		);

		// Check parallel but not on same "infinite line".
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(5.0, 5.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(1.0, 2.0),
			&Vec2::new(5.0, 6.0),
			)).is_none()
		);

		// Check on same "infinite line" and hits at a point vs over a range of points.
		assert_intersection_is_point!(LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(6.0, 6.0),
			)),
			Vec2::new(5.0, 5.0)
		);

		let result = LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(4.0, 4.0),
			&Vec2::new(6.0, 6.0),
			)
		);
		if let LineSegmentIntersection::Many(segment) = result {
			// Note: the order really doesn't matter here.
			assert_vec2_about_eq!(segment.start, Vec2::new(5.0, 5.0));
			assert_vec2_about_eq!(segment.end,   Vec2::new(4.0, 4.0));
		} else {
			panic!("Expected single multipoint intersection but got {:?}", result);
		}

		// Check could intersect except past one of the line segments' start or end points.
		// Shuffle the values around to check different start/end point combinations.
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(5.0, 5.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(2.5, 1.5),
			&Vec2::new(5.0, 0.0),
			)).is_none()
		);
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(5.0, 5.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(5.0, 0.0),
			&Vec2::new(2.5, 1.5),
			)).is_none()
		);
		assert!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(5.0, 5.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(2.5, 1.5),
			&Vec2::new(5.0, 0.0),
			)).is_none()
		);
		assert!(LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(5.0, 0.0),
			&Vec2::new(2.5, 1.5),
			)).is_none()
		);

		// Check simple intersection in the middle of both segments.
		assert_intersection_is_point!(LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(5.0, 0.0),
			&Vec2::new(0.0, 5.0),
			)),
			Vec2::new(2.5, 2.5)
		);

		// Check intersection at the end points (all 4).
		assert_intersection_is_point!(LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(5.0, 5.0),
			)),
			Vec2::new(3.0, 3.0)
		);
		assert_intersection_is_point!(LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(3.0, 3.0),
			)),
			Vec2::new(3.0, 3.0)
		);
		assert_intersection_is_point!(LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(1.0, 1.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(3.0, 3.0),
			&Vec2::new(5.0, 5.0),
			)),
			Vec2::new(3.0, 3.0)
		);
		assert_intersection_is_point!(LineSegment::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(3.0, 3.0),
			).find_intersection_with_line_segment(&LineSegment::new(
			&Vec2::new(5.0, 5.0),
			&Vec2::new(3.0, 3.0),
			)),
			Vec2::new(3.0, 3.0)
		);

		// Could also check rounding behavior, but that's mostly just to limit rounding error propegation... Eh, not too important.
	}
}
