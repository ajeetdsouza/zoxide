# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)
![.github/workflows/release.yml](https://github.com/ajeetdsouza/zoxide/workflows/.github/workflows/release.yml/badge.svg)

A cd command that learns your habits

## Table of contents

- [Introduction](#introduction)
- [Examples](#examples)
- [Getting started](#getting-started)
  - [Installing `zoxide`](#step-1-installing-zoxide)
  - [Adding `zoxide` to your shell](#step-2-adding-zoxide-to-your-shell)
    - [zsh](#zsh)
    - [bash](#bash)
    - [fish](#fish)
    - [POSIX](#posix)
- [Configuration](#configuration)
  - [`init` flags](#init-flags)
  - [Environment variables](#environment-variables)

## Introduction

`zoxide` is a blazing fast alternative to `cd`, inspired by
[`z`](https://github.com/rupa/z) and [`z.lua`](https://github.com/skywind3000/z.lua).
It keeps track of the directories you use most frequently, and uses a ranking algorithm
to navigate to the best match.

## Examples

```sh
z foo       # cd to highest ranked directory matching foo
z foo bar   # cd to highest ranked directory matching foo and bar

z foo/      # can also cd into actual directories

zi foo      # cd with interactive selection using fzf

zq foo      # echo the best match, don't cd

za /foo     # add /foo to the database
zr /foo     # remove /foo from the database
```

## Getting started

### Step 1: Installing `zoxide`

If you have Rust, this should be as simple as:

```sh
cargo install zoxide -f
```

Otherwise, try the install script:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ajeetdsouza/zoxide/master/install.sh | sh
```

If you want the interactive fuzzy selection feature, you will also need to install
[`fzf`](https://github.com/junegunn/fzf.git).

### Step 2: Adding `zoxide` to your shell

If you currently use `z`, `z.lua`, or `zsh-z`, you may want to first import
your existing database into `zoxide`:

```sh
zoxide import /path/to/db
```

#### zsh

Add the following line to your `~/.zshrc`:

```sh
eval "$(zoxide init zsh)"
```

#### bash

Add the following line to your `~/.bashrc`:

```sh
eval "$(zoxide init bash)"
```

#### fish

Add the following line to your `~/.config/fish/config.fish`:

```sh
zoxide init fish | source
```

#### POSIX

Add the following line to your shell's configuration file:

```sh
eval "$(zoxide init posix)"
```

NOTE: If you modify your `PS1` at any point, you may need to re-run the above command. This is due
to the fact that the hook is stored in `PS1`, in order to be evaluated every time the prompt is
displayed.

NOTE: PWD hooks are currently not supported for POSIX shells.

## Configuration

### `init` flags

- `--hook <HOOK>`: change the event that adds a new entry to the database (default: `prompt`)
  - `none`: never add entries (this will make `zoxide` useless unless you manually configure a hook)
  - `prompt`: add an entry at every prompt
  - `pwd`: add an entry whenever you change directories
- `--no-define-aliases`: don't define extra aliases like `zi`, `zq`, `za`, and `zr`
- `--z-cmd`: rename the `z` command to something else

### Environment variables

- `$_ZO_DATA_DIR`: directory where `zoxide` will store its data files (default:
  platform-specific; see the [`dirs` documentation] for more information)
- `$_ZO_ECHO`: `z` will print the matched directory before navigating to it
- `$_ZO_EXCLUDE_DIRS`: list of directories separated by platform-specific
  characters ("`:`" on Linux and macOS, and "`;`" on Windows) to be excluded from
  the database
- `$_ZO_MAXAGE`: sets the maximum total rank after which entries start getting deleted

[`dirs` documentation]: https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html

## Notes

If you recently updated `zoxide` from an older version to `v0.3` and are 
seeing the following error:

```
Error: could not deserialize old database

Caused by:
    io error: failed to fill whole buffer
```

It is because v0.2 and v0.3 databases are incompatible. There is a script to 
automatically migrate your database to the new format, but it seems to be 
failing on some systems.

For a quick fix, you can try removing the old database file: `rm ~/.zo`
