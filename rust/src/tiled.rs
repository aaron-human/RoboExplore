use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::cell::Cell;
use std::ptr;

use crate::externals::*;
use crate::geo::vec2::*;

pub type TiledTileId = u32;

/// All relevant data in a given TiledFile.
pub struct TiledFile {
	/// All the tiles, the tile's ID is its index.
	tiles : Vec<TiledTile>,
	/// All the tile layers.
	tile_layers : Vec<TiledTileLayer>,
	/// Important points.
	points : HashMap<String, Vec2>,
}

impl TiledFile {
	/// Creates a new bullet.
	pub fn new() -> TiledFile {
		TiledFile {
			tiles : Vec::new(),
			tile_layers : Vec::new(),
			points : HashMap::new(),
		}
	}

	/// Gets a reference to the tile's data.
	pub fn get_tiles<'a>(&'a self) -> &'a Vec<TiledTile> {
		&self.tiles
	}

	/// Gets a reference to the tile's data.
	pub fn get_tile<'a>(&'a self, id : TiledTileId) -> &'a TiledTile {
		&self.tiles[id as usize]
	}

	/// Gets a reference to the layers.
	pub fn get_tile_layers<'a>(&'a self) -> &'a Vec<TiledTileLayer> {
		&self.tile_layers
	}

	/// Gets the number of registered tiles.
	pub fn tile_count(&self) -> usize {
		self.tiles.len()
	}

	/// Gets the number of layers.
	pub fn layer_count(&self) -> usize {
		for layer in &self.tile_layers {
			log(&format!("Have layer {:?}", layer.name));
		}
		self.tile_layers.len()
	}
}

/// A specific tile's info.
pub struct TiledTile {
	/// The texture image to use.
	image_url : String,
	/// The position in the image file.
	position : Vec2,
	/// The tile's size in the image file.
	size : Vec2,
}

impl TiledTile {
	/// Gets the URL of the image that this tile comes from.
	pub fn get_image_url<'a>(&'a self) -> &'a str {
		&self.image_url
	}

	/// Gets the position of the tile in the source image.
	pub fn get_position(&self) -> Vec2 {
		self.position.clone()
	}

	/// Gets the size of the tile.
	pub fn get_size(&self) -> Vec2 {
		self.size.clone()
	}
}

/// A single tile layer.
pub struct TiledTileLayer {
	/// The layer's name.
	name : String,
	/// The offset of the entire layer.
	offset : Vec2,
	/// The width (in tiles).
	width : usize,
	/// The height (in tiles).
	height : usize,
	/// The tile indices (in row-major format).
	tile_data : Vec<TiledTileId>,
}

impl TiledTileLayer {
	/// Gets the name.
	pub fn get_name<'a>(&'a self) -> &'a str {
		&self.name
	}

	/// Gets the position offset.
	pub fn get_offset(&self) -> Vec2 {
		self.offset.clone()
	}

	/// Gets the width (in terms of tiles).
	pub fn get_width(&self) -> usize {
		self.width
	}

	/// Gets the height (in terms of tiles).
	pub fn get_height(&self) -> usize {
		self.height
	}

	/// Gets the ID of the gile at a given location.
	pub fn get_tile_id(&self, x : usize, y : usize) -> TiledTileId {
		self.tile_data[x + y * self.width]
	}
}

/// The type of fuction to call when a TiledFile is loaded.
pub type TiledDoneCallback = fn(Cell<TiledFile>);

/// Everything needed to handle calling out to JavaScript to load a Tiled object, and then conclude with a call back into some arbitrary Rust code.
struct TiledGenerator {
	/// A mapping from requested URLs to their corresponding TiledDoneCallbacks.
	callbacks : HashMap<String, TiledDoneCallback>,
	/// The current TiledFile object being loaded.
	current : Cell<TiledFile>,
}

impl TiledGenerator {
	fn new() -> TiledGenerator {
		TiledGenerator {
			callbacks : HashMap::new(),
			current : Cell::new(TiledFile::new()),
		}
	}

	/// Adds a callback for the given URL.
	/// DO NOT setup multiple callbacks for a single URL!
	/// TODO: Allow multiple callbacks per URL?
	fn add_callback(&mut self, url : &str, callback : TiledDoneCallback) {
		let key = url.to_string();
		assert!(!self.callbacks.contains_key(&key), "Callback for this URL aready setup: {:?}", url);
		self.callbacks.insert(key, callback);
	}

	/// Concludes a callback for the given URL using the current file.
	fn conclude(&mut self, url : &str) {
		log(&format!("Concluding {:?}", url));
		let key = url.to_string();
		assert!(self.callbacks.contains_key(&key), "No callback for URL: {:?}", url);
		let callback = self.callbacks.remove(&key);
		let other = Cell::new(TiledFile::new());
		other.swap(&self.current);
		callback.unwrap()(other);
	}
}

static mut TILED_FILE_GENERATOR : *mut TiledGenerator = ptr::null_mut();

/// Starts loading a TiledFile from the given URL. Will call the callback with the item when its ready.
pub fn load_tiled_file(url : &str, callback : TiledDoneCallback) {
	let generator;
	unsafe {
		if TILED_FILE_GENERATOR.is_null() {
			TILED_FILE_GENERATOR = Box::into_raw(Box::new(TiledGenerator::new()));
		}
		generator = &mut *TILED_FILE_GENERATOR;
	}
	generator.add_callback(url, callback);
	startTiledFileLoad(url);
}

// =============== All the functions that JavaScript calls are below. ===============

/// Called to add a tile. The tile's ID is implied by its .
#[wasm_bindgen]
pub fn tiled_generate_add_tile(image_url : String, x : u16, y : u16, width : u16, height : u16) {
	let generator;
	unsafe {
		generator = &mut *TILED_FILE_GENERATOR;
	}
	let file = generator.current.get_mut();
	file.tiles.push(TiledTile{
		image_url: image_url,
		position: Vec2::new(x as f32, y as f32),
		size: Vec2::new(width as f32, height as f32),
	});
}

#[wasm_bindgen]
pub fn tiled_generate_add_tile_layer(name : String, x_offset : f32, y_offset : f32, width : usize, height : usize, data : Vec<TiledTileId>) {
	let generator;
	unsafe {
		generator = &mut *TILED_FILE_GENERATOR;
	}
	let file = generator.current.get_mut();
	file.tile_layers.push(TiledTileLayer{
		name,
		offset : Vec2::new(x_offset, y_offset),
		width, height,
		tile_data : data,
	});
}

#[wasm_bindgen]
pub fn tiled_generation_done(url : &str) {
	let generator;
	unsafe {
		generator = &mut *TILED_FILE_GENERATOR;
	}
	generator.conclude(url);
}
