# Contribution workflow

## Running the suite
- Use `cargo xtask demo <name>` to verify demos through the workbench shell, or pass `--standalone` to launch the binary directly.
- `cargo xtask gallery <target>` mirrors the new `--open` syntax so you can script regression checks against specific categories, themes, or locales.
- `cargo xtask docs` builds `cargo doc --workspace --all-features --no-deps`; copy the output into `docs/api` before opening a pull request.

## Coding guidelines
- Prefer command-bus interactions (`WorkbenchCommand`, `GalleryCommand`) over direct method calls so docs, CLI arguments, and UI buttons stay aligned.
- Documentation helpers must include best practices and gotchas. See `expandable_docs` in the workbench and `doc_section` in the gallery for examples.
- Expose reusable demo APIs via `lib.rs` and keep `main.rs` thin wrappers that call `run()`. This keeps the workbench integration straightforward.

## Pull requests
1. Update relevant markdown docs under `docs/` when changing themes, components, localization keys, or CLI entry points.
2. Run `cargo fmt` and `cargo clippy --workspace --all-targets` before submitting.
3. Attach screenshots using the SVG placeholders in `docs/images/` or capture live previews to help reviewers.
4. If you add new CLI arguments, ensure they are wired to the in-app launchers and documented in the README.
