// commit-and-tag-version updater for src-tauri/Cargo.toml - the `version = "..."`
// line under `[package]` (the very first such line in the file; Cargo.lock has
// many "version" lines for dependencies, but Cargo.toml only has our own).
const VERSION_LINE = /^version\s*=\s*"[^"]+"/m;

module.exports.readVersion = function readVersion(contents) {
  const match = contents.match(VERSION_LINE);
  return match ? match[0].match(/"([^"]+)"/)[1] : null;
};

module.exports.writeVersion = function writeVersion(contents, version) {
  return contents.replace(VERSION_LINE, `version = "${version}"`);
};
