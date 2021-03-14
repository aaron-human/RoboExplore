
use super::consts::*;
use super::vec2::*;

/// A 2D bounding box.
#[derive(Debug)]
pub struct Bounds2 {
	x_min : f32,
	x_max : f32,
	y_min : f32,
	y_max : f32,
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
