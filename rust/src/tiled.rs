use wasm_bindgen::prelude::*;
use std::rc::Rc;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use crate::externals::*;
use crate::static_singletons::{get_tiled_generator, get_game};
use crate::geo::vec2::*;

pub type TiledTileId = u32;

/// All relevant data in a given TiledFile.
pub struct TiledFile {
	/// Whether this file is being loaded.
	is_loading : bool,
	/// The URL the data is being loaded from.
	url : String,
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
			is_loading : false,
			url : "".to_string(),
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

/// The main way TiledFile objects are loaded in.
///
/// This is basically a Rc<RefCell<TiledFile>>, with a little extra to make it easier to work with.
#[derive(Clone)]
pub struct SharedTiledFile {
	/// The TiledFile instance.
	file : Rc<RefCell<TiledFile>>,
}

impl SharedTiledFile {
	/// Creates an empty instance.
	pub fn new() -> SharedTiledFile {
		SharedTiledFile {
			file : Rc::new(RefCell::new(TiledFile::new())),
		}
	}

	/// Gets a mutable reference to the TiledFile instance.
	///
	/// Will return `None` if the TiledFile is already in use.
	pub fn get<'a>(&'a mut self) -> Option<RefMut<'a, TiledFile>> {
		match self.file.try_borrow_mut() {
			Ok(reference) => {
				if !reference.is_loading {
					Some(reference)
				} else {
					None
				}
			},
			Err(_) => None,
		}
	}

	/// Loads in data from a given URL.
	///
	/// Loading in the same URL using separate TiledFile instances will lead to an error.
	pub fn load(&mut self, url : &str) -> Result<(), ()> {
		let mut ok = false;
		if let Ok(reference) = self.file.try_borrow() {
			ok = !reference.is_loading;
		}
		if ok {
			get_tiled_generator().start_loading(url, self)
		} else {
			Err(())
		}
	}
}

//======================================================================================================================
// Below is all the stuff for generating TiledFile objects.
//======================================================================================================================

/// Everything needed to handle calling out to JavaScript to load a Tiled object, and then conclude with a call back
/// into some arbitrary Rust code.
///
/// **NEVER create this.** There's a singleton instance already hooked up in `static_singletons`.
pub struct TiledGenerator {
	/// A mapping from tiled file URLS to the SharedTileFile instances currently being loaded.
	current : HashMap<String, SharedTiledFile>,
}

impl TiledGenerator {
	pub fn new() -> TiledGenerator {
		TiledGenerator {
			current : HashMap::new(),
		}
	}

	/// Starts loading a given SharedTiledFile.
	fn start_loading(&mut self, url : &str, shared : &SharedTiledFile) -> Result<(),()> {
		if self.current.contains_key(url) {
			return Err(());
		}
		// Otherwise good to go.
		{
			let mut file = shared.file.borrow_mut();
			file.is_loading = true;
			file.url = url.to_string();
		}
		self.current.insert(url.to_string(), shared.clone());
		startTiledFileLoad(url);
		Ok(())
	}

	fn borrow_file(&self, url : &str) -> RefMut<'_, TiledFile> {
		assert!(self.current.contains_key(url), "Attempting to update Tiled file {:?} that is no longer stored in the generator!", url);
		self.current.get(url).unwrap().file.borrow_mut()
	}

	/// Concludes a callback for the given URL using the current file.
	fn conclude(&mut self, url : &str) {
		log(&format!("Concluding {:?}", url));
		let completed = self.current.remove(url).unwrap();
		{
			let mut file = completed.file.borrow_mut();
			file.is_loading = false;
		}
		get_game().handle_tiled_file_loaded(url, completed);
	}
}

// =============== All the functions that JavaScript calls are below. ===============

/// Called to add a tile. The tile's ID is implied by its .
#[wasm_bindgen]
pub fn tiled_generate_add_tile(file_url : String, image_url : String, x : u16, y : u16, width : u16, height : u16) {
	get_tiled_generator().borrow_file(&file_url).tiles.push(TiledTile{
		image_url: image_url,
		position: Vec2::new(x as f32, y as f32),
		size: Vec2::new(width as f32, height as f32),
	});
}

#[wasm_bindgen]
pub fn tiled_generate_add_tile_layer(file_url : String, name : String, x_offset : f32, y_offset : f32, width : usize, height : usize, data : Vec<TiledTileId>) {
	get_tiled_generator().borrow_file(&file_url).tile_layers.push(TiledTileLayer{
		name,
		offset : Vec2::new(x_offset, y_offset),
		width, height,
		tile_data : data,
	});
}

#[wasm_bindgen]
pub fn tiled_generation_done(url : &str) {
	get_tiled_generator().conclude(url);
}
