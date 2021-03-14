# Robot Explorer (working title) #

A game about surviving when you're an fragile robot in a collapsed world.  Explore, gain abilities, and figure out why your creators are dead and gone.

## How To Build ##

You must have:

* Python 3.6+. I test on `3.8.2`.
* TypeScript. The version I'm currently using is `3.8.3`, though you can probably go much lower. Must be able to run `tsc` through CLI.
* Rust and Cargo. I have `rustc` and `cargo` at `1.47.0`. Need at least version `1.32.0` otherwise `u16::to_be_bytes()` won't work.
* Install `wasm-pack` via Rust's `cargo` package manager. I've currently got `0.9.1`.

Then just run `python make.py` from the root directory. That should build everything and start a server that hosts on a port on localhost. The exact port will be printed to terminal. The server will stay up until you hit **Control + C**.

Once everything is build, can also run just the server via `python server.py`.
