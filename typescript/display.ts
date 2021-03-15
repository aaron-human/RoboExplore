namespace ExampleProject {
	/// The rendering types for display buffers.
	export enum DisplayBufferType {
		SOLIDS = 0, // for filled shapes (composed of triangles).
		LINES = 1, // for lines (composed of individual lines)
		IMAGES = 2, // for images (composed of triangles).
	}

	/**
	 * A class for storing a single object's display info.
	 * Properties are public as this is an internal class.
	 */
	class _DisplayBuffer {
		/// The vertex buffer.
		public readonly vertices : WebGLBuffer;
		/// The vertex color buffer for SOLID and LINES buffers. Stores texture coordinates in IMAGE buffers.
		public readonly colors : WebGLBuffer;
		/// The texture to use.
		public texture : number = null;
		/// The index buffer.
		public readonly indices : WebGLBuffer;
		/// The number of values in the indices buffer.
		public count : number = 0;
		/// The transform matrix. Should have 16 elements in OpenGL's (confusing) internal memory layout.
		public transform : Float32Array;
		/// The rendering type used by WebGL.
		private _glType : number;
		/// Whether to use the texture.
		private _useTexture : boolean;
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
				case DisplayBufferType.SOLIDS:
					this._glType = context.TRIANGLES;
					this._useTexture = false;
					break;
				case DisplayBufferType.LINES:
					this._glType = context.LINES;
					this._useTexture = false;
					break;
				case DisplayBufferType.IMAGES:
					this._glType = context.TRIANGLES;
					this._useTexture = true;
					break;
			}
		}

		/// The WebGL type.
		get glType() : number {
			return this._glType;
		}

		/// Whether to use the linked texture.
		get useTexture() : boolean {
			return this._useTexture;
		}
	}

	/**
	 * A class for WebGL textures.
	 */
	export class _DisplayTexture {
		/// The WebGL texture.
		public readonly texture : WebGLTexture;

		/// The width in pixels.
		public width : number = 1;
		/// The height in pixels.
		public height : number = 1;

		/// Creates a one pixel red texture.
		constructor(context : WebGL2RenderingContext) {
			this.texture = context.createTexture();
			context.bindTexture(context.TEXTURE_2D, this.texture);
			const data = new Uint8Array([255, 0, 0, 255]);
			context.texImage2D(
				context.TEXTURE_2D,
				0, // No mipmaps
				context.RGBA, // Store RGBA in WebGL.
				1, 1, // Width x height
				0, // border?
				context.RGBA, // The passed in data is RGBA too.
				context.UNSIGNED_BYTE, // Each passed in component channel is one unsigned byte.
				data, // The single pixel of data.
			);

			// Values outside of [0.0, 1.0] clamps down to those edge values.
			context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_S, context.CLAMP_TO_EDGE);
			context.texParameteri(context.TEXTURE_2D, context.TEXTURE_WRAP_T, context.CLAMP_TO_EDGE);
			// Going for a pixel-art style so no interpolation.
			context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MAG_FILTER, context.NEAREST);
			context.texParameteri(context.TEXTURE_2D, context.TEXTURE_MIN_FILTER, context.NEAREST);
		}

		/// Updates the texture to an image.
		public setImage(context : WebGL2RenderingContext, image : HTMLImageElement) {
			context.bindTexture(context.TEXTURE_2D, this.texture);
			context.texImage2D(
				context.TEXTURE_2D,
				0, // No mipmaps
				context.RGBA, // Store RGBA in WebGL.
				context.RGBA, // The passed in data is RGBA too.
				context.UNSIGNED_BYTE, // Each passed in component channel is one unsigned byte.
				image,
			);
			this.width = image.naturalWidth;
			this.height = image.naturalHeight;
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

			out vec4 color_source;

			void main() {
				color_source = color;
				gl_Position = perspective * transform * vec4(position, 1.0); // NOTE: GLSL normalizes based on the w term!
			}
		`;
		/// The source for the fragment shader.
		private readonly _fragmentShaderSource = `\
			#version 300 es
			precision highp float;

			uniform float use_texture;
			uniform sampler2D texture_sampler;
			uniform vec2 texture_size;

			in vec4 color_source;

			out vec4 color;

			void main() {
				vec2 texture_position = color_source.xy / texture_size;
				texture_position.y = 1.0 - texture_position.y; // WebGL does texture position in cartesian coords.
				color = mix(
					color_source / 255.0,
					texture(texture_sampler, texture_position),
					use_texture
				);
				/*
				if (color.w < 1e-6) {
					discard; // Not used so can have alpha channel blending instead.
				}
				*/
			}
		`;
		/// The position of the vertex buffer.
		private _vertexBufferPosition : number;
		/// The vertex color position buffer.
		private _colorBufferPosition : number;
		/// The position of the transform matrix.
		private readonly _transformPosition : WebGLUniformLocation;
		/// The position of the perspective matrix.
		private readonly _perspectivePosition : WebGLUniformLocation;
		/// The position of the 'use_texture' numerical switch.
		private readonly _useTexturePosition : WebGLUniformLocation;
		/// The position of the texture sampler.
		private readonly _texturePosition : WebGLUniformLocation;
		/// The size of the texture.
		private readonly _textureSizePosition : WebGLUniformLocation;

		/// A default texture to use.
		private readonly _defaultTexture : _DisplayTexture;

		/// The next ID to give a new _DisplayBuffer.
		private _nextBufferID = 1;
		/// All of the buffers mapped from their ids. Deleted buffers will be removed.
		private readonly _buffers : Map<number,_DisplayBuffer> = new Map();
		/// All deleted buffers that can be "recycled".
		private readonly _deleted : _DisplayBuffer[] = [];
		/// All of the buffers to draw, in the order they should be drawn.
		private readonly _drawOrder : _DisplayBuffer[] = [];

		/// The next ID to give a new _DisplayTexture.
		private _nextTextureID = 1;
		/// All of the textures mapped from their ids. Deleted textures will be removed.
		private readonly _textures : Map<number, _DisplayTexture> = new Map();

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
			this._useTexturePosition = ctx.getUniformLocation(program, "use_texture");
			this._texturePosition = ctx.getUniformLocation(program, "texture_sampler");
			this._textureSizePosition = ctx.getUniformLocation(program, "texture_size");

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
			ctx.enable(ctx.BLEND); // Let blending handle the alpha channel.
			ctx.blendFunc(ctx.SRC_ALPHA, ctx.ONE_MINUS_SRC_ALPHA);

			// Always just use texture 0. I only need one texture per render (so far).
			ctx.activeTexture(ctx.TEXTURE0);
			ctx.uniform1i(this._texturePosition, 0);
			this._defaultTexture = new _DisplayTexture(ctx);
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
		public deleteBuffer(id : number) : boolean {
			if (!this._buffers.has(id)) { return false; }
			const buffer = this._buffers.get(id);
			this._buffers.delete(id);
			this._drawOrder.splice(this._drawOrder.indexOf(buffer), 1);
			this._deleted.push(buffer);
			return true;
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
		public setBufferVisibility(id : number, visible : boolean) : boolean {
			if (!this._buffers.has(id)) { return false; }
			const buffer = this._buffers.get(id);
			buffer.visible = visible;
			return true;
		}

		/// The overall perspective transform.
		set perspectiveTransform(matrix : Float32Array) {
			this._context.uniformMatrix4fv(
				this._perspectivePosition,
				false,
				matrix,
			);
		}

		/// Creates a texture and returns its handle.
		///
		/// Note that this doesn't require a specific URL source to leave room in case textures need to be generated procedurally in the future.
		public createTexture() : number {
			const id = this._nextTextureID;
			this._nextTextureID += 1;
			this._textures.set(id, new _DisplayTexture(this._context));
			return id;
		}

		/// Deletes a texture.
		public deleteTexture(id : number) : boolean {
			if (!this._textures.has(id)) { return false; }
			const texture = this._textures.get(id);
			this._textures.delete(id);
			this._context.deleteTexture(texture.texture);
			return true;
		}

		/// Sets a texture to the result of loading a given external image.
		public setTextureWithURL(id : number, url : string) : boolean {
			if (!this._textures.has(id)) { return false; }
			const texture = this._textures.get(id);
			// TODO? A way to track when specific textures load so can have a nice loading screen.
			const image = new Image();
			image.addEventListener("load", function(){
				texture.setImage(this._context, image);
				console.log(`Texture ${id} loaded: ${url}`);
				console.log(`Texture ${id} has size: ${texture.width} x ${texture.height}`);
				console.log(`Texture ${id}:`, image);
			}.bind(this));
			console.log(`Starting to loading image into texture ${id}: ${url}`);
			image.src = url;
			return true;
		}

		/// Sets the display buffer's texture.
		public setBufferTexture(bufferId : number, textureId : number) {
			if (!this._buffers.has(bufferId)) { return false; }
			if (!this._textures.has(textureId)) { return false; }
			const buffer = this._buffers.get(bufferId);
			buffer.texture = textureId;
			return true;
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
					(buffer.useTexture) ? (2) : (4), // Textures are packed as 2 16-bit values. Colors are 4 8-bit values.
					(buffer.useTexture) ? (ctx.UNSIGNED_SHORT) : (ctx.UNSIGNED_BYTE), // Using a Uint8Array() or a Uint16Array() to pass the buffer's data.
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

				// Load in the texture information.
				ctx.uniform1f(
					this._useTexturePosition,
					(buffer.useTexture) ? (1.0) : (0.0),
				)
				let texture = this._defaultTexture;
				if (null !== buffer.texture && this._textures.has(buffer.texture)) {
					texture = this._textures.get(buffer.texture);
				}
				ctx.bindTexture(ctx.TEXTURE_2D, texture.texture);
				ctx.uniform2f(
					this._textureSizePosition,
					texture.width,
					texture.height,
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
