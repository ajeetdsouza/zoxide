<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.3] - 2020-08-05

### Added

- `zoxide add` and `zoxide remove` now accept multiple arguments.

### Fixed

- Nushell: errors on 0.33.0.
- PowerShell: errors when initializing in `StrictMode`.
- Bash/POSIX: remove conflicting alias definitions when initializing.
- Bash: remove extra semicolon when setting `$PROMPT_COMMAND`.
- Xonsh: use shell environment instead of `os.environ`.

## [0.7.2] - 2021-06-10

### Fixed

- `zoxide -V` not printing version.

## [0.7.1] - 2021-06-09

### Added

- Auto-generated shell completions.
- `zoxide query --all` for listing deleted directories.
- Lazy deletion for removed directories that have not been accessed in > 90 days.
- Nushell: support for 0.32.0+.

### Fixed

- Nushell: avoid calling `__zoxide_hook` on non-filesystem subshells.
- Fish: `alias cd=z` now works, but it must be done after calling `zoxide init`.
- PowerShell: avoid calling `__zoxide_hook` on non-filesystem providers.
- Fish: avoid calling `__zoxide_hook` in private mode.

## [0.7.0] - 2021-05-02

### Added

- Manpages for all subcommands.
- Default prompt for Nushell.

### Changed

- `zoxide remove -i` now accepts multiple selections.
- `zoxide add` no longer accepts zero parameters.
- `$_ZO_EXCLUDE_DIRS` now defaults to `"$HOME"`.
- Binary releases now use `.zip` on Windows, `.tar.gz` otherwise.

### Fixed

- `cd -` on fish shells.
- `__zoxide_hook` no longer changes value of `$?` within `$PROMPT_COMMAND` on bash.

### Removed

- GitHub install script.
- Release binaries built with `glibc`, use `musl` instead.

## [0.6.0] - 2021-04-09

### Added

- Support for [Nushell](https://www.nushell.sh/).
- Support for [Elvish](https://elv.sh/).

### Changed

- `z` now excludes the current directory from search results.

### Fixed

- Removed backtraces on Rust nightly.
- Generated shell code avoids using aliased builtins.
- Handle broken pipe errors gracefully when writing to streams.
- NUL file appearing in working directory on Windows.
- Accidental redefinition of hooks when initialized twice on some shells.
- zoxide unable to find itself on Xonsh shells.

### Removed

- Aliases: `za`, `zq`, `zqi`, `zr`, `zri`. These are trivial aliases to zoxide that can easily be defined manually, and aren't very useful to most users.

## [0.5.0] - 2020-10-30

### Added

- Inaccessible directories are no longer removed; zoxide can now remember paths on removable devices.
- `$_ZO_EXCLUDE_DIRS` now supports globs.
- `zoxide init` now defines `__zoxide_z*` functions that can be aliased as needed.
- Support for the [xonsh](https://xon.sh/) shell.
- `zoxide import` can now import from Autojump.

### Changed

- `zoxide init --no-aliases` no longer generates `z` or `zi`.

### Fixed

- Clobber conflicting alias definitions in bash/fish/zsh/POSIX shells.

### Removed

- Deprecated PWD hooks for POSIX shells.

## [0.4.3] - 2020-07-04

### Fixed

- Bug in Fish init script

## [0.4.2] - 2020-07-03

### Added

- `$_ZO_FZF_OPTS` to specify custom options for `fzf`
- `zoxide query --list` to list all matches
- `zoxide query --score` to show score along with result

### Changed

- Increased default value of `$_ZO_MAXAGE` to `10000`.
- Symlinks are treated as separate directories by default, this can be changed by setting `_ZO_RESOLVE_SYMLINKS=1`.

### Removed

- Help menus for `z` and `zri`.
- `zoxide remove -i` is replaced with `zri`.

## [0.4.1] - 2020-05-25

### Added

- Support for powershell.

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

- fish no longer `cd`s to the user's home when no match is found.

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

- Incorrect exit codes in `z` command on fish.

### Removed

- File locks on database.

## [0.2.1] - 2020-03-16

### Added

- `$_ZO_ECHO` to echo match before `cd`ing.
- Minimal `ranger` plugin.
- PWD hook to only update the database when the current directory is changed.
- Support for bash.
- `migrate` subcommand to allow users to migrate from `z`.

### Fixed

- Interactive queries causing other open shells to hang.

## [0.2.0] - 2020-03-11

### Added

- `init` subcommand to remove dependency on shell plugin managers.
- Support for `z -` command to go to previous directory.
- `Cargo.lock` for more reproducible builds.
- Support for the fish shell.

### Fixed

- `_zoxide_precmd` overriding other precmd hooks on zsh.

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
- Support for zsh.

[0.7.3]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.2...v0.7.3
[0.7.2]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.1...v0.7.2
[0.7.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.0...v0.7.1
[0.7.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.4.3...v0.5.0
[0.4.3]: https://github.com/ajeetdsouza/zoxide/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/ajeetdsouza/zoxide/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/ajeetdsouza/zoxide/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ajeetdsouza/zoxide/commits/v0.1.0
