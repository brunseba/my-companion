// commit-and-tag-version updater for src-tauri/Cargo.lock's *own* package
// entry. Cargo.lock has one `version = "..."` line per dependency, but only
// the one directly under `name = "my-companion"` is ours to touch - Cargo
// owns every other crate's lock entry.
const OWN_PACKAGE_VERSION = /name = "my-companion"\nversion = "([^"]+)"/;

module.exports.readVersion = function readVersion(contents) {
  const match = contents.match(OWN_PACKAGE_VERSION);
  return match ? match[1] : null;
};

module.exports.writeVersion = function writeVersion(contents, version) {
  return contents.replace(OWN_PACKAGE_VERSION, `name = "my-companion"\nversion = "${version}"`);
};
