"""
A simple HTTP server that serves on localhost.
"""

import http.server
import textwrap
import functools

import config

class RequestHandler(http.server.SimpleHTTPRequestHandler):
	"""Adds WASM MIME handling as that seems to be required in some browsers."""
	extensions_map = dict(http.server.SimpleHTTPRequestHandler.extensions_map) # Copy so don't accidentally update the dictionary in parent class.
	extensions_map[".wasm"] = "application/wasm";

def run_server():
	"""Runs the server until the user hits Ctrl+C."""
	server = http.server.HTTPServer(("", config.PORT), functools.partial(RequestHandler, directory=config.SITE_DIRECTORY))
	print(textwrap.dedent(f"""\
		Starting server, with pages at:
			* Main page: http://localhost:{config.PORT}/
			* Tests: http://localhost:{config.PORT}/tests.html
		Hit Ctrl + C to stop.
"""))
	try:
		server.serve_forever()
	except KeyboardInterrupt:
		pass

if "__main__" == __name__:
	run_server()
