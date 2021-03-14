use super::consts::*;

/// Converts a value into a unit-length value with the same sign.
pub fn sign(value : f32) -> f32 {
	if value.abs() < EPSILON {
		0.0
	} else if value < 0.0 {
		-1.0
	} else {
		1.0
	}
}