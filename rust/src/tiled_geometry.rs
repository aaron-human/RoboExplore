use std::collections::HashSet;

use crate::geo::bounds2::Bounds2;
use crate::geo::vec2::Vec2;
use crate::geo::vec3::Vec3;
use crate::color::Color;

use crate::tiled::TiledFile;

use crate::display_buffer::{DisplayBuffer, DisplayBufferType};


/// A place to store geometry for the underlying tile map.
pub struct TiledGeometry {
	/// The rectangles to collide with.
	collision_rects : Vec<Bounds2>,
	/// A debugging buffer to show all the geometry with.
	pub debug_buffer : DisplayBuffer,
}

impl TiledGeometry {
	pub fn new() -> TiledGeometry {
		TiledGeometry {
			collision_rects : Vec::new(),
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
							self.collision_rects.push(collision);
						}
					}
					for property in tile.get_boolean_properties() {
						if "solid" == property.name && property.value {
							self.collision_rects.push(Bounds2::from_points(
								&tile_offset,
								&(tile_offset + tile.get_size()),
							));
							break;
						}
					}
				}
			}
		}
		self.collision_rects = simplify_rects(&mut self.collision_rects);
		// For debugging: draw all the rectangles.
		{
			let mut editor = self.debug_buffer.make_editor();
			editor.clear();
			let color = Color::new(255, 0, 0, 255);
			let z : f32 = -0.8;
			for rect in &self.collision_rects {
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

/// Iterates through every unique 2-pair of items, and passes them to a lambda function.
/// The iterator returns whether the right item should be skipped by future iterations.
/// The passed in skip_indices is updated to include any newly skipped items.
fn dual_iterate<T : Sized, FN>(source : &mut Vec<T>, skip_indices : &mut HashSet<usize>, iterator : FN)
	where FN : Fn(&mut T, & T) -> bool {
	let mut index = 0;
	while index < source.len() {
		if !skip_indices.contains(&index) {
			// Scan forward against all rectangles after to see if they match.
			let mut other_index = index + 1;
			while other_index < source.len() {
				if !skip_indices.contains(&other_index) {
					let (current_, other_) = source.split_at_mut(other_index);
					let current = &mut current_[index];
					let other = &other_[0];
					if iterator(current, other) {
						skip_indices.insert(other_index);
					}
				}
				other_index += 1;
			}
		}
		index += 1;
	}
}

/// Simplifies a set of rectangles that will often share edges.
fn simplify_rects(source : &mut Vec<Bounds2>) -> Vec<Bounds2> {
	// First pass: Combine rectangles that share a common top/bottom boundary.
	let mut removed_indices : HashSet<usize> = HashSet::new(); // TODO? Could optimize this a lot.
	dual_iterate(source, &mut removed_indices, |current, other| {
		if current.x_min() == other.x_min() && current.x_max() == other.x_max() {
			if current.y_min() == other.y_max() {
				current.expand_to_y(other.y_min());
				true
			} else if current.y_max() == other.y_min() {
				current.expand_to_y(other.y_max());
				true
			} else {
				false
			}
		} else {
			false
		}
	});
	// Second pass: Combine rectangles that share a common left/right boundary.
	dual_iterate(source, &mut removed_indices, |current, other| {
		if current.y_min() == other.y_min() && current.y_max() == other.y_max() {
			if current.x_min() == other.x_max() {
				current.expand_to_x(other.x_min());
				true
			} else if current.x_max() == other.x_min() {
				current.expand_to_x(other.x_max());
				true
			} else {
				false
			}
		} else {
			false
		}
	});
	// Last step: Remove the redundant geometry.
	let mut updated = Vec::with_capacity(source.len());
	for index in 0..source.len() {
		if !removed_indices.contains(&index) {
			updated.push(source[index].clone());
		}
	}
	updated
}