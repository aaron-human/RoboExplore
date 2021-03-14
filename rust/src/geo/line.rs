use super::vec2::*;

/// An infinite 2D line.
/// The line is defined as: delta.x * x + delta.y * y = c
#[derive(Debug)]
pub struct Line {
	pub delta : Vec2, // The coefficients. The vector is unit-length.
	pub c : f32, // The constant offset.
	pub origin : Vec2, // A point on the line.
}

impl Line {
	/// Creates a line given two points on it.
	pub fn new(p1 : &Vec2, p2 : &Vec2) -> Line {
		let delta = (p2 - p1).norm();
		Line {
			c: (&delta).dot(p1),
			delta,
			origin: p1.clone(),
		}
	}

	/// Gets the distance from the closest point on the line to a given point.
	pub fn ortho_distance_to(&self, point : &Vec2) -> f32 {
		self.delta.ext(point - &self.origin).abs()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn ortho_distance() {
		let line = Line::new(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(2.0, 1.0),
		);
		assert_eq!(line.ortho_distance_to(&Vec2::new(-1.0, -1.0)), 2.0);
		assert_eq!(line.ortho_distance_to(&Vec2::new(-1.0, 1.0)), 0.0);
	}
}
