## Fippli Language

Welcome! This repo contains the reference implementation of the Fippli language (interpreter, CLI tools, and supporting documentation).

### Prerequisites

- Rust toolchain via [`rustup`](https://rustup.rs/) (Rust 1.70 or newer recommended).
- `cargo` comes with Rust and is required for building, running, and testing.

After installing `rustup`, update to the latest stable toolchain:

```
rustup update stable
rustup override set stable
```

Run these commands inside the project root to ensure the repo always uses a recent compiler that meets dependency requirements.

### Running a Fip program

The CLI exposes a `run` command that evaluates a `.fip` source file. From the repo root:

```
cargo run -- run test-program/main.fip
```

To run another program, point the command at any `.fip` file:

```
cargo run -- run path/to/program.fip
```

The interpreter resolves imports relative to the entry file’s directory.

### Installing the CLI

If you want a reusable binary instead of invoking through `cargo run`, install it locally:

```
cargo install --path . --bin fip
```

This places the `fip` binary in `~/.cargo/bin`. Make sure that directory is on your `PATH`. After installation you can run:

```
fip run test-program/main.fip
```

### Formatting source files

The CLI also includes a formatter. To print a formatted version of a file:

```
cargo run -- format path/to/file.fip
```

To rewrite the file in place:

```
cargo run -- format path/to/file.fip --write
```

If you installed the CLI, replace `cargo run --` with `fip`.

### Docs builder

Documentation pages are generated from the markdown specs under `/syntax`. Use the helper script to rebuild the static site:

```
./scripts/build-docs.sh
```

The script runs the dedicated builder found at `docs/tools/build-docs`. The first invocation downloads Rust dependencies; subsequent runs work offline.

### Testing

Unit tests are implemented inside the interpreter crate. Run them with:

```
cargo test
```

### Project layout

- `src/` – Interpreter and CLI implementation
- `syntax/` – Language reference specs (markdown)
- `docs/` – Generated documentation site (static HTML/CSS)
- `test-program/` – Sample `.fip` programs used for manual testing
- `scripts/` – Utility scripts (`build-docs.sh`, etc.)

### Contributing

1. Ensure `cargo fmt` and `cargo test` pass before opening a PR.
2. Update relevant docs in `syntax/` and rebuild the site if language features change.
3. Keep new files ASCII unless UTF-8 is required for examples.
