use std::collections::HashSet;
use std::f32::INFINITY;

use crate::externals::log;

use crate::geo::bounds2::Bounds2;
use crate::geo::vec2::Vec2;
use crate::geo::vec3::Vec3;
use crate::color::Color;

use crate::tiled::{TiledFile, TiledTileLayer};

use crate::display_buffer::{DisplayBuffer, DisplayBufferType};

/// A way to store a pneumatic pipe between two locations.
pub struct PneumaticPipe {
	/// The "starting" end's collision geometry
	start_collision : Bounds2,
	/// The "ending" end's collision geometry.
	end_collision : Bounds2,
	/// The path from the start to the end.
	path : Vec<Vec2>,
}

impl PneumaticPipe {
	/// Gets the path of points to traverse through.
	pub fn get_path<'a>(&'a self) -> &'a Vec<Vec2> {
		&self.path
	}
}

// A way to store directions in a single u8.
const DIR_UP    : u8 = 0b0001;
const DIR_LEFT  : u8 = 0b0010;
const DIR_DOWN  : u8 = 0b0100;
const DIR_RIGHT : u8 = 0b1000;

/// A place to store geometry for the underlying tile map.
pub struct TiledGeometry {
	/// The rectangles that represent "tracks".
	tracks : Vec<Bounds2>,
	/// The rectangles to collide with.
	collision_rects : Vec<Bounds2>,
	/// The polygons to collide with.
	collision_polygons : Vec<Vec<Vec2>>,
	/// All of the level's penumatic pipes.
	pneumatic_pipes : Vec<PneumaticPipe>,
	/// A debugging buffer to show all the geometry with.
	pub debug_buffer : DisplayBuffer,
}

impl TiledGeometry {
	pub fn new() -> TiledGeometry {
		TiledGeometry {
			tracks : Vec::new(),
			collision_rects : Vec::new(),
			collision_polygons : Vec::new(),
			pneumatic_pipes : Vec::new(),
			debug_buffer : DisplayBuffer::new(DisplayBufferType::LINES),
		}
	}

	/// The collision rectangle geometry.
	pub fn get_collision_rects<'a>(&'a self) -> &'a Vec<Bounds2> {
		&self.collision_rects
	}

	/// The collision polygon geometry.
	pub fn get_collision_polygons<'a>(&'a self) -> &'a Vec<Vec<Vec2>> {
		&self.collision_polygons
	}

	/// Finds the closest point inside the tracts.
	pub fn get_closest_track_point(&self, position : &Vec2) -> Vec2 {
		let mut closest = Vec2::new(0.0, 0.0);
		let mut closest_distance = INFINITY;
		for rect in &self.tracks {
			let limited = Vec2::new(
				position.x.min(rect.x_max()).max(rect.x_min()),
				position.y.min(rect.y_max()).max(rect.y_min()),
			);
			let distance = (limited - position).length();
			if distance < closest_distance {
				closest = limited;
				closest_distance = distance;
			}
		}
		closest
	}

	/// Finds the closest point on a track that intersects with a given moving point.
	pub fn collide_moving_point_with_track(&self, position : &Vec2, movement : &Vec2) -> Option<Vec2> {
		let end = position + movement;
		let mut closest = None;
		let mut closest_distance = INFINITY;
		for rect in &self.tracks {
			if let Some(intersection) = rect.collide_with_line_segment(&position, &end) {
				let distance = (position - intersection).length();
				if distance < closest_distance {
					closest = Some(intersection);
					closest_distance = distance;
				}
			}
		}
		closest
	}

	/// Gets the pneumatic pipe that the position is currently inside (if any).
	pub fn get_activated_pneumatic_pipe<'a>(&'a self, position : &Vec2, movement : &Vec2) -> Option<(Vec2, bool, &'a PneumaticPipe)> {
		let end = position + movement;
		for pipe in &self.pneumatic_pipes {
			let maybe_hit = pipe.start_collision.collide_with_line_segment(position, &end);
			if let Some(hit) = maybe_hit {
				return Some((hit, true, pipe));
			}
			let maybe_hit = pipe.end_collision.collide_with_line_segment(position, &end);
			if let Some(hit) = maybe_hit {
				return Some((hit, false, pipe));
			}
		}
		None
	}

	/// Collects a given penumatic pipe from the given input layer.
	fn load_pneumatic_pipe(&mut self, file : &TiledFile, layer : &TiledTileLayer, mut x : usize, mut y : usize, used_positions : &mut Vec<usize>) -> Result<PneumaticPipe, String> {
		let layer_width  = layer.get_width();
		let layer_height = layer.get_height();
		let start_position = x + y * layer_width;
		let mut tile_space = layer.get_size(); // How much space to give the tile. It may not use it all.
		tile_space.x /= layer.get_width()  as f32;
		tile_space.y /= layer.get_height() as f32;
		let mut path : Vec<Vec2> = Vec::new();
		let mut maybe_start : Option<Bounds2> = None;
		let mut maybe_end : Option<Bounds2> = None;
		let mut done = false;
		let mut error : Option<String> = None;
		let mut next_source : u8 = 0;
		let mut direction : u8;
		while !done && error.is_none() {
			// First load in the current tile.
			let tile = file.get_tile(layer.get_tile_id(x, y));
			let offset = TiledGeometry::get_tile_offset(layer, x, y);
			direction = 0;
			for rect in tile.get_collision_rectangles() {
				if "pipeEnter" == rect.r#type {
					if maybe_start.is_none() {
						let mut translated = rect.position.clone();
						translated.translate(&offset);
						maybe_start = Some(translated);
					} else if maybe_end.is_none() {
						let mut translated = rect.position.clone();
						translated.translate(&offset);
						maybe_end = Some(translated);
						done = true;
					}
				}
				if "pipeTravel" == rect.r#type {
					if 0.0          == rect.position.x_min() { direction |= DIR_LEFT; }
					if 0.0          == rect.position.y_min() { direction |= DIR_DOWN; }
					if tile_space.x == rect.position.x_max() { direction |= DIR_RIGHT; }
					if tile_space.y == rect.position.y_max() { direction |= DIR_UP; }
				}
			}

			// If it's done, then complete the path and return.
			if done {
				path.push(offset + tile_space * 0.5);
				break;
			}

			if 0 == next_source || (DIR_LEFT | DIR_RIGHT) != direction || (DIR_UP | DIR_DOWN) != direction {
				// If just started, then store the current location as the starting point.
				path.push(offset + tile_space * 0.5);
			}
			if 0 != next_source {
				// If not just starting out, then remove the previous source direction so know which way is forward.
				direction &= !next_source;
			}

			// If it's not done, then continue moving forward.
			match direction {
				DIR_UP  => {
					if 0 < y {
						y -= 1;
						next_source = DIR_DOWN;
					} else {
						error = Some(format!("Pipe hit edge at ({},{})", x, y));
					}
				},
				DIR_DOWN  => {
					if y+1 < layer_height {
						y += 1;
						next_source = DIR_UP;
					} else {
						error = Some(format!("Pipe hit edge at ({},{})", x, y));
					}
				},
				DIR_RIGHT  => {
					if x+1 < layer_width {
						x += 1;
						next_source = DIR_LEFT;
					} else {
						error = Some(format!("Pipe hit edge at ({},{})", x, y));
					}
				},
				DIR_LEFT  => {
					if 0 < x {
						x -= 1;
						next_source = DIR_RIGHT;
					} else {
						error = Some(format!("Pipe hit edge at ({},{})", x, y));
					}
				},
				0b0000 => { error = Some(format!("Pipe abrubtly ended at ({},{})", x, y)); },
				_ => { error = Some(format!("Weird pipe tile at ({},{}): direction={}", x, y, direction)); },
			}
			// Then update the path.
		}
		if let Some(error_message) = error {
			Err(error_message)
		} else {
			used_positions.push(start_position);
			used_positions.push(x + y * layer_width);
			Ok(PneumaticPipe {
				start_collision : maybe_start.unwrap(),
				end_collision : maybe_end.unwrap(),
				path,
			})
		}
	}

	/// Gets the offset position of a tile in a layer.
	fn get_tile_offset(layer : &TiledTileLayer, x : usize, y : usize) -> Vec2 {
		let mut tile_space = layer.get_size(); // How much space to give the tile. It may not use it all.
		tile_space.x /= layer.get_width()  as f32;
		tile_space.y /= layer.get_height() as f32;
		layer.get_offset() + Vec2::new(
			(x as f32) * tile_space.x,
			((layer.get_height() - y - 1) as f32) * tile_space.y,
		)
	}

	/// Loads in all data from a TiledFile instance.
	pub fn load_from(&mut self, file : &TiledFile) {
		// First pass: Extract all collision information from the map.
		for layer in file.get_tile_layers() {
			let layer_width = layer.get_width();
			let layer_height = layer.get_height();
			let mut used_pipe_entrance_positions : Vec<usize> = Vec::new();
			for y in 0..layer_height {
				for x in 0..layer_width {
					let int_position = x + y * layer_width;
					let tile = file.get_tile(layer.get_tile_id(x, y));
					let tile_offset = TiledGeometry::get_tile_offset(layer, x, y);
					for rect in tile.get_collision_rectangles() {
						if "collision" == rect.r#type {
							let mut final_copy = rect.position.clone();
							final_copy.translate(&tile_offset);
							self.collision_rects.push(final_copy);
						}
						if "track" == rect.r#type {
							let mut final_copy = rect.position.clone();
							final_copy.translate(&tile_offset);
							self.tracks.push(final_copy);
						}
						if "pipeEnter" == rect.r#type {
							let mut make_new = true;
							for used in &used_pipe_entrance_positions {
								if *used == int_position {
									make_new = false;
									break;
								}
							}
							if make_new {
								match self.load_pneumatic_pipe(file, layer, x, y, &mut used_pipe_entrance_positions) {
									Ok(pipe) => { self.pneumatic_pipes.push(pipe); },
									Err(error) => { log(&format!("Couldn't load pipe starting at {},{} in map {:?} due to: {}", x, y, file.get_url(), error)); },
								}
							}
						}
					}
					for polygon in tile.get_collision_polygons() {
						if "collision" == polygon.r#type {
							let mut final_copy = Vec::with_capacity(polygon.points.len());
							for point in &polygon.points {
								final_copy.push(point + tile_offset);
							}
							self.collision_polygons.push(final_copy);
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
		self.tracks = simplify_rects(&mut self.tracks);
		// For debugging: draw all the rectangles.
		if true {
			let mut editor = self.debug_buffer.make_editor();
			editor.clear();
			if false {
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
				for polygon in &self.collision_polygons {
					let mut points : Vec<Vec3> = Vec::with_capacity(polygon.len());
					for source in polygon {
						points.push(Vec3::new(source.x, source.y, z));
					}
					editor.add_polygon(
						&points,
						&color,
					);
				}
			}
			{
				let color = Color::new(255, 0, 0, 255);
				let z : f32 = -0.75;
				for pipe in &self.pneumatic_pipes {
					for index in 0..(pipe.path.len()-1) {
						editor.add_lines(
							vec![
								Vec3::new(pipe.path[index  ].x, pipe.path[index  ].y, z),
								Vec3::new(pipe.path[index+1].x, pipe.path[index+1].y, z),
							],
							&color,
						);
					}
					editor.add_polygon(
						&vec![
							Vec3::new(pipe.start_collision.x_min(), pipe.start_collision.y_min(), z),
							Vec3::new(pipe.start_collision.x_max(), pipe.start_collision.y_min(), z),
							Vec3::new(pipe.start_collision.x_max(), pipe.start_collision.y_max(), z),
							Vec3::new(pipe.start_collision.x_min(), pipe.start_collision.y_max(), z),
						],
						&color,
					);
					editor.add_polygon(
						&vec![
							Vec3::new(pipe.end_collision.x_min(), pipe.end_collision.y_min(), z),
							Vec3::new(pipe.end_collision.x_max(), pipe.end_collision.y_min(), z),
							Vec3::new(pipe.end_collision.x_max(), pipe.end_collision.y_max(), z),
							Vec3::new(pipe.end_collision.x_min(), pipe.end_collision.y_max(), z),
						],
						&color,
					);
				}
			}
			if false {
				let color = Color::new(0, 255, 0, 255);
				let z : f32 = -0.85;
				for rect in &self.tracks {
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