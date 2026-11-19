## [Unreleased] — fix/clippy-double-must-use-1.99

### Fixed
- Bump `async-trait` 0.1.89 -> 0.1.92 (workspace floor `0.1.92`) to clear the 24 `clippy::double_must_use` errors raised by stable clippy 1.99.0.

### Changed
- `wiki/development/CI-CD.md`: new "Toolchain & Clippy drift" section.
