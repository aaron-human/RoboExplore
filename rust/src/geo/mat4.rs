use crate::externals::*;
use super::vec3::*;

/// A 4x4 transform matrix.
#[derive(Clone)]
pub struct Mat4 {
	data : [DrawCoord; 16], // The matrix layed out NORMALLY (i.e. the transpose of what OpenGL does in memory).
}

impl Mat4 {
	/// Creates a new identity matrix.
	pub fn new() -> Mat4 {
		Mat4 {
			data : [
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			],
		}
	}

	/// Resets this matrix to identity.
	pub fn make_identity(&mut self) -> &mut Self {
		self.data = [
			1.0, 0.0, 0.0, 0.0,
			0.0, 1.0, 0.0, 0.0,
			0.0, 0.0, 1.0, 0.0,
			0.0, 0.0, 0.0, 1.0,
		];
		self
	}

	/// Resets this matrix to identity.
	pub fn translate_before(&mut self, translation : &Vec3) -> &mut Self {
		self.data[ 3] = self.data[0] * translation.x + self.data[1] * translation.y + self.data[ 2] * translation.z + self.data[ 3];
		self.data[ 7] = self.data[4] * translation.x + self.data[5] * translation.y + self.data[ 6] * translation.z + self.data[ 7];
		self.data[11] = self.data[8] * translation.x + self.data[9] * translation.y + self.data[10] * translation.z + self.data[11];
		self
	}

	/// Rotates the matrix about the z axis by some amount.
	/// Makes this rotation happen before the current transform stored in this matrix.
	pub fn rotz_before(&mut self, radians : f32) {
		let sin = radians.sin();
		let cos = radians.cos();
		let mut x;
		let mut y;
		x = self.data[0] * cos + self.data[1] * sin;
		y = self.data[0] *-sin + self.data[1] * cos;
		    self.data[0] = x;    self.data[1] = y;

		x = self.data[4] * cos + self.data[5] * sin;
		y = self.data[4] *-sin + self.data[5] * cos;
		    self.data[4] = x;    self.data[5] = y;

		x = self.data[8] * cos + self.data[9] * sin;
		y = self.data[8] *-sin + self.data[9] * cos;
		    self.data[8] = x;    self.data[9] = y;

		x = self.data[12] * cos + self.data[13] * sin;
		y = self.data[12] *-sin + self.data[13] * cos;
		    self.data[12] = x;    self.data[13] = y;
	}

	/// Rescales the matrix using the given vector's values.
	pub fn scale_before(&mut self, factor : &Vec3) {
		self.data[ 0] *= factor.x; self.data[ 1] *= factor.y; self.data[ 2] *= factor.z;
		self.data[ 4] *= factor.x; self.data[ 5] *= factor.y; self.data[ 6] *= factor.z;
		self.data[ 8] *= factor.x; self.data[ 9] *= factor.y; self.data[10] *= factor.z;
		self.data[12] *= factor.x; self.data[13] *= factor.y; self.data[14] *= factor.z;
	}

	/// Creates a vec<DrawCoord> suitable for WebGL to process. (So it transposes the matrix.)
	pub fn export(&self) -> Vec<DrawCoord> {
		vec!(
			self.data[0], self.data[4], self.data[ 8], self.data[12],
			self.data[1], self.data[5], self.data[ 9], self.data[13],
			self.data[2], self.data[6], self.data[10], self.data[14],
			self.data[3], self.data[7], self.data[11], self.data[15],
		)
	}
}
