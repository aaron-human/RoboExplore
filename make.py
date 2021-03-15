"""
Builds everything.
"""

import time
import subprocess
import urllib.request
import shutil
from os.path import join, getmtime, exists
import os
import re
import textwrap

import config
import server

class CommandHung(Exception):
	"""For when a command hangs (no response for several seconds)."""
	def __init__(self, command, max_hang, run_time):
		super().__init__(f"Gave up on command {command!r} as it hung for {max_hang} seconds (after running for {run_time} seconds).")

class CommandTimeout(Exception):
	"""For when a command times out (takes too long overall)."""
	def __init__(self, command, max_time):
		super().__init__(f"Gave up on command {command!r} as it too longer than {max_time} seconds to complete.")

class WrongReturnCode(Exception):
	"""A class for when the return code is wrong."""
	def __init__(self, command, expected, actual):
		super().__init__(f"Command {command!r} got wrong return code! Expected {expected} but got {actual}.")

def call_cli(command, directory=None, max_hang=10.0, max_time=60.0, expected_return_code=0):
	"""
	Runs a function through CLI. Can use a different working directory and/or fail fast depending on return code.

	:param command: The command string to run.
	:param directory: The working directory to use when running the command.
	:param max_hang: The max number of seconds that the command can hang without outputting anything before this gives up on it.
	:param max_time: The max number of seconds that the process can take before this gives up on it.
	:param ignore_return_code: Whether to ignore the return code.
	"""
	start = time.time()
	process = subprocess.Popen(command, shell=True, cwd=directory, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)

	# Then handle waiting and giving up on this thread.
	now = time.time()
	absolute_end = now + max_time
	hang_end     = now + max_hang
	last_stdout_len, last_stderr_len = 0, 0
	printed_stdout_lines, printed_stderr_lines = 0, 0
	done = False
	try:
		while not done:
			# Wait a bit so don't absolutely occupy the processor.
			try:
				stdout,stderr = process.communicate(timeout=2.0)
			except subprocess.TimeoutExpired as timeout:
				stdout, stderr = timeout.stdout, timeout.stderr

				stdout_updated = stdout and len(stdout) > last_stdout_len
				stderr_updated = stderr and len(stderr) > last_stderr_len

				# Try to print out any new stdout/stderr lines.
				if stdout_updated:
					lines = stdout.decode("UTF-8").split("\n")
					for line in lines[printed_stdout_lines:-1]:
						print(f"STDOUT > {line}")
					printed_stdout_lines = len(lines)
					last_stdout_len = len(stdout)
				if stderr_updated:
					lines = stderr.decode("UTF-8").split("\n")
					for line in lines[printed_stderr_lines:-1]:
						print(f"STDERR > {line}")
					printed_stderr_lines = len(lines)
					last_stderr_len = len(stderr)

				# Check if timed out.
				now = time.time()
				if stdout_updated or stderr_updated:
					hang_end = now + max_hang
				elif now > hang_end:
					raise CommandHung(command, max_hang, now - start) from None
				if now > absolute_end:
					raise CommandTimeout(command, max_time) from None
			else:
				# Print any remaining output.
				lines = stdout.split("\n")
				for line in lines[printed_stdout_lines:-1]:
					print(f"STDOUT > {line}")
				lines = stderr.split("\n")
				for line in lines[printed_stderr_lines:-1]:
					print(f"STDERR > {line}")

				# Signal process is done
				done = True

		if expected_return_code is not None and process.returncode != expected_return_code:
			raise WrongReturnCode(command, expected_return_code, process.returncode)
	except:
		# Make errors look nice by putting them on a separate line.
		print("")
		raise
	finally:
		if not done:
			# If the process didn't complete and got here, then must've timed out so stop it.
			process.kill() # Or maybe terminate()?

def build_typescript(directory, output_file):
	"""
	Builds the TypeScript in a directory if the associated files are newer than the output_file.

	Returns if any files have changed.
	"""
	should_build = True
	if exists(output_file):
		output_file_changed_time = getmtime(output_file)
		directory_changed_times = []
		for (directory, _, files) in os.walk(directory):
			for file in files:
				directory_changed_times.append(getmtime(join(directory, file)))
		directory_changed_time = max(directory_changed_times)
		should_build = (directory_changed_time > output_file_changed_time)

	if should_build:
		call_cli("tsc --pretty", directory=directory)
	return should_build


def download(url, destination):
	"""Downloads a file from a URL to the given destination if the destination file doesn't exist."""
	if not exists(destination):
		print(f"Downloading {url} ...", end="")
		urllib.request.urlretrieve(url, destination)
		print(" Done!")

_FIX_EXPORT_KEYWORD = re.compile("^export (default )?", re.MULTILINE)
_FIX_RENAME_INIT_FUNCTION = re.compile(r"function\s+init\s*\(")
def fix_wasm_declaration_file(path):
	"""
	The wasm-pack handling of d.ts files is a big borked.

	This fixes it.
	"""
	with open(path, "r") as source:
		contents = source.read()

	# Split the file where the generic wasm-bindgen setup code starts.
	index = re.search(".*InitInput", contents, re.MULTILINE).start()
	before, after = contents[:index], contents[index:]
	# Indent everything, so it looks nice.
	before = textwrap.indent(before, "\t")
	# Put everything before in a "wasm_bindgen" namespace declaration.
	before = "declare namespace wasm_bindgen {\n" + before + "}\n"
	# Replace all "export" keywords in after with "declare".
	after = _FIX_EXPORT_KEYWORD.sub("declare ", after)
	# The `init` function isn't actually defined. It's stored as `wasm_bindgen`.
	after = _FIX_RENAME_INIT_FUNCTION.sub("function wasm_bindgen (", after, count=1)
	# Remove the extra spaces at the end to make things nice looking...
	after = after.rstrip()
	# And that's all.
	with open(path, "w") as output:
		output.write(before + after)




start = time.time()
print("▶ Running Rust tests...")
call_cli(
	"RUST_BACKTRACE=1 cargo test --color=always",
	directory=config.RUST_DIRECTORY,
	max_hang=15.0, # A little higher because need to compile several packages.
)
print(f"✓ Rust tests done in {time.time() - start:.3f}")





start = time.time()
print("▶ Compiling Rust to WASM + JS...")
call_cli(
	"wasm-pack build --target no-modules -- --color=always",
	directory=config.RUST_DIRECTORY,
)
print(f"✓ Done compiling Rust in {time.time() - start:.3f} seconds. Copying out results...")
shutil.copy(
	join(config.RUST_DIRECTORY, config.RUST_OUTPUT_SUBDIRECTORY, config.RUST_OUTPUT_D_TS_FILE),
	join(config.TYPESCRIPT_DIRECTORY, config.RUST_OUTPUT_D_TS_FILE),
)
fix_wasm_declaration_file(join(config.TYPESCRIPT_DIRECTORY, config.RUST_OUTPUT_D_TS_FILE))
shutil.copy(
	join(config.RUST_DIRECTORY, config.RUST_OUTPUT_SUBDIRECTORY, config.RUST_OUTPUT_JS_FILE),
	join(config.SITE_DIRECTORY, config.RUST_OUTPUT_JS_FILE),
)
shutil.copy(
	join(config.RUST_DIRECTORY, config.RUST_OUTPUT_SUBDIRECTORY, config.RUST_OUTPUT_WASM_FILE),
	join(config.SITE_DIRECTORY, config.RUST_FINAL_WASM_FILE),
)
print(f"✓ Copying completed.")





start = time.time()
print("▶ Compiling TypeScript...")
if build_typescript(config.TYPESCRIPT_DIRECTORY, join(config.SITE_DIRECTORY, config.TYPESCRIPT_OUTPUT_FILE)):
	print(f"✓ Done compiling TypeScript in {time.time() - start:.3f} seconds.")

	# If this changed, then copy the *.d.ts file into the tests directory for it.
	definitions_file = config.TYPESCRIPT_OUTPUT_FILE[:-2] + "d.ts"
	shutil.move(
		join(config.SITE_DIRECTORY, definitions_file),
		join(config.TEST_DIRECTORY, definitions_file),
	)
else:
	print(f"✓ TypeScript unchanged. No need to compile.")





start = time.time()
print("▶ Checking QUnit test files...")
download("https://code.jquery.com/qunit/qunit-2.11.3.css", join(config.SITE_DIRECTORY, "qunit-2.11.3.css"))
download("https://code.jquery.com/qunit/qunit-2.11.3.js",  join(config.SITE_DIRECTORY, "qunit-2.11.3.js"))
download("https://raw.githubusercontent.com/DefinitelyTyped/DefinitelyTyped/master/types/qunit/index.d.ts",  join(config.TEST_DIRECTORY, "qunit.d.ts"))
print(f"✓ QUnit test files ready after {time.time() - start:.3f} seconds.")





start = time.time()
print("▶ Compiling TypeScript tests...")
if build_typescript(config.TEST_DIRECTORY, join(config.SITE_DIRECTORY, config.TYPESCRIPT_TEST_OUTPUT_FILE)):
	print(f"✓ Done compiling TypeScript tests in {time.time() - start:.3f} seconds.")
else:
	print(f"✓ TypeScript tests unchanged. No need to compile.")







server.run_server()
