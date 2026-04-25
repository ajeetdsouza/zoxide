# zoxide

zoxide is a smarter `cd` command for your terminal, inspired by `z` and `autojump`.
It remembers which directories you use most frequently, so you can "jump" to them
in just a few keystrokes. It works on all major shells.

This is a Rust CLI project. The repository lives at
<https://github.com/ajeetdsouza/zoxide>.

---

## Technology stack

- **Language**: Rust (edition 2024, MSRV 1.85.0)
- **CLI parsing**: `clap` v4 with derive macros
- **Template engine**: `askama` (for generating shell integration scripts)
- **Database serialization**: `bincode`
- **Error handling**: `anyhow`
- **Self-referencing structs**: `ouroboros` (used by the database layer)
- **Task runner**: `just`
- **Dev environment**: Nix (`shell.nix`)

---

## Project structure

```
.
├── Cargo.toml              # Package manifest
├── build.rs                # Generates shell completions into contrib/completions/
├── justfile                # Task definitions (lint, test, fmt)
├── rustfmt.toml            # Rustfmt configuration
├── shell.nix               # Reproducible Nix development shell
├── Cross.toml              # Cross-compilation configuration
│
├── src/
│   ├── main.rs             # Entry point: parses args, runs commands, handles SilentExit
│   ├── config.rs           # Reads environment variables (_ZO_DATA_DIR, _ZO_ECHO, etc.)
│   ├── error.rs            # SilentExit and BrokenPipeHandler traits
│   ├── util.rs             # Fzf wrapper, atomic file writes, path resolution, time utils
│   ├── shell.rs            # Askama template structs for each supported shell
│   │
│   ├── cmd/                # CLI subcommand implementations
│   │   ├── mod.rs          # Run trait and command dispatch
│   │   ├── cmd.rs          # Clap derive structs for all subcommands and enums
│   │   ├── add.rs          # `zoxide add` — add/increment directory rank
│   │   ├── query.rs        # `zoxide query` — search database (list/interactive/first)
│   │   ├── init.rs         # `zoxide init` — render shell integration script
│   │   ├── import.rs       # `zoxide import` — import from autojump / z
│   │   ├── remove.rs       # `zoxide remove` — remove directory from database
│   │   └── edit.rs         # `zoxide edit` — interactive database editor via fzf
│   │
│   └── db/                 # Database layer
│       ├── mod.rs          # Database struct (open, save, add, remove, age, dedup, sort)
│       ├── dir.rs          # Dir struct (path, rank, last_accessed, score, display)
│       └── stream.rs       # Stream iterator for querying directories with filters
│
├── templates/              # Askama templates for shell integration scripts
│   ├── bash.txt, zsh.txt, fish.txt, powershell.txt, ...
│
├── contrib/completions/    # Generated completion files (updated by build.rs)
│
├── man/man1/               # Man pages
│
└── tests/
    └── completions.rs      # Integration tests for generated completion scripts
```

---

## Build and test commands

### Standard (without Nix)

```bash
# Build debug binary
cargo build

# Build release binary
cargo build --release

# Run tests
cargo test

# Run lints
cargo clippy --all-features --all-targets -- -Dwarnings

# Format code
cargo fmt --all
```

### With Nix (recommended on Unix)

The project uses a pure Nix shell for CI and local development. All linting and
testing tools are pinned there.

```bash
# Enter dev shell
nix-shell

# Run lints (includes rustfmt, clippy, msrv, udeps, shellcheck, markdownlint, ...)
just lint

# Run tests (uses cargo-nextest when inside nix-shell)
just test

# Format everything (Rust, Nix, shell, YAML, Markdown)
just fmt
```

### Important notes

- `build.rs` generates shell completions into `contrib/completions/`. It is not
  a no-op — modifying CLI definitions in `src/cmd/cmd.rs` will regenerate these
  files on the next build.
- The `nix-dev` Cargo feature gates tests that require external binaries
  (shells, fzf, shellcheck, shfmt, black, mypy, pylint, fish_indent, etc.).
  These tests only run when the feature is enabled and the tools are available.

---

## Code style guidelines

- Use `cargo fmt` with the project's `rustfmt.toml`.
- Key settings:
  - `group_imports = "StdExternalCrate"`
  - `imports_granularity = "Module"`
  - `style_edition = "2024"`
  - `use_try_shorthand = true`
  - `use_field_init_shorthand = true`
- Prefer `?` and `anyhow::Result` for error propagation.
- Use `SilentExit` for intentional early exits that should not print an error
  message.
- Use `BrokenPipeHandler::pipe_exit` when writing to stdout/stderr to handle
  broken pipes gracefully (e.g. `| head`).
- Keep platform-specific code behind `cfg(unix)` / `cfg(windows)`.
- Only use `unsafe` when absolutely necessary (the project currently uses it
  only once, to forcibly disable Rust backtrace environment variables at startup).

---

## Testing instructions

### Unit tests

Embedded in source files under `#[cfg(test)]`:

- `src/db/mod.rs` — database add/remove persistence
- `src/db/stream.rs` — keyword matching algorithm (parameterized with `rstest`)
- `src/cmd/import.rs` — autojump and z importer logic

Run them with:

```bash
cargo test
```

### Integration tests

- `tests/completions.rs` — loads generated completion scripts into bash, fish,
  zsh, and PowerShell to verify they parse without errors. Requires the
  `nix-dev` feature.

### Shell integration tests

Located in `src/shell.rs` under `#[cfg(feature = "nix-dev")]`.
These are the most comprehensive tests: they render every Askama template for
all combinations of options and then:

- Execute the generated script in the actual shell binary (bash, dash, zsh,
  fish, elvish, nushell, pwsh, tcsh, xonsh)
- Run linters on the generated script (shellcheck, shfmt, fish_indent, black,
  mypy, pylint)

Because they require many external tools, they are gated behind the `nix-dev`
feature and are intended to run inside the Nix shell.

```bash
# Run all tests including nix-dev tests
nix-shell --pure --run "cargo nextest run --all-features --no-fail-fast --workspace"
```

---

## Security considerations

- **Atomic database writes**: The database is written atomically using a
  temporary file + `fs::rename` to prevent corruption on crashes. On Unix, the
  temporary file preserves the original file's owner via `fchown`.
- **Windows executable resolution**: On Windows, `fzf.exe` is resolved via the
  `which` crate before spawning to avoid the current-directory executable search
  behavior of `CreateProcess`.
- **Path validation**: `add` rejects paths containing newlines or carriage
  returns. Symlink resolution is optional and controlled by `_ZO_RESOLVE_SYMLINKS`.
- **Database size limit**: Deserialization enforces a 32 MiB maximum to prevent
  malformed input from causing excessive memory use.
- **Backtrace suppression**: The binary forcibly unsets `RUST_BACKTRACE` and
  `RUST_LIB_BACKTRACE` at startup to avoid leaking internal paths in error
  output.

---

## Key architectural details

### Database

- Stored as a binary file (`db.zo`) in the data directory, serialized with
  `bincode`. The format has a version header (current version: 3).
- Each entry has a `path`, `rank` (f64), and `last_accessed` (Unix epoch).
- The `Database` struct is self-referencing (via `ouroboros`) because `Dir`
  contains a `Cow<'a, str>` borrowed from the deserialized bytes.
- Scoring applies time-based multipliers: entries accessed within the last hour
  get 4x, within a day 2x, within a week 0.5x, older 0.25x.
- Aging keeps total rank bounded. When the sum exceeds `_ZO_MAXAGE` (default
  10000), all ranks are scaled down and entries below rank 1.0 are removed.
- Non-existent directories are lazily removed during queries if they have not
  been accessed within a 3-month TTL.

### Shell integration

- `zoxide init <shell>` renders an Askama template from `templates/` to stdout.
- The generated script defines `z` (jump), `zi` (interactive jump), and a hook
  that calls `zoxide add` automatically.
- Hook modes: `none` (never), `prompt` (every prompt), `pwd` (on directory
  change, default).

### Query flow

1. Open the database.
2. Create a `Stream` with filters (keywords, base_dir, exclude globs, exists check).
3. Stream sorts entries by score and returns them one by one.
4. For interactive mode, results are piped into `fzf`.
5. Save the database (lazy cleanup may mark it dirty).

---

## Release and deployment

- CI runs on `ubuntu-latest` inside a Nix shell for lints and tests.
- The release workflow cross-compiles for many targets (Linux musl, Android,
  macOS, Windows) using `cross`, packages them as `.tar.gz`, `.zip`, and `.deb`,
  and uploads artifacts.
- A draft GitHub release is automatically created when a commit on `main`
  starts with `chore(release)`.
