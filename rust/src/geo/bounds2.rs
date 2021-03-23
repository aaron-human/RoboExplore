
use super::super::externals::log;
use super::consts::*;
use super::vec2::*;

/// A 2D bounding box.
#[derive(Debug, Clone)]
pub struct Bounds2 {
	x_min : f32,
	x_max : f32,
	y_min : f32,
	y_max : f32,
}

#[derive(PartialEq)]
enum RelativePosition {
	Above = 0,
	Between = 1,
	Below = 2,
}

impl Bounds2 {
	/// Creates an instance where a rectangle is centered on a point.
	pub fn from_centered_rect(center : &Vec2, mut width : f32, mut height : f32) -> Bounds2 {
		width = width.abs() / 2.0;
		height = height.abs() / 2.0;
		Bounds2 {
			x_min: center.x - width,
			x_max: center.x + width,
			y_min: center.y - height,
			y_max: center.y + height,
		}
	}

	/// Creates an instance that contains a pair of points.
	pub fn from_points(first : &Vec2, second : &Vec2) -> Bounds2 {
		let mut ret = Bounds2 {
			x_min: 0.0,
			x_max: 0.0,
			y_min: 0.0,
			y_max: 0.0,
		};
		if first.x < second.x {
			ret.x_min = first.x;
			ret.x_max = second.x;
		} else {
			ret.x_min = second.x;
			ret.x_max = first.x;
		}
		if first.y < second.y {
			ret.y_min = first.y;
			ret.y_max = second.y;
		} else {
			ret.y_min = second.y;
			ret.y_max = first.y;
		}
		ret
	}

	/// Translates this instance in place.
	pub fn translate(&mut self, amount : &Vec2) {
		self.x_min += amount.x;
		self.y_min += amount.y;
		self.x_max += amount.x;
		self.y_max += amount.y;
	}

	/// Expands to include a given x value.
	pub fn expand_to_x(&mut self, x : f32) {
		if x < self.x_min {
			self.x_min = x;
		}
		if x > self.x_max {
			self.x_max = x;
		}
	}
	/// Expands to include a given y value.
	pub fn expand_to_y(&mut self, y : f32) {
		if y < self.y_min {
			self.y_min = y;
		}
		if y > self.y_max {
			self.y_max = y;
		}
	}

	pub fn x_min(&self) -> f32 { self.x_min }
	pub fn x_max(&self) -> f32 { self.x_max }
	pub fn y_min(&self) -> f32 { self.y_min }
	pub fn y_max(&self) -> f32 { self.y_max }

	/// Checks if this range overlaps another. This IS NOT exact (so is to within EPSILON).
	pub fn overlaps(&self, other : &Bounds2) -> bool {
		EPSILON >= self.x_min - other.x_max &&
		EPSILON >= other.x_min - self.x_max &&
		EPSILON >= self.y_min - other.y_max &&
		EPSILON >= other.y_min - self.y_max
	}

	/// Checks if this overlaps a given point.
	pub fn overlaps_point(&self, other : &Vec2) -> bool {
		self.x_min <= other.x && other.x <= self.x_max && self.y_min <= other.y && other.y <= self.y_max
	}

	/// Finds the point on the line segment that intersects with this instance.
	/// When possible tries to find the point that's closest to the start.
	pub fn collide_with_line_segment(&self, start : &Vec2, end : &Vec2) -> Option<Vec2> {
		let x_relative = if start.x > self.x_max { RelativePosition::Above } else if start.x < self.x_min { RelativePosition::Below } else { RelativePosition::Between };
		let y_relative = if start.y > self.y_max { RelativePosition::Above } else if start.y < self.y_min { RelativePosition::Below } else { RelativePosition::Between };

		// If it starts inside, then that's the closest point.
		if RelativePosition::Between == x_relative && RelativePosition::Between == y_relative {
			return Some(start.clone());
		}
		// Otherwise, much check if intersects any of the outer boundaries of the rectangle.
		let delta = end - start;
		if RelativePosition::Above == x_relative && -EPSILON > delta.x && end.x <= self.x_max {
			// Being above the x means can only intersect with the x_max wall. Check that.
			let hit = start + delta.scale((self.x_max - start.x) / delta.x);
			if self.y_min <= hit.y && hit.y <= self.y_max {
				return Some(hit);
			}
		} else if RelativePosition::Below == x_relative && EPSILON < delta.x && end.x >= self.x_min {
			// Being above the x means can only intersect with the x_max wall. Check that.
			let hit = start + delta.scale((self.x_min - start.x) / delta.x);
			if self.y_min <= hit.y && hit.y <= self.y_max {
				return Some(hit);
			}
		}
		if RelativePosition::Above == y_relative && -EPSILON > delta.y && end.y <= self.y_max {
			// Being above the x means can only intersect with the x_max wall. Check that.
			let hit = start + delta.scale((self.y_max - start.y) / delta.y);
			if self.x_min <= hit.x && hit.x <= self.x_max {
				return Some(hit);
			}
		} else if RelativePosition::Below == y_relative && EPSILON < delta.y && end.y >= self.y_min {
			// Being above the x means can only intersect with the x_max wall. Check that.
			let hit = start + delta.scale((self.y_min - start.y) / delta.y);
			if self.x_min <= hit.x && hit.x <= self.x_max {
				return Some(hit);
			}
		}
		// If nothing happened, then there is not intersection.
		None
	}
}

#[cfg(test)]
mod tests_everything {
	use super::*;

	#[test]
	fn overlaps() {
		// Clearly not overlapping.
		assert!(
			!Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new( 1.0, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new( 2.0,-1.0),
				&Vec2::new( 3.0, 1.0),
			))
		);
		assert!(
			!Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new( 1.0, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new(-1.0, 2.0),
				&Vec2::new( 1.0, 3.0),
			))
		);
		// Just touching.
		assert!(
			Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new( 1.0, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new( 1.0,-1.0),
				&Vec2::new( 3.0, 1.0),
			))
		);
		assert!(
			Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new( 1.0, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new(-1.0, 1.0),
				&Vec2::new( 1.0, 3.0),
			))
		);
		assert!(
			Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new( 1.0, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new(-2.0,-2.0),
			))
		);
		// Fully touching.
		assert!(
			Bounds2::from_points(
				&Vec2::new(-1.0,-1.0),
				&Vec2::new( 1.0, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new( 0.0, 0.0),
				&Vec2::new( 3.0, 3.0),
			))
		);
		// Barely touching (less than epsilon apart).
		const SUB_EPSILON : f32 = EPSILON / 8.0;
		assert!(
			Bounds2::from_points(
				&Vec2::new(-SUB_EPSILON,-1.0),
				&Vec2::new(-SUB_EPSILON, 1.0),
			).overlaps(&Bounds2::from_points(
				&Vec2::new( SUB_EPSILON,-1.0),
				&Vec2::new( SUB_EPSILON, 1.0),
			))
		);
	}
}
