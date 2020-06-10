# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `$_ZO_FZF_OPTS` to specify custom options for `fzf`
- `zoxide query --list` to list all matches
- `zoxide query --score` to show score along with result

### Changed

- `zri` is now a shell function.

### Removed

- Help menus for `z` and `zri`.

## [0.4.1] - 2020-05-25

### Added

- Support for PowerShell.

### Removed

- Backward compatibility with `v0.2.x` databases.
- Support for paths with invalid UTF-8.

## [0.4.0] - 2020-05-03

### Added

- Interactive mode for removing entries (`zoxide remove -i`).
- Aliases for interactive `query` and `remove` (`zqi` and `zri` respectively).
- PWD hooks for POSIX shells.

### Changed

- `zoxide remove` now throws an error if there was no match in the database.
- Interactive mode in `zoxide` no longer throws an error if `fzf` exits gracefully.
- Canonicalize to regular paths instead of UNC paths on Windows.
- `zoxide init` now uses PWD hooks by default for better performance.
- `$_ZO_ECHO` now only works when set to `1`.
- Using the `--z-cmd` flag now also renames the associated aliases.
- The `--z-cmd` flag has been renamed to `--cmd`.
- The `--no-define-aliases` flag has been renamed to `--no-aliases`.

### Fixed

- `fish` no longer `cd`s to the user's home when no match is found.

## [0.3.1] - 2020-04-03

### Added

- Version output displays `git` revision information.
- `--z-cmd` flag for `zoxide init` to rename the `z` command to something else.

### Changed

- Query output no longer has the `query:` prefix, so `$(zq)` can now be used as an argument to commands.

### Fixed

- Queries now also include checks for if the top level directory matches.

## [0.3.0] - 2020-03-30

### Added

- Automatic migration from `v0.2.x` databases.
- `$_ZO_EXCLUDE_DIRS` to prevent certain directories from being added to the database.
- Support for POSIX-compliant shells.

### Changed

- Database location defaults to user's local data directory.
- Database schema now includes a version number.
- `migrate` subcommand renamed to `import`.

### Fixed

- Achieve thread safety using unique temporary database file names for each `zoxide` instance.
- Incomprehensive "could not allocate" message on database corruption.

## [0.2.2] - 2020-03-20

### Fixed

- Incorrect exit codes in `z` command on `fish`.

### Removed

- File locks on database.

## [0.2.1] - 2020-03-16

### Added

- `$_ZO_ECHO` to echo match before `cd`ing.
- Minimal `ranger` plugin.
- PWD hook to only update the database when the current directory is changed.
- Support for the `bash` shell.
- `migrate` subcommand to allow users to migrate from `z`.

### Fixed

- Interactive queries causing other open shells to hang.

## [0.2.0] - 2020-03-11

### Added

- `init` subcommand to remove dependency on shell plugin managers.
- Support for `z -` command to go to previous directory.
- `Cargo.lock` for more reproducible builds.
- Support for the `fish` shell.

### Fixed

- `_zoxide_precmd` overriding other precmd hooks on `zsh`.

## [0.1.1] - 2020-03-08

### Added

- Install script for Linux/macOS users.
- Aging algorithm to remove stale entries.

### Changed

- Database schema now uses `f64` values for rank instead of `i32`.

### Fixed

- Multiple hooks being added upon initializing `zoxide` multiple times.

## [0.1.0] - 2020-03-05

### Added

- GitHub Actions pipeline to build and upload releases.
- Support for the `zsh` shell.

[0.4.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/ajeetdsouza/zoxide/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ajeetdsouza/zoxide/commits/v0.1.0
