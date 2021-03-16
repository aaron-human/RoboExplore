use std::collections::HashMap;

use crate::geo::vec3::*;
use crate::display_buffer::*;
use crate::display_texture::*;
use crate::tiled::*;

/// A way to display a TiledFile using DisplayBuffers and DisplayTextures.
pub struct TiledDisplay {
	/// A mapping from display texture URLs to the DisplayTexture objects.
	textures : HashMap<String, DisplayTexture>,
	/// The display buffers in display order (back to front).
	buffers : Vec<DisplayBuffer>
}

impl TiledDisplay {
	/// Creates an empty instance.
	pub fn new() -> TiledDisplay {
		TiledDisplay {
			textures : HashMap::new(),
			buffers : Vec::new(),
		}
	}

	/// Loads in all data from a TiledFile instance.
	pub fn load_from(&mut self, file : &TiledFile) {
		self.textures.clear();
		self.buffers.clear();
		for tile in file.get_tiles() {
			let url = tile.get_image_url();
			if 0 == url.len() { continue; }
			if !self.textures.contains_key(url) {
				let mut texture = DisplayTexture::new();
				texture.load_from_url(url);
				self.textures.insert(url.to_string(), texture);
			}
		}

		for (layer_index, layer) in file.get_tile_layers().iter().enumerate() {
			let mut buffer = DisplayBuffer::new(DisplayBufferType::IMAGES);
			let mut tile_url = String::new();
			{
				let mut editor = buffer.make_editor();
				let width = layer.get_width();
				let height = layer.get_height();
				let offset = layer.get_offset();
				let depth = -(layer_index as f32) / 100.0;
				for y in 0..height {
					for x in 0..width {
						let tile = file.get_tile(layer.get_tile_id(x, y));
						let current_url = tile.get_image_url();
						if 0 == tile_url.len() && 0 < current_url.len() {
							tile_url = current_url.to_string();
						}
						let tile_size = tile.get_size();
						let position = Vec3::new(
							offset.x + (x as f32) * tile_size.x,
							offset.y + ((height - y - 1) as f32) * tile_size.y,
							depth,
						);
						editor.add_image(
							&tile.get_position(),
							&tile_size,
							&position,
						);
					}
				}
			}
			buffer.set_texture(self.textures.get(&tile_url).unwrap());
			self.buffers.push(buffer);
		}
	}
}
