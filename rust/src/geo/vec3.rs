use crate::externals::*;
use std::ops;
use auto_ops::{impl_op, impl_op_ex};

/// A 3D vector suitable for drawing.
#[derive(Clone, Debug)]
pub struct Vec3 {
	pub x : DrawCoord,
	pub y : DrawCoord,
	pub z : DrawCoord,
}

impl Vec3 {
	/// Creates a new instance with specific coordinate values.
	pub fn new(x : DrawCoord, y : DrawCoord, z : DrawCoord) -> Vec3 {
		Vec3 { x, y, z }
	}

	/// Creates a new instance with all zero values.
	pub fn zero() -> Vec3 {
		Vec3{ x: 0.0, y: 0.0, z: 0.0 }
	}

	/// Copies the value from another.
	pub fn copy_from(&mut self, other : &Vec3) {
		self.x = other.x;
		self.y = other.y;
		self.z = other.z;
	}

	/// Get the length of the vector.
	pub fn length(&self) -> DrawCoord {
		(self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
	}

	/// Tries to rescale the vector to a specific length.
	/// If the the current length() is too small, this could result in NaN/Infinity values.
	pub fn set_length(&mut self, new_length : DrawCoord) -> &mut Self {
		let rescale = new_length / self.length();
		self.x *= rescale;
		self.y *= rescale;
		self.z *= rescale;
		self
	}
}

impl ops::MulAssign<DrawCoord> for Vec3 {
	fn mul_assign(&mut self, right : DrawCoord) {
		self.x *= right;
		self.y *= right;
		self.z *= right;
	}
}

impl_op_ex!(+ |left: &Vec3, right: &Vec3| -> Vec3 { Vec3{ x: left.x + right.x, y: left.y + right.y, z: left.z + right.z } } );
impl_op_ex!(- |left: &Vec3, right: &Vec3| -> Vec3 { Vec3{ x: left.x - right.x, y: left.y - right.y, z: left.z - right.z } } );
impl_op_ex!(* |left: &Vec3, right: f32|   -> Vec3 { Vec3{ x: left.x * right,   y: left.y * right,   z: left.z * right } } );
impl_op!(+= |left: &mut Vec3, right:  Vec3| { left.x += right.x; left.y += right.y; } );
impl_op!(+= |left: &mut Vec3, right: &Vec3| { left.x += right.x; left.y += right.y; } );
impl_op_ex!(-= |left: &mut Vec3, right: Vec3| { left.x -= right.x; left.y -= right.y; } );
