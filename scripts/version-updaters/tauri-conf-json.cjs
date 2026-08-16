// commit-and-tag-version updater for src-tauri/tauri.conf.json's top-level
// "version" field. Re-serializes with 2-space indent (matching the scaffold's
// formatting) and a trailing newline.
module.exports.readVersion = function readVersion(contents) {
  return JSON.parse(contents).version;
};

module.exports.writeVersion = function writeVersion(contents, version) {
  const json = JSON.parse(contents);
  json.version = version;
  return `${JSON.stringify(json, null, 2)}\n`;
};
