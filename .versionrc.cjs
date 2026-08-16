// Config for `commit-and-tag-version` (npm run release).
// Keeps package.json, src-tauri/Cargo.toml, and src-tauri/tauri.conf.json all
// on the same semver, driven by Conventional Commits since the last tag.
module.exports = {
  packageFiles: [{ filename: "package.json", type: "json" }],
  bumpFiles: [
    { filename: "package.json", type: "json" },
    { filename: "package-lock.json", type: "json" },
    { filename: "src-tauri/Cargo.toml", updater: "scripts/version-updaters/cargo-toml.cjs" },
    { filename: "src-tauri/Cargo.lock", updater: "scripts/version-updaters/cargo-lock.cjs" },
    { filename: "src-tauri/tauri.conf.json", updater: "scripts/version-updaters/tauri-conf-json.cjs" },
  ],
};
