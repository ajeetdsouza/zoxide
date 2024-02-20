<!-- markdownlint-disable-file MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.9.4] - 2024-02-21

### Changed

- zsh: improved Space-Tab completions.

## [0.9.3] - 2024-02-13

### Added

- Nushell: support for v0.89.0.

## [0.9.2] - 2023-08-04

### Added

- Short option `-a` for `zoxide query --all`.

### Fixed

- PowerShell: use `global` scope for variables / functions.

## [0.9.1] - 2023-05-07

### Added

- Fish/Zsh: aliases on `__zoxide_z` will now use completions.
- Nushell: support for v0.78.0.
- Fish: plugin now works on older versions.
- PowerShell: warn when PowerShell version is too old for `z -` and `z +`.
- PowerShell: support for PWD hooks on all versions.

### Fixed

- Fish: not providing `cd` completions when there is a space in the path.
- Bash/Fish/Zsh: providing `z` completions when the last argument starts with `z!`.
- Bash/Fish/Zsh: attempting to `cd` when the last argument is `z!`.

## [0.9.0] - 2023-01-08

### Added

- `edit` subcommand to adjust the scores of entries.

### Fixed

- Zsh: completions clashing with `zsh-autocomplete`.
- Fzf: 'invalid option' on macOS.
- PowerShell: handle UTF-8 encoding correctly.
- Zsh: don't hide output from `chpwd` hooks.
- Nushell: upgrade minimum supported version to v0.73.0.
- Zsh: fix extra space in interactive completions when no match is found.
- Fzf: various improvements, upgrade minimum supported version to v0.33.0.
- Nushell: accidental redefinition of hooks when initialized twice.

### Removed

- `remove -i` subcommand: use `edit` instead.

## [0.8.3] - 2022-09-02

### Added

- Nushell: support for `z -`.
- Nushell: support for PWD hooks.

### Changed

- Fish: change fuzzy completion prefix to `z!`.
- Zsh: allow `z` to navigate dirstack via `+n` and `-n`.
- Fzf: improved preview window.

### Fixed

- Bash: double forward slash in completions.

## [0.8.2] - 2022-06-26

### Changed

- Fzf: show preview window below results.

### Fixed

- Bash/Fish/POSIX/Zsh: paths on Cygwin.
- Fish: completions not working on certain systems.
- Bash: completions not escaping spaces correctly.

## [0.8.1] - 2021-04-23

### Changed

- Manpages: moved to `man/man1/*.1`.
- Replace `--no-aliases` with `--no-cmd`.
- Elvish: upgrade minimum supported version to v0.18.0.
- Nushell: upgrade minimum supported version to v0.61.0.

### Fixed

- Bash/Zsh: rename `_z` completion function to avoid conflicts with other shell
  plugins.
- Fzf: added `--keep-right` option by default, upgrade minimum supported version
  to v0.21.0.
- Bash: only enable completions on 4.4+.
- Fzf: bypass `ls` alias in preview window.
- Retain ownership of database file.
- `zoxide query --interactive` should not conflict with `--score`.

## [0.8.0] - 2021-12-25

### Added

- Zsh: completions for `z` command.

### Changed

- Fzf: better default options.
- Fish: interactive completions are only triggered when the last argument is
  empty.
- PowerShell: installation instructions.

### Fixed

- PowerShell: use global scope for aliases.
- Zsh: fix errors with `set -eu`.
- Fzf: handle early selection.
- PowerShell: correctly handle escape characters in paths.
- Parse error on Cygwin/MSYS due to CRLF line endings.
- Fzf: handle spaces correctly in preview window.
- Bash: avoid initializing completions on older versions.
- Fzf: avoid launching binary from current directory on Windows.

## [0.7.9] - 2021-11-02

### Changed

- Bash/Fish: improved completions for `z` command.

### Fixed

- Fish: error erasing completions on older versions.
- PowerShell: enable `--cmd cd` to replace the `cd` command.

## [0.7.8] - 2021-10-21

### Added

- Auto-generated completions for [Fig](https://fig.io/).

### Fixed

- Compile error with `clap v3.0.0-beta.5`.

## [0.7.7] - 2021-10-15

### Fixed

- PowerShell: hook not initializing correctly.

## [0.7.6] - 2021-10-13

### Changed

- Nushell: upgrade minimum supported version to v0.37.0.

### Fixed

- Xonsh: error messages in `zi`.
- Xonsh: configuration environment variables not being handled correctly.

## [0.7.5] - 2021-09-09

### Added

- Bash/Elvish: completions for `z` command.

### Changed

- Nushell: upgrade minimum supported version to v0.36.0.
- Nushell: easier installation instructions.

### Fixed

- Elvish: unable to `z` into directories by path.
- Elvish: don't show traceback when `z` or `zi` fails.
- Elvish: nested shells do not initialize correctly.

## [0.7.4] - 2021-08-15

### Fixed

- Compile error with `clap v3.0.0-beta.4`.

## [0.7.3] - 2021-08-05

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
- Lazy deletion for removed directories that have not been accessed in > 90
  days.
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
- `__zoxide_hook` no longer changes value of `$?` within `$PROMPT_COMMAND` on
  bash.

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

- Aliases: `za`, `zq`, `zqi`, `zr`, `zri`. These are trivial aliases that can
  easily be defined manually, and aren't very useful to most users.

## [0.5.0] - 2020-10-30

### Added

- `$_ZO_EXCLUDE_DIRS` now supports globs.
- `zoxide init` now defines `__zoxide_z*` functions that can be aliased as
  needed.
- Support for the [xonsh](https://xon.sh/) shell.
- `zoxide import` can now import from Autojump.

### Changed

- `zoxide init --no-aliases` no longer generates `z` or `zi`.

### Fixed

- Clobber conflicting alias definitions in bash/fish/zsh/POSIX shells.

### Removed

- Deprecated PWD hooks for POSIX shells.
- Lazy deletion for inaccessible directories.

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
- Symlinks are treated as separate directories by default, this can be changed
  by setting `_ZO_RESOLVE_SYMLINKS=1`.

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
- Interactive mode in `zoxide` no longer errors out if `fzf` exits gracefully.
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

- `zoxide query` output no longer has the `query:` prefix.

### Fixed

- Queries now also include checks for if the top level directory matches.

## [0.3.0] - 2020-03-30

### Added

- Automatic migration from `v0.2.x` databases.
- `$_ZO_EXCLUDE_DIRS` to prevent directories from being added to the database.
- Support for POSIX-compliant shells.

### Changed

- Database location defaults to user's local data directory.
- Database schema now includes a version number.
- `migrate` subcommand renamed to `import`.

### Fixed

- Thread safety using unique tempfile names for each `zoxide` instance.
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

[0.9.4]: https://github.com/ajeetdsouza/zoxide/compare/v0.9.3...v0.9.4
[0.9.3]: https://github.com/ajeetdsouza/zoxide/compare/v0.9.2...v0.9.3
[0.9.2]: https://github.com/ajeetdsouza/zoxide/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.8.3...v0.9.0
[0.8.3]: https://github.com/ajeetdsouza/zoxide/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/ajeetdsouza/zoxide/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/ajeetdsouza/zoxide/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.9...v0.8.0
[0.7.9]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.8...v0.7.9
[0.7.8]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.7...v0.7.8
[0.7.7]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.6...v0.7.7
[0.7.6]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.5...v0.7.6
[0.7.5]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.4...v0.7.5
[0.7.4]: https://github.com/ajeetdsouza/zoxide/compare/v0.7.3...v0.7.4
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
