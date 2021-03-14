
use super::consts::*;
use super::vec2::*;
use super::line_segment::*;
use super::circle::*;

/// Checks if a point is inside the given polygon.
/// This uses the old even-odd collision counting rule.
/// Being on the border counts as being inside the polygon.
/// This should work on basically any type of polygon, though it follows the "even-odd rule" when it comes to defining self-intersecting polygons.
pub fn is_point_inside_polygon(point : &Vec2, polygon : &Vec<Vec2>) -> bool {
	assert!(2 < polygon.len());
	println!("=============> Start!");
	let mut inside = false;
	let count = polygon.len();
	for index in 0..count {
		let start = polygon[index];
		let end = polygon[if index+1 < count { index+1 } else { 0 }];
		println!("Line {:?} to {:?}", start, end);
		// Find the hit between the ray from point down -x, and the start-end line segment.
		let denom = end.y - start.y;
		if denom.abs() < EPSILON {
			// Ingore all basically horizontal lines.
			println!("Denom skip");
			continue;
		}
		let t = (point.y - start.y) / denom;
		println!("t = {}", t);
		if -EPSILON > t || EPSILON > 1.0 - t {
			// Ignore before the start and after (or at) the end.
			// Ignoring at end because that prevents the end points from being double-counted.
			println!("Skip for t");
			continue;
		}
		let hit_x = end.x * t + start.x * (1.0 - t);
		if hit_x > point.x {
			// Ignore if happend on the +x side of the point.
			continue;
		}
		if EPSILON > (hit_x - point.x).abs() {
			// If on a border, then that's always inside.
			return true;
		}
		if EPSILON > t.abs() {
			println!("Checking near start.");
			// If at start point, then only count this as a hit if end points it connects to are on opposite sides of the line.
			// But, there could be a bunch of horizontal lines before this one. Those should be ignored.
			let after_side = end.y > point.y;
			let mut ignore = false;
			for offset in 1..(count-1) {
				let mut prev_index = (index as i32) - (offset as i32);
				if prev_index < 0 { prev_index += count as i32; }
				println!("Searching for before @ {}.", prev_index);
				let prev = polygon[prev_index as usize];
				if EPSILON < (prev.y - point.y).abs() {
					let before_side = prev.y > point.y;
					ignore = before_side == after_side;
					println!("Found prev: {:?}.", prev);
					break;
				}
			}
			println!("Ignore = {:?}.", ignore);
			if ignore { continue; }
		}
		// If made it this far, then it's a hit that should flip whether inside.
		inside = !inside;
		println!("Flipped to {:?}.", inside);
	}
	inside
}

#[cfg(test)]
mod test_is_point_inside_polygon {
	use super::*;

	#[test]
	fn easy_outside() {
		assert_eq!( // Ray hits nothing.
			is_point_inside_polygon(
				&Vec2::new(10.0, 10.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(7.0, 3.0),
					Vec2::new(1.0, 1.0),
				],
			),
			false,
		);

		assert_eq!( // Ray hits middle of two lines.
			is_point_inside_polygon(
				&Vec2::new(5.0, 5.0),
				&vec![
					Vec2::new(1.0, 9.0),
					Vec2::new(3.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			false,
		);

		assert_eq!( // Ray hits middle of a line, and one of the corner points.
			is_point_inside_polygon(
				&Vec2::new(5.0, 5.0),
				&vec![
					Vec2::new(1.0, 9.0),
					Vec2::new(3.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			false,
		);

		assert_eq!( // Ray hits top corner point.
			is_point_inside_polygon(
				&Vec2::new(5.0, 9.0),
				&vec![
					Vec2::new(1.0, 9.0),
					Vec2::new(3.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			false,
		);

		assert_eq!( // Ray hits bottom corner point.
			is_point_inside_polygon(
				&Vec2::new(5.0, 1.0),
				&vec![
					Vec2::new(1.0, 9.0),
					Vec2::new(3.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			false,
		);

		assert_eq!( // Ray hits a parallel end side.
			is_point_inside_polygon(
				&Vec2::new(6.0, 5.0),
				&vec![
					Vec2::new(1.0, 1.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(5.0, 5.0),
					Vec2::new(5.0, 1.0),
				],
			),
			false,
		);

		assert_eq!( // Ray hits a parallel end side.
			is_point_inside_polygon(
				&Vec2::new(6.0, 1.0),
				&vec![
					Vec2::new(1.0, 1.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(5.0, 5.0),
					Vec2::new(5.0, 1.0),
				],
			),
			false,
		);
	}

	#[test]
	fn easy_inside() {
		assert_eq!( // Hits one line.
			is_point_inside_polygon(
				&Vec2::new(2.0, 4.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // Hits a corner point.
			is_point_inside_polygon(
				&Vec2::new(2.0, 5.0),
				&vec![
					Vec2::new(3.0, 9.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(3.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // Hits apex of a "v".
			is_point_inside_polygon(
				&Vec2::new(3.0, 2.0),
				&vec![
					Vec2::new(1.0, 1.0),
					Vec2::new(2.0, 2.0),
					Vec2::new(3.0, 1.0),
					Vec2::new(4.0, 1.0),
					Vec2::new(4.0, 3.0),
					Vec2::new(1.0, 3.0),
				],
			),
			true,
		);

		assert_eq!( // Hits a horizontal line.
			is_point_inside_polygon(
				&Vec2::new(3.0, 2.0),
				&vec![
					Vec2::new(1.0, 2.0),
					Vec2::new(2.0, 2.0),
					Vec2::new(3.0, 1.0),
					Vec2::new(4.0, 1.0),
					Vec2::new(4.0, 3.0),
					Vec2::new(1.0, 3.0),
				],
			),
			true,
		);

		assert_eq!( // Hits a horizontal line with a bajillion points in between.
			is_point_inside_polygon(
				&Vec2::new(3.0, 2.0),
				&vec![
					Vec2::new(1.0, 2.0),
					Vec2::new(1.5, 2.0),
					Vec2::new(1.6, 2.0),
					Vec2::new(1.7, 2.0),
					Vec2::new(1.8, 2.0),
					Vec2::new(1.9, 2.0),
					Vec2::new(2.0, 2.0),
					Vec2::new(3.0, 1.0),
					Vec2::new(4.0, 1.0),
					Vec2::new(4.0, 3.0),
					Vec2::new(1.0, 3.0),
				],
			),
			true,
		);
	}

	#[test]
	fn on_boundary() {
		assert_eq!( // On top horizontal line.
			is_point_inside_polygon(
				&Vec2::new(2.0, 5.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // On top right corner
			is_point_inside_polygon(
				&Vec2::new(5.0, 5.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // On right side
			is_point_inside_polygon(
				&Vec2::new(3.0, 3.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // On bottom point
			is_point_inside_polygon(
				&Vec2::new(1.0, 1.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // On left side
			is_point_inside_polygon(
				&Vec2::new(1.0, 3.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);

		assert_eq!( // On top-left corner
			is_point_inside_polygon(
				&Vec2::new(1.0, 5.0),
				&vec![
					Vec2::new(5.0, 5.0),
					Vec2::new(1.0, 5.0),
					Vec2::new(1.0, 1.0),
				],
			),
			true,
		);
	}
}

/// Converts a "polygon" into a vector of the lines that compose it.
/// Consecutive lines are connected. The start point for each entry is equivalent to the points with the same index in the passed in vector of Vec2 instances.
pub fn make_polygon_lines(polygon : &Vec<Vec2>) -> Vec<LineSegment> {
	let len = polygon.len();
	let mut lines  = Vec::with_capacity(len);
	for start_index in 0..len {
		let mut end_index = start_index + 1;
		if len <= end_index { end_index -= len; }
		lines.push(LineSegment::new(&polygon[start_index], &polygon[end_index]));
	}
	lines
}

/// Check if two polygons overlap. As one should expect, this uses some small EPSILON terms internally so round-off error doesn't break things.
/// Sharing a border or a single point counts as an overlap.
/// This should work on basically any type of polygon, though it follows the "even-odd rule" when it comes to defining self-intersecting polygons.
pub fn do_polygons_overlap(first : &Vec<Vec2>, second : &Vec<Vec2>) -> bool {
	assert!(2 < first.len());
	assert!(2 < second.len());
	// First create one LineSegment instance for every line in the polygons.
	let first_len  = first.len();
	let second_len = second.len();
	let first_lines  = make_polygon_lines(first);
	let second_lines = make_polygon_lines(second);
	// Then check if any of the line segments overlap each other.
	for first_index in 0..first_len {
		for second_index in 0..second_len {
			if first_lines[first_index].check_if_intersects_with_line_segment(&second_lines[second_index]) {
				return true;
			}
		}
	}
	// If none of the lines intersect, then there's one last way the polygons could overlap: one could be completely in the other.
	// Since the "contained" one could be either, have to check in both directions.
	is_point_inside_polygon(&first[0], second) || is_point_inside_polygon(&second[0], first)
}

#[cfg(test)]
mod test_do_polygons_overlap {
	use super::*;

	#[test]
	fn easy_apart() {
		assert!(!do_polygons_overlap(
			&vec![
				Vec2::new(1.0, 1.0),
				Vec2::new(2.0, 2.0),
				Vec2::new(1.0, 3.0),
			],
			&vec![
				Vec2::new(3.0,-1.0),
				Vec2::new(4.0, 1.0),
				Vec2::new(3.0, 3.0),
			],
		));
	}

	#[test]
	fn easy_overlap() {
		assert!(do_polygons_overlap(
			&vec![
				Vec2::new(1.0, 1.0),
				Vec2::new(3.0, 2.0),
				Vec2::new(1.0, 3.0),
			],
			&vec![
				Vec2::new(3.0, 1.0),
				Vec2::new(0.0, 2.0),
				Vec2::new(3.0, 3.0),
			],
		));
	}

	#[test]
	fn fully_inside() {
		assert!(do_polygons_overlap(
			&vec![
				Vec2::new(0.0, 0.0),
				Vec2::new(9.0, 0.0),
				Vec2::new(0.0, 9.0),
			],
			&vec![
				Vec2::new(1.0, 1.0),
				Vec2::new(2.0, 1.0),
				Vec2::new(1.0, 2.0),
			],
		));
		assert!(do_polygons_overlap(
			&vec![
				Vec2::new(1.0, 1.0),
				Vec2::new(2.0, 1.0),
				Vec2::new(1.0, 2.0),
			],
			&vec![
				Vec2::new(0.0, 0.0),
				Vec2::new(9.0, 0.0),
				Vec2::new(0.0, 9.0),
			],
		));
	}
}

/// Checks if a circle and a polygon share any points
pub fn does_circle_overlap_polygon(circle : &Circle, polygon : &Vec<Vec2>) -> bool {
	// If the circle's center is in the polygon, then it definitely overlaps.
	if is_point_inside_polygon(&circle.center, polygon) {
		return true;
	}
	// Otherwise see if any of the polygon's line are within radius of the circle.
	for segment in make_polygon_lines(&polygon) {
		if segment.shortest_distance_to_point(&circle.center) - circle.radius < EPSILON {
			return true;
		}
	}
	false
}

#[cfg(test)]
mod test_does_circle_overlap_polygon {
	use super::*;

	#[test]
	fn easy_checks() {
		assert!(!does_circle_overlap_polygon( // Apart
			&Circle::new(&Vec2::new(-2.0,-2.0), 0.1),
			&vec![
				Vec2::new(0.0, 0.0),
				Vec2::new(8.0, 0.0),
				Vec2::new(0.0, 8.0),
			],
		));
		assert!(does_circle_overlap_polygon( // Fully inside.
			&Circle::new(&Vec2::new(2.0, 2.0), 0.1),
			&vec![
				Vec2::new(0.0, 0.0),
				Vec2::new(8.0, 0.0),
				Vec2::new(0.0, 8.0),
			],
		));

		assert!(does_circle_overlap_polygon( // Just-barely on line.
			&Circle::new(&Vec2::new(0.0, 4.0), 1.0),
			&vec![
				Vec2::new(1.0, 1.0),
				Vec2::new(8.0, 8.0),
				Vec2::new(1.0, 8.0),
			],
		));

		assert!(does_circle_overlap_polygon( // Centered on line.
			&Circle::new(&Vec2::new(4.0, 4.0), 0.1),
			&vec![
				Vec2::new(1.0, 1.0),
				Vec2::new(8.0, 8.0),
				Vec2::new(1.0, 8.0),
			],
		));
	}
}
