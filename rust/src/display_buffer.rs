use crate::externals::*;
use crate::geo::vec3::*;
use crate::geo::mat4::*;
use crate::color::*;
use std::f32::consts::PI;

#[derive(PartialEq)]
pub enum DisplayBufferType {
	SOLID,
	LINES,
}

//impl Eq for DisplayBufferType {}

pub struct DisplayBuffer {
	id : DrawBufferID, // The reference to the external JS buffer.
	vertices : Vec<DrawCoord>, // A vector of raw vertex values.
	colors : Vec<ColorMagnitude>, // A vector of raw vertex values.
	indices : Vec<DrawIndex>, // A vector of raw vertex index values.
	buffers_dirty : bool, // Whether the above has been modified since it was last updated.
	pub transform : Mat4, // The transform matrix to apply to the buffers at render time.
	type_ : DisplayBufferType, // What sort of drawing this wll do.
}



impl DisplayBuffer {
	pub fn new(type_ : DisplayBufferType) -> DisplayBuffer {
		DisplayBuffer {
			id : createDrawBuffer(
				match type_ {
					DisplayBufferType::SOLID => 0,
					DisplayBufferType::LINES => 1,
				}
			),
			vertices : Vec::new(),
			colors : Vec::new(),
			indices : Vec::new(),
			buffers_dirty : false,
			transform : Mat4::new(),
			type_ : type_,
		}
	}

	/// Clears out all stored geometry.
	pub fn clear(&mut self) {
		self.vertices.clear();
		self.colors.clear();
		self.indices.clear();
		self.buffers_dirty = true;
	}

	/// Stores a vertex.
	fn store_vertex(&mut self, position : &Vec3, color : &Color) {
		self.vertices.push(position.x);
		self.vertices.push(position.y);
		self.vertices.push(position.z);

		self.colors.push(color.red);
		self.colors.push(color.green);
		self.colors.push(color.blue);
		self.colors.push(color.alpha);
	}

	/// Adds a triangle.
	pub fn add_triangle(&mut self, points : [Vec3; 3], color : &Color) {
		let index : u16 = (self.vertices.len() / 3) as u16;

		self.store_vertex(&points[0], color);
		self.indices.push(index + 0);

		self.store_vertex(&points[1], color);
		self.indices.push(index + 1);

		self.store_vertex(&points[2], color);
		self.indices.push(index + 2);

		self.buffers_dirty = true;
	}

	/// Adds a polygon. This will either be a line loop or a filled shape.
	/// Only convex polygons are guaranteed to be filled everywhere.
	pub fn add_polygon(&mut self, points : &Vec<Vec3>, color : &Color) {
		let start : u16 = (self.vertices.len() / 3) as u16;

		// Always add all the points.
		for point in points {
			self.store_vertex(point, color);
		}

		let length = points.len() as u16;
		match self.type_ {
			DisplayBufferType::SOLID => {
				// Creates a triangle fan centered around the first point.
				for index in 2..length {
					self.indices.push(start + 0);
					self.indices.push(start + index - 1);
					self.indices.push(start + index);
				}
			},
			DisplayBufferType::LINES => {
				// Just draws all of the lines separately.
				for index in 0..length-1 {
					self.indices.push(start + index);
					self.indices.push(start + index + 1);
				}
				self.indices.push(start + length - 1);
				self.indices.push(start + 0);
			},
		}

		self.buffers_dirty = true;
	}

	/// Adds a circle on the x-y plane (facing the viewer).
	/// @param center The center of the circle.
	/// @param radius The radius of the circle.
	/// @param count The number of segments to make the circle out of.
	/// @param color The color to use.
	pub fn add_circle(&mut self, center : Vec3, radius : DrawCoord, count : i32, color : &Color) {
		let mut circle : Vec<Vec3> = Vec::new();
		let to_radians = 2.0 * PI  / (count as f32);
		for index in 0..count {
			let mut position = center.clone();
			position.x += ((index as f32) * to_radians).cos() * radius;
			position.y += ((index as f32) * to_radians).sin() * radius;
			circle.push(position);
		}
		self.add_polygon(&circle, color);
	}

	/// Adds a series of lines.
	/// Panics if this is called on a SOLID type.
	pub fn add_lines(&mut self, points : Vec<Vec3>, color : &Color) {
		if DisplayBufferType::SOLID == self.type_ {
			panic!("Can't call add_lines() on a SOLID type DisplayBuffer!");
		}

		let start_index : u16 = (self.vertices.len() / 3) as u16;
		self.store_vertex(&points[0], color);
		for index in 1..points.len() {
			self.store_vertex(&points[index], color);
			self.indices.push(start_index + (index as u16) - 1);
			self.indices.push(start_index + (index as u16));
		}
		self.buffers_dirty = true;
	}

	/// Makes sure the buffer is shown, and immediately update()s.
	pub fn show(&mut self) {
		setDisplayBufferVisibility(self.id, true);
		self.update();
	}

	/// Makes sure the buffer is hidden, and immediately update()s.
	pub fn hide(&mut self) {
		setDisplayBufferVisibility(self.id, false);
		self.update();
	}

	/// Updates the buffer if anything has changed.
	pub fn update(&mut self) {
		if self.buffers_dirty {
			setDisplayBuffer(self.id, &self.vertices, &self.colors, &self.indices);
		}
		self.buffers_dirty = false;
		setDisplayBufferTransform(self.id, self.transform.export());
	}
}

impl Drop for DisplayBuffer {
	/// Remove the buffer.
	/// The TypeScript side of things will re-use it later.
	/// Using TypeScript for that to keep the DisplayBuffer::new() calls simple.
	fn drop(&mut self) {
		deleteDrawBuffer(self.id);
	}
}
