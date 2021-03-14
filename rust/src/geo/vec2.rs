use auto_ops::{impl_op, impl_op_ex};

use super::consts::*;

/// A 2D vector.
#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
	pub x : f32, // The x component.
	pub y : f32, // The y component.
}

impl Vec2 {
	/// Creates a new instance.
	pub fn new(x : f32, y : f32) -> Vec2 {
		Vec2 { x, y }
	}

	/// Creates a new zero vector.
	pub fn zero() -> Vec2 {
		Vec2 { x: 0.0, y: 0.0 }
	}

	/// The vector's length.
	pub fn length(&self) -> f32 {
		(self.x * self.x + self.y * self.y).sqrt()
	}
}

impl_op_ex!(+ |left: &Vec2, right: &Vec2| -> Vec2 { Vec2{ x: left.x + right.x, y: left.y + right.y } } );
impl_op_ex!(- |left: &Vec2, right: &Vec2| -> Vec2 { Vec2{ x: left.x - right.x, y: left.y - right.y } } );
impl_op!(+= |left: &mut Vec2, right: Vec2| { left.x += right.x; left.y += right.y; } );
impl_op!(+= |left: &mut Vec2, right: &Vec2| { left.x += right.x; left.y += right.y; } );
impl_op_ex!(-= |left: &mut Vec2, right: Vec2| { left.x -= right.x; left.y -= right.y; } );

/// For unary operators on a Vec2.
pub trait VecOp {
	type Output;

	/// Normalizes the vector.
	fn norm(self) -> Self::Output;

	/// Creates an orthogonal vector, or makes the current one orthogonal to what it's currently set to.
	fn ortho(self) -> Self::Output;
}

impl<'l> VecOp for &'l Vec2 {
	type Output = Vec2;

	fn norm(self) -> Self::Output {
		let length = self.length();
		Vec2 { x: self.x / length, y: self.y / length }
	}

	fn ortho(self) -> Self::Output {
		Vec2 { x: -self.y, y: self.x }
	}
}

macro_rules! impl_vec_op {
	( $(& $l:lifetime mut)? $l_type:ident ) => {
		impl<$($l)?> VecOp for $(& $l mut)? $l_type {
			type Output = Self;

			fn norm(mut self) -> Self::Output {
				let length = self.length();
				self.x /= length;
				self.y /= length;
				self
			}

			fn ortho(mut self) -> Self::Output {
				let x = self.x;
				self.x = -self.y;
				self.y = x;
				self
			}
		}
	}
}

impl_vec_op!( Vec2 );
impl_vec_op!( &'l mut Vec2 );

#[cfg(test)]
mod tests_vec_op {
	use super::*;

	/// Verify the expressions that can have "function overloading" like handling via traits.
	#[test]
	fn norm_expressions() {
		let mut x = Vec2::new(2.0, 0.0);
		let mut result : Vec2;
		result = (&x).norm();
		assert_eq!(result.x, 1.0);
		assert_eq!(result.y, 0.0);
		assert_eq!(x.x, 2.0);
		assert_eq!(x.y, 0.0);
		(&mut x).norm();
		assert_eq!(x.x, 1.0);
		assert_eq!(x.y, 0.0);

		x.x = 0.0;
		x.y = 3.0;
		result = x.norm();
		assert_eq!(result.x, 0.0);
		assert_eq!(result.y, 1.0);
	}
}

/// For operators between a Vec2 and a scalar.
pub trait VecOpScalar<SCALAR> {
	type Output;

	/// Scales the vector by a given factor.
	fn scale(self, factor : SCALAR) -> Self::Output;

	/// Sets the length of the vector to a given value.
	fn set_length(self, length : SCALAR) -> Self::Output;
}

impl<'l> VecOpScalar<f32> for &'l Vec2 {
	type Output = Vec2;

	fn scale(self, factor : f32) -> Self::Output {
		Vec2 { x: self.x * factor, y : self.y * factor }
	}

	fn set_length(self, length : f32) -> Self::Output {
		let factor = length / self.length();
		self.scale(factor)
	}
}

macro_rules! impl_vec_op_scalar {
	( $(& $l:lifetime mut)? $l_type:ident ) => {
		impl<$($l)?> VecOpScalar<f32> for $(& $l mut)? $l_type {
			type Output = Self;

			fn scale(mut self, factor : f32) -> Self::Output {
				self.x *= factor;
				self.y *= factor;
				self
			}

			fn set_length(self, length : f32) -> Self::Output {
				let factor = length / self.length();
				self.scale(factor)
			}
		}
	}
}

impl_vec_op_scalar!( Vec2 );
impl_vec_op_scalar!( &'l mut Vec2 );

#[cfg(test)]
mod tests_vec_op_scalar {
	use super::*;

	/// Verify the expressions that can have "function overloading" like handling via traits.
	#[test]
	fn scale_expressions() {
		let mut x = Vec2::new(1.0, 1.0);
		let mut result : Vec2;
		result = (&x).scale(1.0).scale(0.0);
		assert_eq!(result.x, 0.0);
		assert_eq!(result.y, 0.0);
		assert_eq!(x.x, 1.0);
		assert_eq!(x.y, 1.0);
		(&mut x).scale(1.0).scale(0.0);
		assert_eq!(x.x, 0.0);
		assert_eq!(x.y, 0.0);

		x.x = 1.0;
		result = x.scale(-1.0);
		assert_eq!(result.x, -1.0);
		assert_eq!(result.y, 0.0);
	}
}

/// For binary operators between a Vec2 and another Vec2 that yield a scalar.
pub trait VecOpVecToScalar<RIGHT> {
	/// Computes the dot product.
	fn dot(self, right : RIGHT) -> f32;

	/// Computes the exterior product.
	fn ext(self, right : RIGHT) -> f32;
}

macro_rules! impl_vec_op_vec {
	( $(& $r:lifetime)? $r_type:ident ) => {
		impl<'l, $( $r )?> VecOpVecToScalar<$(& $r )? $r_type> for &'l Vec2 {
			fn dot(self, right : $(& $r )? $r_type) -> f32 {
				self.x * right.x + self.y * right.y
			}

			fn ext(self, right : $(& $r )? $r_type) -> f32 {
				self.x * right.y - self.y * right.x
			}
		}
	};
}

// TODO: Setup an `&mut self` variant?
impl_vec_op_vec!( Vec2 );
impl_vec_op_vec!( &'r Vec2 );

#[cfg(test)]
mod test_vec_op_vec {
	use super::*;

	/// Verify the expressions allowed with the dot product. Should also be able to swap in the ext() method, though didn't care to try.
	#[test]
	fn dot_expressions() {
		let x = Vec2::zero();
		let y = Vec2::zero();
		let mut _result : f32;
		_result = (&x).dot(&y);
		{
			let z = Vec2::zero();
			_result = z.dot(&y); // The left most is implicitly made an immutable reference. So 'z' not consumed.
			z.length();
		}
		{
			let z = Vec2::zero();
			_result = (&x).dot(z);
		}
		_result = (x).dot(y);
	}
}

/// For binary operators between a Vec2 and another Vec2.
pub trait VecOpVecToVec<RIGHT> {
	type Output;

	/// Gets an orthogonal vector that's within 90 degrees of the given right vector.
	fn ortho_like(self, right : RIGHT) -> Self::Output;
}

macro_rules! impl_vec_op_vec_to_vec_ref {
	( $(& $r:lifetime)? $r_type:ident ) => {
		impl<'l, $($r)?> VecOpVecToVec<$(& $r)? $r_type> for &'l Vec2 {
			type Output = Vec2;

			fn ortho_like(self, right : $(& $r)? $r_type) -> Self::Output {
				let mut ortho = self.ortho();
				if 0.0 > (&ortho).dot(right) {
					(&mut ortho).scale(-1.0);
				}
				ortho
			}
		}
	};
}

// TODO? Create an &mut variant? I'm not sure why that would ever be needed. Though might be useful for symmetry if add more operators?

impl_vec_op_vec_to_vec_ref!( Vec2 );
impl_vec_op_vec_to_vec_ref!( &'r Vec2 );

macro_rules! impl_vec_op_vec_to_vec {
	( $(& $r:lifetime)? $r_type:ident ) => {
		impl<'l, $($r)?> VecOpVecToVec<$(& $r)? $r_type> for Vec2 {
			type Output = Vec2;

			fn ortho_like(mut self, right : $(& $r)? $r_type) -> Self::Output {
				let temp = self.x;
				self.y = self.x;
				self.x = temp;
				if 0.0 > (&self).dot(right) {
					(&mut self).scale(-1.0);
				}
				self
			}
		}
	};
}

impl_vec_op_vec_to_vec!( Vec2 );
impl_vec_op_vec_to_vec!( &'r Vec2 );

/// Checks if any 3 points are colinear (to within EPSILON). All points being the same counts.
pub fn points_are_colinear(p1 : &Vec2, p2 : &Vec2, p3 : &Vec2) -> bool {
	(p2 - p1).ext(p3 - p1).abs() < EPSILON
}

#[cfg(test)]
mod test_colinear_check {
	use super::*;

	/// Verifies points_are_colinear() basically works.
	#[test]
	fn everything() {
		assert!(points_are_colinear(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(2.0, 2.0),
			&Vec2::new(3.0, 3.0),
		));
		assert!(points_are_colinear(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(1.0, 1.0),
			&Vec2::new(1.0, 1.0),
		));
		assert!(!points_are_colinear(
			&Vec2::new(1.0, 1.0),
			&Vec2::new(2.0, 2.0),
			&Vec2::new(4.0, 3.0),
		));
	}
}