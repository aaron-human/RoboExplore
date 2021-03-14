use std::f32::{NAN, INFINITY};

use super::consts::*;

/// A continuous range over a 1D value.
#[derive(Debug, Clone)]
pub struct Range {
	min : f32, // The lower value.
	max : f32, // The upper value.
}

impl Range {
	/// Creates a new instance containing no values.
	pub fn empty() -> Range {
		Range { min: NAN, max: NAN }
	}

	/// Creates a new instance containing two values.
	pub fn from_values(val1 : f32, val2 : f32) -> Range {
		if val1 <= val2 {
			Range { min: val1, max: val2 }
		} else {
			Range { min: val2, max: val1 }
		}
	}

	/// Creates a new instance containing one value.
	pub fn from_value(value : f32) -> Range {
		Range { min: value, max: value }
	}

	/// Creates a new instance containing all values.
	pub fn all() -> Range {
		Range { min: -INFINITY, max: INFINITY }
	}

	/// Creates a range with end points at the zeros of a quadratic (or linear, or constant).
	pub fn from_quadratic_zeros(a : f32, b : f32, c : f32) -> Range {
		if a.abs() < EPSILON {
			// If a is basically zero, then this isn't quadratic.
			if b.abs() < EPSILON {
				// If b is also basically zero, then this is a "constant equation". Just check if c is always (pretty much) zero.
				if c.abs() < EPSILON { Range::all() } else { Range::empty() }
			} else {
				// Then it's a linear equation with one solution.
				Range::from_value(-c / b)
			}
		} else {
			// Definitely a quadratic. Do the usual formula.
			let denom = 2.0 * a;
			let mut det = b * b - 4.0 * a * c;
			if det < -EPSILON {
				Range::empty() // Negative determinite means no zeros.
			} else if det < EPSILON {
				Range::from_value(-b / denom)
			} else {
				det = det.sqrt();
				Range::from_values((-b + det) / denom, (-b - det) / denom)
			}
		}
	}

	/// Makes this range contain no values.
	pub fn make_empty(&mut self) -> &mut Self {
		self.min = NAN;
		self.max = NAN;
		self
	}

	/// Makes this range contain no values.
	pub fn make_all(&mut self) -> &mut Self {
		self.min = -INFINITY;
		self.max = INFINITY;
		self
	}

	/// If this range contains NO values.
	pub fn is_empty(&self) -> bool { self.min.is_nan() || self.max.is_nan() }
	/// If the range covers all values.
	pub fn is_all(&self) -> bool { -INFINITY == self.min && INFINITY == self.max }
	/// Gets the minimum boundary.
	pub fn min(&self) -> Option<f32> {
		if self.is_empty() {
			None
		} else {
			Some(self.min)
		}
	}
	/// Gets the maximum boundary.
	pub fn max(&self) -> Option<f32> {
		if self.is_empty() {
			None
		} else {
			Some(self.max)
		}
	}
	/// Gets the min and max at the same time.
	pub fn min_max(&self) -> Option<(f32, f32)> {
		if self.is_empty() {
			None
		} else {
			Some((self.min, self.max))
		}
	}

	/// Check if the range contains a given value.
	pub fn contains_exactly(&self, value : f32) -> bool {
		!self.is_empty() && self.min <= value && value <= self.max
	}

	/// Check if the range contains a given value with some epsilon.
	pub fn contains(&self, value : f32) -> bool {
		!self.is_empty() && ( (self.min <= value && value <= self.max) || (self.min - value).abs() < EPSILON || (self.max - value).abs() < EPSILON )
	}
}

#[cfg(test)]
mod tests_range {
	use super::*;

	/// Verify the various constructors.
	#[test]
	fn range_constructors() {
		let mut x = Range::empty();
		assert!(x.is_empty());
		assert!(x.min().is_none());
		assert!(x.max().is_none());

		x = Range::from_value(5.0);
		assert_eq!(x.min().unwrap(), 5.0);
		assert_eq!(x.max().unwrap(), 5.0);

		x = Range::from_values(-1.0, 1.0);
		assert_eq!(x.min().unwrap(),-1.0);
		assert_eq!(x.max().unwrap(), 1.0);

		x = Range::all();
		assert_eq!(x.min().unwrap(),-INFINITY);
		assert!(x.contains(9999.0));
		assert!(x.contains(-9999.0));
		assert_eq!(x.max().unwrap(), INFINITY);
	}

	/// Verify can solve equation that's "constant" with no solution.
	#[test]
	fn quad_zeros_consts() {
		assert!(Range::from_quadratic_zeros(0.0, 0.0, 1.0).is_empty(), "No solutions");
		assert!(Range::from_quadratic_zeros(0.0, 0.0, 0.0).is_all(), "Any solution");
	}

	/// Verify can solve equation that's just linear.
	#[test]
	fn quad_zeros_linear() {
		let x = Range::from_quadratic_zeros(0.0, 2.0, 4.0);
		assert_eq!(x.min().unwrap(), -2.0, "One solution (a)");
		assert_eq!(x.max().unwrap(), -2.0, "One solution (b)");
	}

	/// Verify can solve quadratics.
	#[test]
	fn quad_zeros_normal() {
		let mut x = Range::from_quadratic_zeros(1.0, 0.0, 1.0);
		assert!(x.is_empty(), "No solutions");
		x = Range::from_quadratic_zeros(1.0, 10.0, 25.0);
		assert_eq!(x.min().unwrap(), -5.0, "One solution (a)");
		assert_eq!(x.max().unwrap(), -5.0, "One solution (b)");
		x = Range::from_quadratic_zeros(1.0, 0.0, -25.0);
		assert_eq!(x.min().unwrap(),-5.0, "Two solutions (a)");
		assert_eq!(x.max().unwrap(), 5.0, "Two solutions (b)");
	}
}

/// For the cover() operator on Ranges.
pub trait RangeCover<OTHER> {
	type Output;

	/// Expands the existing range to include the other.
	fn cover(self, other : OTHER) -> Self::Output;
}

impl<'l> RangeCover<f32> for &'l Range {
	type Output = Range;

	fn cover(self, value : f32) -> Self::Output {
		if self.is_empty() {
			Range { min : value, max : value }
		} else if value < self.min {
			Range { min : value, max : self.max }
		} else if value > self.max {
			Range { min : self.min, max : value }
		} else {
			self.clone()
		}
	}
}

macro_rules! impl_range_cover_f32 {
	( $(& $l:lifetime mut)? $l_type:ident ) => {
		impl<$($l)?> RangeCover<f32> for $(& $l mut)? $l_type {
			type Output = Self;

			fn cover(mut self, value : f32) -> Self::Output {
				if self.is_empty() {
					self.min = value;
					self.max = value;
				} else if value < self.min {
					self.min = value;
				} else if value > self.max {
					self.max = value;
				}
				self
			}
		}
	}
}

impl_range_cover_f32!( Range );
impl_range_cover_f32!( &'l mut Range );

#[cfg(test)]
mod tests_cover_f32 {
	use super::*;

	/// Verify the expressions that can have "function overloading" like handling via traits.
	#[test]
	fn cover_expressions() {
		let x = Range::empty();
		let mut y = (&x).cover(1.0);
		assert!(x.is_empty());
		assert!(!y.is_empty());
		assert!(!y.contains(-1.0));
		assert!( y.contains( 1.0));
		assert!(!y.contains( 2.0));
		assert_eq!(y.min().unwrap(), 1.0);
		assert_eq!(y.max().unwrap(), 1.0);

		(&mut y).cover(2.0);
		assert!(!y.contains(-1.0));
		assert!( y.contains( 1.0));
		assert!( y.contains( 1.5));
		assert!( y.contains( 2.0));
		assert!(!y.contains( 3.0));

		y = x.cover(-1.0);
		assert!(!y.is_empty());
		assert!(!y.contains(-2.0));
		assert!( y.contains(-1.0));
		assert!(!y.contains( 0.0));
		assert_eq!(y.min().unwrap(),-1.0);
		assert_eq!(y.max().unwrap(),-1.0);
	}
}

macro_rules! impl_range_cover_range {
	( $(& $l:lifetime mut)? $l_type:ident ; $(& $r:lifetime)? $r_type:ident ) => {
		impl<$($l ,)? $($r)?> RangeCover<$(& $r)? $r_type> for $(& $l mut)? $l_type {
			type Output = $(& $l mut)? $l_type;

			fn cover(mut self, right : $(& $r)? $r_type) -> Self::Output {
				if self.is_empty() {
					self.min = right.min;
					self.max = right.max;
				} else if !right.is_empty() {
					self.min = if self.min < right.min { self.min } else { right.min };
					self.max = if self.max > right.max { self.max } else { right.max };
				}
				self
			}
		}
	}
}

impl_range_cover_range!(         Range ;     Range );
impl_range_cover_range!(         Range ; &'r Range );
impl_range_cover_range!( &'l mut Range ;     Range );
impl_range_cover_range!( &'l mut Range ; &'r Range );

macro_rules! impl_range_cover_range_ref {
	( $(& $r:lifetime)? $r_type:ident ) => {
		impl<'l $(, $r)?> RangeCover<$(& $r)? $r_type> for &'l Range {
			type Output = Range;

			fn cover(self, right : $(& $r)? $r_type) -> Self::Output {
				if self.is_empty() {
					right.clone()
				} else if right.is_empty() {
					self.clone()
				} else {
					Range {
						min : if self.min < right.min { self.min } else { right.min },
						max : if self.max > right.max { self.max } else { right.max },
					}
				}
			}
		}
	}
}

impl_range_cover_range_ref!( Range );
impl_range_cover_range_ref!( &'r Range );

#[cfg(test)]
mod tests_range_cover_range {
	use super::*;

	/// Verify the expressions that can have "function overloading" like handling via traits.
	#[test]
	fn cover_expressions() {
		let mut x = Range::empty();
		{
			let y = Range::all();
			(&x).cover(&y);
			(&x).cover(y);
		}
		{
			let y = Range::all();
			(&mut x).cover(&y);
			(&mut x).cover(y);
		}
		{
			let a = Range::empty();
			let b = Range::all();
			a.cover(b);
		}
		{
			let a = Range::empty();
			let b = Range::all();
			a.cover(&b);
		}
	}

	/// Verify covering ranges with ranges actually works.
	#[test]
	fn cover_values() {
		let mut x = Range::empty();
		(&mut x).cover(Range::empty());
		assert!(x.is_empty());
		(&mut x).cover(Range::from_value(5.0));
		assert_eq!(x.min().unwrap(), 5.0);
		assert_eq!(x.max().unwrap(), 5.0);
		(&mut x).cover(Range::empty());
		assert_eq!(x.min().unwrap(), 5.0);
		assert_eq!(x.max().unwrap(), 5.0);
		(&mut x).cover(Range::from_value(-5.0));
		assert_eq!(x.min().unwrap(),-5.0);
		assert_eq!(x.max().unwrap(), 5.0);

		x = Range::empty();
		let mut y = (&x).cover(Range::empty());
		assert!(y.is_empty());
		y = (&x).cover(Range::from_values(1.0, -1.0));
		assert_eq!(y.min().unwrap(),-1.0);
		assert_eq!(y.max().unwrap(), 1.0);
		y = y.cover(Range::from_values(-2.0, 0.5));
		assert_eq!(y.min().unwrap(),-2.0);
		assert_eq!(y.max().unwrap(), 1.0);
	}
}

/// For the intersect() operator on Ranges.
pub trait RangeIntersect<OTHER> {
	type Output;

	/// Finds the common values between this Range and another.
	fn intersect(self, right : OTHER) -> Self::Output;
}

macro_rules! impl_range_intersect_range {
	( $(& $l:lifetime mut)? $l_type:ident ; $(& $r:lifetime)? $r_type:ident ) => {
		impl<$($l ,)? $($r)?> RangeIntersect<$(& $r)? $r_type> for $(& $l mut)? $l_type {
			type Output = $(& $l mut)? $l_type;

			fn intersect(mut self, right : $(& $r)? $r_type) -> Self::Output {
				if self.is_empty() || right.is_empty() {
					self.make_empty();
				} else {
					if self.min < right.min {
						self.min = right.min;
					}
					if self.max > right.max {
						self.max = right.max;
					}
					if self.max < self.min {
						self.make_empty();
					}
				}
				self
			}
		}
	}
}

impl_range_intersect_range!(         Range ;     Range );
impl_range_intersect_range!(         Range ; &'r Range );
impl_range_intersect_range!( &'l mut Range ;     Range );
impl_range_intersect_range!( &'l mut Range ; &'r Range );

macro_rules! impl_range_intersect_range_ref {
	( $(& $r:lifetime)? $r_type:ident ) => {
		impl<'l, $($r)?> RangeIntersect<$(& $r)? $r_type> for &'l Range {
			type Output = Range;

			fn intersect(self, right : $(& $r)? $r_type) -> Self::Output {
				if self.is_empty() || right.is_empty() {
					Range::empty()
				} else {
					let min = if self.min > right.min { self.min } else { right.min };
					let max = if self.max < right.max { self.max } else { right.max };
					if max < min {
						Range::empty()
					} else {
						Range { min, max }
					}
				}
			}
		}
	}
}

impl_range_intersect_range_ref!( Range );
impl_range_intersect_range_ref!( &'r Range );

#[cfg(test)]
mod tests_range_intersect_range {
	use super::*;

	/// Verify the expressions that can have "function overloading" like handling via traits.
	#[test]
	fn intersect_expressions() {
		let mut x = Range::all();
		{
			let y = Range::all();
			(&x).intersect(&y);
			(&x).intersect(y);
		}
		{
			let y = Range::all();
			(&mut x).intersect(&y);
			(&mut x).intersect(y);
		}
		{
			let a = Range::all();
			let b = Range::all();
			a.intersect(b);
		}
		{
			let a = Range::all();
			let b = Range::all();
			a.intersect(&b);
		}
	}

	/// Verify covering ranges with ranges actually works.
	#[test]
	fn intersect_values() {
		let mut x = Range::empty();
		let mut y = Range::from_value(1.0);
		(&mut x).intersect(&y);
		assert!(x.is_empty());

		(&mut y).intersect(&x);
		assert!(y.is_empty());

		x = Range::from_values( 1.0, 2.0);
		y = Range::from_values(-1.0,-2.0);
		(&mut x).intersect(&y);
		assert!(x.is_empty());

		x = Range::from_values(-1.0, 2.0);
		y = Range::from_values( 1.0,-2.0);
		(&mut y).intersect(&x);
		assert_eq!(y.min().unwrap(),-1.0);
		assert_eq!(y.max().unwrap(), 1.0);


		x = Range::empty();
		let mut y = (&x).intersect(Range::from_value(-1.0));
		assert!(y.is_empty());

		y = Range::from_value(-1.0);
		y = (&y).intersect(&x);
		assert!(y.is_empty());

		x = Range::from_values(-1.0, 1.0);
		y = (&x).intersect(Range::from_value(0.0));
		assert_eq!(y.min().unwrap(), 0.0);
		assert_eq!(y.max().unwrap(), 0.0);

		y = (&y).intersect(Range::from_values(-2.0, -3.0));
		println!("{:?}", y);
		assert!(y.is_empty());
	}
}
