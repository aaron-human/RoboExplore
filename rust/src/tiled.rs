use wasm_bindgen::prelude::*;
use std::collections::HashMap;
use std::cell::Cell;
use std::ptr;

use crate::externals::*;
use crate::geo::vec2::*;

/// All relevant data in a given TiledFile.
pub struct TiledFile {
	/// All the tiles, the tile's ID is its index.
	tiles : Vec<TiledTile>,
	/// All the layers.
	layers : Vec<TiledLayer>,
	/// Important points.
	points : HashMap<String, Vec2>,
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

/// A single tile layer.
pub struct TiledLayer {
	/// The width (in tiles).
	width : usize,
	/// The height (in tiles).
	height : usize,
	/// The tile indices (in row-major format).
	tiles : Vec<usize>,
}

impl TiledFile {
	/// Creates a new bullet.
	pub fn new() -> TiledFile {
		TiledFile {
			tiles : Vec::new(),
			layers : Vec::new(),
			points : HashMap::new(),
		}
	}

	/// Gets the number of registered tiles.
	pub fn tile_count(&self) -> usize {
		self.tiles.len()
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
		assert!(self.callbacks.contains_key(&key), "Callback for this URL aready setup: {:?}", url);
		self.callbacks.insert(key, callback);
	}

	/// Concludes a callback for the given URL using the current file.
	fn conclude(&mut self, url : &str) {
		log(&format!("Concluding {:?}", url));
		let key = url.to_string();
		assert!(!self.callbacks.contains_key(&key), "No callback for URL: {:?}", url);
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
pub fn tiled_generation_done(url : &str) {
	let generator;
	unsafe {
		generator = &mut *TILED_FILE_GENERATOR;
	}
	generator.conclude(url);
}
