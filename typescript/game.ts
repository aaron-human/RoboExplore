namespace ExampleProject {
	/**
	 * The main game class.
	 */
	export class Game {
		/// The target number of milliseconds between consecutive game updates.
		private readonly _UPDATE_PERIOD = 33 // Gives about 30 frames per second.
		/// The last time the game updated.
		private _last_update_time : number = null;

		/// The object that handles drawing everything.
		private _display : Display;
		/// The object managing keyboard and mouse inputs.
		private _input : Input;

		/// Creates a minimal Game instance.
		constructor() {
			// Note: This is run before the document is fully loaded. So can't rely on it being available yet...

			// Try to keep the methods on this class bound to this instance.
			// Need this as unfortunately wasm-bindgen will call some of these methods with `this` set, which breaks things...
			for (let name of Object.getOwnPropertyNames(Game.prototype)) {
				const value = this[name];
				if ("function" === typeof value) {
					this[name] = value.bind(this);
					console.log(`Rebound GAME.${name}`);
				}
			}
		}

		/// Run once the document is setup. This is where the WASM is loaded in.
		public setup() {
			// Once GAME is set, try loading in the WASM.
			// The wasm_bindgen() method requires that GAME exists to link against it.
			wasm_bindgen("rust.wasm").then(function(wasm) {
				// The "wasm" argument is the raw WASM object. It has all the same methods as wasm_bindgen, except that they have no JS interfacing setup...
				// So use wasm_bindgen's "namespace" instead.
				this._start();
			}.bind(this));
		}

		/// Starts running the game now that the WASM has been loaded in.
		private _start() {
			console.log("Starting game...");
			// Create the objects ahead of fully setting them up (in case `wasm_bindgen.setup()` needs acces to these).
			this._display = new Display();
			this._input = new Input(this._display.canvas);

			wasm_bindgen.setup();

			// Must do this AFTER the above setup() function is run (as it causes the on_resize to be called).
			// Must also be set after this._display is setup.
			this._display.resizeCallback = wasm_bindgen.on_resize;
			this._input.setup(
				wasm_bindgen.on_key_down,
				wasm_bindgen.on_key_up,
				wasm_bindgen.on_mouse_enter,
				wasm_bindgen.on_mouse_update,
				wasm_bindgen.on_mouse_leave,
			);

			setInterval(this._update.bind(this), this._UPDATE_PERIOD);
		}

		/// An example exported method.
		public exportExample(value : number) {
			console.log(`WASM requested that this print ${value}`);
		}

		/**
		 * The periodic update function that makes the game run.
		 */
		private _update() {
			const now = Date.now() / 1000.0;
			const elapsed_seconds = (null !== this._last_update_time) ? (now - this._last_update_time) : (0);
			this._last_update_time = now;

			// Get the WASM to update
			wasm_bindgen.update(elapsed_seconds);

			requestAnimationFrame(this._draw.bind(this));
		}

		/// Creates a buffer and returns its ID.
		public createDrawBuffer(type_ : DisplayBufferType) : number {
			return this._display.createBuffer(type_);
		}

		/// Marks a draw buffer's ID as used up.
		public deleteDrawBuffer(id : number) {
			return this._display.deleteBuffer(id);
		}

		/// Sets the contents of a display buffer.
		public setDisplayBuffer(id : number, vertices : Float32Array, colors : Uint8Array, indices : Uint16Array) : boolean {
			return this._display.setBuffer(id, vertices, colors, indices);
		}

		/// Sets the transform on a display buffer.
		public setDisplayBufferTransform(id : number, matrix : Float32Array) : boolean {
			return this._display.setBufferTransform(id, matrix);
		}

		/// Sets the overall transform for the display.
		public setDisplayTransform(matrix : Float32Array) {
			this._display.perspectiveTransform = matrix;
		}

		/// Sets whether a display buffer is visible.
		public setDisplayBufferVisibility(id : number, visible : boolean) {
			this._display.setDisplayBufferVisibility(id, visible);
		}

		/// The text management object.
		get text() : TextDisplay {
			return this._display?.text;
		}

		/**
		 * Draws to the canvas.
		 */
		private _draw() {
			this._display.draw();
		}
	}
}
