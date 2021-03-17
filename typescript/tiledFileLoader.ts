namespace ExampleProject {

	/// A class for storing collision rectangle info.
	class _CollisionRect {
		/// The type.
		public type : string = "";
		/// One of the edge x values.
		public x1 : number = 0;
		/// One of the edge y values.
		public y1 : number = 0;
		/// The other edge x value.
		public x2 : number = 0;
		/// The other edge y value.
		public y2 : number = 0;
	}

	/// A class for storing a boolean property.
	class _BooleanProperty {
		constructor(public name : string, public value : boolean) {
			//
		}
	}

	/// A class for storing partial information about a tile.
	class _PartialTileInfo {
		/// The image to get the tile from.
		public url : string = "";
		/// The x position.
		public x : number = 0;
		/// The y position.
		public y : number = 0;
		/// The width.
		public width : number = 1;
		/// The height.
		public height : number = 1;
		/// The properties.
		public properties : _BooleanProperty[] = []; // TODO: Add more property types.
		/// The collision rectangle info.
		public collisionRectangles : _CollisionRect[] = [];
	}

	type AddTileFunc = (url : string, imageUrl : string, x : number, y : number, width : number, height : number) => void;
	type AddTileBooleanPropertyFunc = (url : string, name : string, value : boolean) => void;
	type AddTileCollisionRectangleFunc = (url : string, type : string, x1 : number, y1 : number, x2 : number, y2 : number) => void;
	type AddTilePointFunc = (url : string, name : string, x : number, y : number) => void;
	type AddTileLayerFunc = (url : string, name : string, xOffset : number, yOffset : number, width : number, height : number, pixelWidth : number, pixelHeight : number, data : Uint32Array) => void;
	type OnDoneFunc = (url : string) => void;

	/**
	 * A class for loading in JSON exports from Tiled.
	 */
	export class TiledFileLoader {
		private _addTile : AddTileFunc = null;
		private _addTileBooleanProperty : AddTileBooleanPropertyFunc = null;
		private _addTileCollisionRectangle : AddTileCollisionRectangleFunc = null;
		private _addPoint : AddTilePointFunc = null;
		private _addTileLayer : AddTileLayerFunc = null;
		private _onDone : OnDoneFunc = null;

		/// Stores callbacks useful for loading tile info.
		public setup(addTile : AddTileFunc, addTileBooleanProperty : AddTileBooleanPropertyFunc, addTileCollisionRectangle : AddTileCollisionRectangleFunc, addPoint : AddTilePointFunc, addTileLayer : AddTileLayerFunc, onDone : OnDoneFunc) {
			this._addTile = addTile;
			this._addTileBooleanProperty = addTileBooleanProperty;
			this._addTileCollisionRectangle = addTileCollisionRectangle;
			this._addPoint = addPoint;
			this._addTileLayer = addTileLayer;
			this._onDone = onDone;
		}

		/// Starts loading the file at the given URL.
		public startLoading(url : string) {
			let sourceUrl = url;
			fetch(url).then(
				(response) => response.json()
			).then(function(json : any){
				// First map all the tile IDs to the data.
				const tileIdToInfo : Map<number, _PartialTileInfo> = new Map();
				let maxId : number = 0;
				const tilesets : any[] = json["tilesets"];
				for (let tilesetIndex = 0;tilesetIndex < tilesets.length;tilesetIndex += 1) {
					const tileset = tilesets[tilesetIndex];
					const tileWidth : number = tileset["tilewidth"];
					const tileImageHeight : number = tileset["imageheight"];
					if (undefined === tileWidth) {
						console.error(`Tileset #${tilesetIndex} has no "tilewidth" in file ${sourceUrl}`);
						continue;
					}
					const tileHeight : number = tileset["tileheight"];
					if (undefined === tileHeight) {
						console.error(`Tileset #${tilesetIndex} has no "tileHeight" in file ${sourceUrl}`);
						continue;
					}
					const tileRowCount : number = tileset["columns"]; // How many tiles per row.
					if (undefined === tileRowCount) {
						console.error(`Tileset #${tilesetIndex} has no "columns" in file ${sourceUrl}`);
						continue;
					}
					const totalTileCount : number = tileset["tilecount"]; // How many tiles in total.
					if (undefined === totalTileCount) {
						console.error(`Tileset #${tilesetIndex} has no "tilecount" in file ${sourceUrl}`);
						continue;
					}
					const tileColumnCount : number = totalTileCount / tileRowCount;
					const imageUrl : string = tileset["image"];
					if (undefined === imageUrl) {
						console.error(`Tileset #${tilesetIndex} has no "image" in file ${sourceUrl}`);
						continue;
					}
					const idOffset : number = tileset["firstgid"];
					if (undefined === idOffset) {
						console.error(`Tileset #${tilesetIndex} has no "firstgid" in file ${sourceUrl}`);
						continue;
					}
					const tiles : any[] = tileset["tiles"];
					if (undefined === tiles) {
						console.error(`Tileset #${tilesetIndex} has no "tiles" in file ${sourceUrl}`);
						continue;
					}
					for (let tileIndex = 0;tileIndex < tiles.length;tileIndex += 1) {
						const tile : any = tiles[tileIndex];
						const id : number = idOffset + tile["id"];
						const tileInfo = new _PartialTileInfo();
						tileInfo.url = imageUrl;
						tileInfo.x = (tileIndex % tileRowCount) * tileWidth;
						tileInfo.y = (tileColumnCount - Math.floor(tileIndex / tileRowCount) - 1) * tileHeight;
						tileInfo.width = tileWidth;
						tileInfo.height = tileHeight;
						// Add the boolean property information.
						const properties : any[] = tile["properties"];
						if (properties) {
							for (let property of properties) {
								if ("bool" === property["type"]) {
									const name = property["name"];
									if (undefined === name) { continue; }
									const value = property["value"];
									if (undefined === value) { continue; }
									tileInfo.properties.push(new _BooleanProperty(name, value));
								}
							}
						}
						// Then get the collision information.
						const collisionObjects : any[] = tile?.objectgroup?.objects;
						if (collisionObjects) {
							for (let collision of collisionObjects) {
								const type = collision["type"];
								if (undefined === type) { continue; }
								const x = collision["x"];
								if (undefined === x) { continue; }
								const y = collision["y"];
								if (undefined === y) { continue; }
								const width = collision["width"];
								if (undefined === width) { continue; }
								const height = collision["height"];
								if (undefined === height) { continue; }
								const rectangle = new _CollisionRect();
								rectangle.type = type;
								rectangle.x1 = x;
								rectangle.y1 = tileInfo.height - y;
								rectangle.x2 = x + width;
								rectangle.y2 = tileInfo.height - (y + height);
								tileInfo.collisionRectangles.push(rectangle);
							}
						}
						tileIdToInfo.set(id, tileInfo);
						maxId = Math.max(maxId, id);
					}
				}
				// Then add all the tiles.
				for (let tileId = 0;tileId <= maxId;tileId += 1) {
					let info = tileIdToInfo.get(tileId);
					if (!info) {
						info = new _PartialTileInfo(); // Resort to the defaults.
					}
					this._addTile(sourceUrl, info.url, info.x, info.y, info.width, info.height);
					for (let property of info.properties) {
						if (property instanceof _BooleanProperty) {
							this._addTileBooleanProperty(sourceUrl, property.name, property.value);
						}
					}
					for (let rectangle of info.collisionRectangles) {
						this._addTileCollisionRectangle(
							sourceUrl,
							rectangle.type,
							rectangle.x1,
							rectangle.y1,
							rectangle.x2,
							rectangle.y2,
						);
					}
				}
				// Then add all the layers.
				const layers : any[] = json["layers"];
				for (let layerIndex = 0;layerIndex < layers.length;layerIndex += 1) {
					const layer : any = layers[layerIndex];
					if ("tilelayer" === layer["type"]) {
						// Handle the tile layers.
						let name : string = layer["name"];
						if (!name) { name = ""; }
						let xOffset : number = layer["offsetx"];
						if (undefined === xOffset) { xOffset = 0; }
						let yOffset : number = layer["offsety"];
						if (undefined === yOffset) { yOffset = 0; }
						const width : number = layer["width"];
						if (undefined === width) {
							console.error(`Layer ${name} (index=${layerIndex}) has no "width" in file ${sourceUrl}`);
							continue;
						}
						const height : number = layer["height"];
						if (undefined === height) {
							console.error(`Layer ${name} (index=${layerIndex}) has no "height" in file ${sourceUrl}`);
							continue;
						}
						const data : number[] = layer["data"];
						if (undefined === data) {
							console.error(`Layer ${name} (index=${layerIndex}) has no "data" in file ${sourceUrl}`);
							continue;
						}
						// Find the resulting total dimensions.
						let maxTileWidth = 0;
						let maxTileHeight = 0;
						for (let tileId of data) {
							if (!tileIdToInfo.has(tileId)) { continue; }
							const info = tileIdToInfo.get(tileId);
							maxTileWidth  = Math.max(maxTileWidth,  info.width);
							maxTileHeight = Math.max(maxTileHeight, info.height);
						}
						const pixelWidth  = width  * maxTileWidth;
						const pixelHeight = height * maxTileHeight;
						// Then store it.
						this._addTileLayer(sourceUrl, name, xOffset, yOffset, width, height, pixelWidth, pixelHeight, new Uint32Array(data));
					} else if ("objectgroup" === layer["type"]) {
						// Handle the geometry layers.
						// Mostly just extract a few useful bits of information.
						const objects = layer["objects"];
						for (let objectIndex = 0;objectIndex < objects.length;objectIndex += 1) {
							const object = objects[objectIndex];
							if (true === object["point"]) {
								const name : string = object["name"];
								if (undefined === name) {
									console.error(`Object #${objectIndex} in layer #{layerIndex} has no "name" in file ${sourceUrl}`);
									continue;
								}
								const x : number = object["x"];
								if (undefined === x) {
									console.error(`Object #${objectIndex} in layer #{layerIndex} has no "x" in file ${sourceUrl}`);
									continue;
								}
								const y : number = object["y"];
								if (undefined === y) {
									console.error(`Object #${objectIndex} in layer #{layerIndex} has no "y" in file ${sourceUrl}`);
									continue;
								}
								this._addPoint(sourceUrl, name, x, y);
							} else {
								console.warn(`Object #${objectIndex} in layer #{layerIndex} has an recognized type  in file ${sourceUrl}`);
							}
						}
					} else {
						console.warn(`Unsure of how to deal with Tiled layer type ${layer["type"]} in file ${sourceUrl}`);
					}
				}
				this._onDone(sourceUrl);
			}.bind(this)).catch(
				(error) => console.error(`Failed loading ${sourceUrl} due to:`, error)
			);
		}
	}
}
