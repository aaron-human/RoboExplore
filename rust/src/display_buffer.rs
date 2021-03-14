use crate::externals::*;
use crate::geo::vec3::*;
use crate::geo::vec2::*;
use crate::geo::mat4::*;
use crate::color::*;
use crate::display_texture::DisplayTexture;
use std::f32::consts::PI;

#[derive(PartialEq)]
pub enum DisplayBufferType {
	SOLIDS,
	LINES,
	IMAGES,
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
					DisplayBufferType::SOLIDS => 0,
					DisplayBufferType::LINES => 1,
					DisplayBufferType::IMAGES => 2,
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
	fn store_vertex(&mut self, position : &Vec3, color : &dyn ColorExportable) {
		self.vertices.push(position.x);
		self.vertices.push(position.y);
		self.vertices.push(position.z);

		color.raw_export(&mut self.colors);
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
			DisplayBufferType::SOLIDS => {
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
			DisplayBufferType::IMAGES => panic!("DisplayBuffers of type IMAGES cannot use add_polygon()"),
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
		if DisplayBufferType::LINES != self.type_ {
			panic!("Can only call add_lines() on a LINES type DisplayBuffer!");
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
		assert!(setDisplayBufferVisibility(self.id, true), "Couldn't set visibiltiy of display buffer {}", self.id);
		self.update();
	}

	/// Makes sure the buffer is hidden, and immediately update()s.
	pub fn hide(&mut self) {
		assert!(setDisplayBufferVisibility(self.id, false), "Couldn't set visibiltiy of display buffer {}", self.id);
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

	pub fn add_image(&mut self, source_position : &Vec2, size : &Vec2, destination_position : &Vec3) {
		if DisplayBufferType::IMAGES != self.type_ {
			panic!("Can only call add_image() on a IMAGES type DisplayBuffer!");
		}

		let start_index : u16 = (self.vertices.len() / 3) as u16;
		let mut texture_position = TexturePositionAsColor::new(
			source_position.x as u16,
			source_position.y as u16,
		);
		let mut position = destination_position.clone();
		self.store_vertex(&position, &texture_position);
		position.x         += size.x;
		texture_position.x += size.x as u16;
		self.store_vertex(&position, &texture_position);
		position.y         += size.y;
		texture_position.y += size.y as u16;
		self.store_vertex(&position, &texture_position);
		position.x         -= size.x;
		texture_position.x -= size.x as u16;
		self.store_vertex(&position, &texture_position);

		self.indices.push(start_index + 0);
		self.indices.push(start_index + 1);
		self.indices.push(start_index + 2);

		self.indices.push(start_index + 0);
		self.indices.push(start_index + 2);
		self.indices.push(start_index + 3);

		self.buffers_dirty = true;
	}

	/// Sets the associated texture.
	///
	/// Since this will happen infrequently, it's done immediately rather than being put off until the next update() call.
	pub fn set_texture(&mut self, texture : &DisplayTexture) { // TODO: Could store this as an Rc<RefCell<DisplayTexture>> so the texture would be guaranteed to be kept until all associated buffers are deleted?
		if DisplayBufferType::IMAGES != self.type_ {
			panic!("Can only call set_texture() on a IMAGES type DisplayBuffer!");
		}

		let texture_id = texture.get_id();
		assert!(setDrawBufferTexture(self.id, texture_id), "Couldn't set display buffer {} to use texture {}", self.id, texture_id);
	}
}

impl Drop for DisplayBuffer {
	/// Remove the buffer.
	/// The TypeScript side of things will re-use it later.
	/// Using TypeScript for that to keep the DisplayBuffer::new() calls simple.
	fn drop(&mut self) {
		assert!(deleteDrawBuffer(self.id), "Couldn't delete draw buffer {}", self.id);
	}
}
