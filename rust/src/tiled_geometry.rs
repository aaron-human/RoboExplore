use crate::geo::bounds2::Bounds2;
use crate::geo::vec2::Vec2;
use crate::geo::vec3::Vec3;
use crate::color::Color;

use crate::tiled::TiledFile;

use crate::display_buffer::{DisplayBuffer, DisplayBufferType};


/// A place to store geometry for the underlying tile map.
pub struct TiledGeometry {
	/// The rectangles to collide with.
	rects : Vec<Bounds2>,
	/// A debugging buffer to show all the geometry with.
	pub debug_buffer : DisplayBuffer,
}

impl TiledGeometry {
	pub fn new() -> TiledGeometry {
		TiledGeometry {
			rects : Vec::new(),
			debug_buffer : DisplayBuffer::new(DisplayBufferType::LINES),
		}
	}

	/// Loads in all data from a TiledFile instance.
	pub fn load_from(&mut self, file : &TiledFile) {
		// First pass: Extract all collision information from the map.
		for layer in file.get_tile_layers() {
			let layer_width = layer.get_width();
			let layer_height = layer.get_height();
			let layer_offset = layer.get_offset();
			let mut tile_space = layer.get_size(); // How much space to give the tile. It may not use it all.
			tile_space.x /= layer_width as f32;
			tile_space.y /= layer_height as f32;
			for y in 0..layer_height {
				for x in 0..layer_width {
					let tile = file.get_tile(layer.get_tile_id(x, y));
					let tile_offset = Vec2::new(
						layer_offset.x + (x as f32) * tile_space.x,
						layer_offset.y + ((layer_height - y - 1) as f32) * tile_space.y,
					);
					for rect in tile.get_collision_rectangles() {
						if "collision" == rect.r#type {
							let mut collision = rect.position.clone();
							collision.translate(&tile_offset);
							self.rects.push(collision);
						}
					}
					for property in tile.get_boolean_properties() {
						if "solid" == property.name && property.value {
							self.rects.push(Bounds2::from_points(
								&tile_offset,
								&(tile_offset + tile.get_size()),
							));
							break;
						}
					}
				}
			}
		}
		// Second pass: Combine rectangles that share a common top/bottom boundary.
		// Third pass: Combine rectangles that share a common left/right boundary.
		// For debugging: draw all the rectangles.
		{
			let mut editor = self.debug_buffer.make_editor();
			editor.clear();
			let color = Color::new(255, 0, 0, 255);
			let z : f32 = -0.8;
			for rect in &self.rects {
				editor.add_polygon(
					&vec![
						Vec3::new(rect.x_min(), rect.y_min(), z),
						Vec3::new(rect.x_max(), rect.y_min(), z),
						Vec3::new(rect.x_max(), rect.y_max(), z),
						Vec3::new(rect.x_min(), rect.y_max(), z),
					],
					&color,
				);
			}
		}
	}
}
