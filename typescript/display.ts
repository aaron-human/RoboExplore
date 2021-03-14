namespace ExampleProject {
	/// The rendering types for display buffers.
	export enum DisplayBufferType {
		SOLID = 0,
		LINES = 1,
	}

	/**
	 * A class for storing a single object's display info.
	 * Properties are public as this is an internal class.
	 */
	class _DisplayBuffer {
		/// The vertex buffer.
		public readonly vertices : WebGLBuffer;
		/// The vertex color buffer.
		public readonly colors : WebGLBuffer;
		/// The index buffer.
		public readonly indices : WebGLBuffer;
		/// The number of values in the indices buffer.
		public count : number = 0;
		/// The transform matrix. Should have 16 elements in OpenGL's (confusing) internal memory layout.
		public transform : Float32Array;
		/// The rendering type used by WebGL.
		private _glType : number;
		/// Whether the buffer should be drawn.
		public visible : boolean = true;

		/// Creates an instance.
		constructor(context : WebGL2RenderingContext, type : DisplayBufferType) {
			this.vertices = context.createBuffer();
			this.colors = context.createBuffer();
			this.indices = context.createBuffer();
			this.transform = new Float32Array([
				1, 0, 0, 0,
				0, 1, 0, 0,
				0, 0, 1, 0,
				0, 0, 0, 1, // This last row is the "constant row" that's multiplied by w and added!
			]);
			this.setType(context, type);
		}

		/// Sets the DisplayBufferType. Doesn't do anything else, as this isn't the part of the system that handles updating buffer internals.
		public setType(context : WebGL2RenderingContext, type : DisplayBufferType) {
			switch (type) {
				case DisplayBufferType.SOLID: this._glType = context.TRIANGLES; break;
				case DisplayBufferType.LINES: this._glType = context.LINES; break;
			}
		}

		/// The WebGL type.
		get glType() : number {
			return this._glType;
		}
	}

	/// The type for a resize callback.
	export type DisplayResizeCallback = (width : number, height : number) => void;

	/**
	 * A class for managing the canvas/visuals.
	 */
	export class Display {
		/// The canvas to use.
		private readonly _canvas : HTMLCanvasElement;
		/// The context to use.
		private readonly _context : WebGL2RenderingContext;
		/// The div element to store overlayed text through.
		public readonly text : TextDisplay;

		/// The source for the vertex shader.
		private readonly _vertexShaderSource = `\
			#version 300 es

			uniform mat4 perspective;
			uniform mat4 transform;

			in vec3 position;
			in vec4 color;

			out vec4 color_interpolation;

			void main() {
				color_interpolation = color / 255.0;
				gl_Position = perspective * transform * vec4(position, 1.0); // NOTE: GLSL normalizes based on the w term!
			}
		`;
		/// The source for the fragment shader.
		private readonly _fragmentShaderSource = `\
			#version 300 es
			precision highp float;

			in vec4 color_interpolation;

			out vec4 color;

			void main() {
				color = color_interpolation;
			}
		`;
		/// The position of the vertex buffer.
		private _vertexBufferPosition : number;
		/// The vertex color position buffer.
		private _colorBufferPosition : number;
		/// The position of the transform matrix.
		private _transformPosition : WebGLUniformLocation;
		/// The position of the perspective matrix.
		private _perspectivePosition : WebGLUniformLocation;

		/// The next ID to give a new DisplayBuffer.
		private _nextBufferID = 1;
		/// All of the buffers mapped from their ids. Deleted buffers will be removed.
		private readonly _buffers : Map<number,_DisplayBuffer> = new Map();
		/// All deleted buffers that can be "recycled".
		private readonly _deleted : _DisplayBuffer[] = [];
		/// All of the buffers to draw, in the order they should be drawn.
		private readonly _drawOrder : _DisplayBuffer[] = [];

		/// A function to be called whenever the canvas resizes.
		private _resizeCallback : DisplayResizeCallback = null;
		set resizeCallback(func : DisplayResizeCallback) {
			this._resizeCallback = func;
			this._onResize();
		}

		/// Creates a shader and attaches it to the given program.
		private _setupShader(program : WebGLProgram, type : number, source : string) {
			const ctx = this._context;
			const shader = this._context.createShader(type);
			ctx.shaderSource(shader, source);
			ctx.compileShader(shader);
			if (!ctx.getShaderParameter(shader, ctx.COMPILE_STATUS)) {
				const name = (this._context.VERTEX_SHADER === type) ? "vertex shader" : "fragment shader";
				const message = `Couldn't compile ${name} due to:\n${ctx.getShaderInfoLog(shader)}`;
				ctx.deleteShader(shader);
				throw message;
			}
			ctx.attachShader(program, shader);
		}

		/// Creates and starts the game.
		constructor() {
			// Get the canvas and context.
			this._canvas = document.getElementsByTagName("canvas")[0];
			const ctx = this._context = this._canvas.getContext("webgl2");
			this.text = new TextDisplay(this._canvas.parentElement);

			// Handle resizing.
			window.addEventListener("resize", this._onResize.bind(this));
			this._onResize(); // Get the size handling setup immediately too.

			// Setup the program and shaders.
			const program = this._context.createProgram();
			this._setupShader(program, this._context.VERTEX_SHADER, this._vertexShaderSource);
			this._setupShader(program, this._context.FRAGMENT_SHADER, this._fragmentShaderSource);
			ctx.linkProgram(program);
			if (!ctx.getProgramParameter(program, ctx.LINK_STATUS)) {
				const message = `Couldn't link together program due to:\n${ctx.getProgramInfoLog(program)}`;
				ctx.deleteProgram(program); // I assume this cleans up the attached shaders too?
				throw message;
			}
			ctx.useProgram(program);

			// Setup the shader inputs.
			this._vertexBufferPosition = ctx.getAttribLocation(program, "position");
			this._colorBufferPosition = ctx.getAttribLocation(program, "color");
			this._transformPosition = ctx.getUniformLocation(program, "transform");
			this._perspectivePosition = ctx.getUniformLocation(program, "perspective");

			/// Always start with a unit perspective.
			this.perspectiveTransform = new Float32Array([
				1.0, 0.0, 0.0, 0.0,
				0.0, 1.0, 0.0, 0.0,
				0.0, 0.0, 1.0, 0.0,
				0.0, 0.0, 0.0, 1.0,
			]);

			// Setup some rendering settings.
			ctx.clearColor(0.0, 0.0, 0.0, 1.0); // Clearing the display always sets it to solid black.
			ctx.clearDepth(1.0); // Clearing the display will set the depth buffer to a uniform 1.0.
			ctx.enable(ctx.DEPTH_TEST); // Do hide geometry using the depth buffer.
			ctx.depthFunc(ctx.LEQUAL); // Geometry with a lower depth value will be drawn "on top".
		}

		/// The canvas that the Display is using.
		get canvas() : HTMLCanvasElement { return this._canvas; }

		/**
		 * Resizes the canvas' internals.
		 */
		private _onResize() {
			// Bare minimum required for the context to redraw without stretching.
			this._canvas.width = this._canvas.clientWidth;
			this._canvas.height = this._canvas.clientHeight;
			this._context.viewport(0, 0, this._canvas.width, this._canvas.height);
			if (this._resizeCallback) {
				this._resizeCallback(this._canvas.width, this._canvas.height);
			}
		}

		/// Creates a display buffer and returns its handle.
		public createBuffer(type : DisplayBufferType) : number {
			const id = this._nextBufferID;
			this._nextBufferID += 1;
			let buffer;
			let doReset = false;
			if (0 < this._deleted.length) {
				buffer = this._deleted.shift();
				doReset = true;
				buffer.setType(type);
			} else {
				buffer = new _DisplayBuffer(this._context, type);
			}
			this._buffers.set(id, buffer);
			this._drawOrder.push(buffer);
			if (doReset) {
				this.setBuffer(id, new Float32Array([]), new Uint8Array([]), new Uint16Array([]));
				this.setBufferTransform(id, new Float32Array([
					1.0, 0.0, 0.0, 0.0,
					0.0, 1.0, 0.0, 0.0,
					0.0, 0.0, 1.0, 0.0,
					0.0, 0.0, 0.0, 1.0,
				]));
			}
			console.log(`Display buffer count: ${this._drawOrder.length + this._deleted.length}`);
			return id;
		}

		/// Removes a draw buffer from use. It will eventually be "recycled" by a createBufffer() call later.
		public deleteBuffer(id : number) {
			if (!this._buffers.has(id)) { return false; }
			const buffer = this._buffers.get(id);
			this._buffers.delete(id);
			this._drawOrder.splice(this._drawOrder.indexOf(buffer), 1);
			this._deleted.push(buffer);
		}

		/// Sets the contents of a display buffer.
		public setBuffer(id : number, vertices : Float32Array, colors : Uint8Array, indices : Uint16Array) : boolean {
			if (!this._buffers.has(id)) { return false; }
			const buffer = this._buffers.get(id);
			const ctx = this._context;
			ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.vertices);
			ctx.bufferData(ctx.ARRAY_BUFFER, vertices, ctx.STATIC_DRAW); // Maybe make this "dynamic draw" someday?
			ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.colors);
			ctx.bufferData(ctx.ARRAY_BUFFER, colors, ctx.STATIC_DRAW); // Maybe make this "dynamic draw" someday?
			ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, buffer.indices);
			ctx.bufferData(ctx.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), ctx.STATIC_DRAW); // Maybe make this "dynamic draw" someday?
			buffer.count = indices.length;
			return true;
		}

		/// Sets the transform on a buffer.
		public setBufferTransform(id : number, matrix : Float32Array) : boolean {
			if (!this._buffers.has(id)) { return false; }
			const buffer = this._buffers.get(id);
			buffer.transform = matrix;
			return true;
		}

		/// Sets whether a display buffer is visible.
		public setDisplayBufferVisibility(id : number, visible : boolean) {
			const buffer = this._buffers.get(id);
			buffer.visible = visible;
		}

		/// The overall perspective transform.
		set perspectiveTransform(matrix : Float32Array) {
			this._context.uniformMatrix4fv(
				this._perspectivePosition,
				false,
				matrix,
			);
		}

		/// Draws to the canvas.
		public draw() {
			const ctx = this._context;
			// Assuming the program and buffers haven't changed, this is pretty trivial.

			ctx.clear(ctx.COLOR_BUFFER_BIT | ctx.DEPTH_BUFFER_BIT); // Clear the color and depth of the display.

			for (let buffer of this._drawOrder) {
				if (!buffer.visible) { continue; }
				ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.vertices);
				ctx.vertexAttribPointer(
					this._vertexBufferPosition,
					3, // There are 3 floats per vertex.
					ctx.FLOAT, // Using a Float32Array() to pass the buffer's data.
					false, // Whether to normalize input values to some specified range. (Not needed here.)
					0, // Don't skip any consecutive bytes when reading the buffer.
					0, // Don't start at some byte offset when reading the buffer.
				);
				ctx.enableVertexAttribArray(this._vertexBufferPosition);

				ctx.bindBuffer(ctx.ARRAY_BUFFER, buffer.colors);
				ctx.vertexAttribPointer(
					this._colorBufferPosition,
					4, // There are 4 bytes per vertex (RGBA).
					ctx.UNSIGNED_BYTE, // Using a Uint8Array() to pass the buffer's data.
					false, // Whether to normalize input values to some specified range. (Not needed here.)
					0, // Don't skip any consecutive bytes when reading the buffer.
					0, // Don't start at some byte offset when reading the buffer.
				);
				ctx.enableVertexAttribArray(this._colorBufferPosition);

				ctx.uniformMatrix4fv(
					this._transformPosition,
					false,
					buffer.transform,
				);

				ctx.bindBuffer(ctx.ELEMENT_ARRAY_BUFFER, buffer.indices);
				// Then draw everything.
				ctx.drawElements(
					buffer.glType,
					buffer.count, // Number of vertices in the index buffer.
					ctx.UNSIGNED_SHORT, // The type used to encode the index buffer in bufferData().
					0, // Byte offset into the index buffer to start at.
				);
			}
		}
	}
}
