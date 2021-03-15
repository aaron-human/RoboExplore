namespace ExampleProject {
	type AddTileFunc = (imageUrl : string, x : number, y : number, width : number, height : number) => void;
	type OnDoneFunc = (url : string) => void;
	/**
	 * A class for loading in JSON exports from Tiled.
	 */
	export class TiledFileLoader {
		private _addTile : AddTileFunc = null;
		private _onDone : OnDoneFunc = null;
		/// Stores callbacks useful for loading tile info.
		public setup(addTile : AddTileFunc, onDone : OnDoneFunc) {
			this._addTile = addTile;
			this._onDone = onDone;
		}

		/// Starts loading the file at the given URL.
		public startLoading(url : string) {
			let sourceUrl = url;
			fetch(url).then(
				(response) => response.json()
			).then(function(json : any){
				this._addTile("", 0, 0, 1, 1);
				this._onDone(sourceUrl);
			}.bind(this)).catch(
				(error) => console.error(`Failed loading ${sourceUrl} due to:`, error)
			);
		}
	}
}
