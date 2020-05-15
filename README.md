# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)
![.github/workflows/release.yml](https://github.com/ajeetdsouza/zoxide/workflows/.github/workflows/release.yml/badge.svg)

A faster way to navigate your filesystem

## Table of contents

- [Introduction](#introduction)
- [Examples](#examples)
- [Getting started](#getting-started)
  - [Installing `zoxide`](#step-1-installing-zoxide)
  - [Adding `zoxide` to your shell](#step-2-adding-zoxide-to-your-shell)
    - [bash](#bash)
    - [fish](#fish)
    - [POSIX](#posix-shells)
    - [PowerShell](#powershell)
    - [zsh](#zsh)
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

#### Fedora 32+

```sh
sudo dnf install zoxide
```

#### Cargo

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

#### POSIX shells

Add the following line to your shell's configuration file:

```sh
eval "$(zoxide init posix --hook prompt)"
```

The `prompt` hook is recommended for POSIX shells because the default `pwd`
hook creates a temporary file for every session, which are only deleted upon
reboot. If you do want to use `pwd` hooks instead, you may want to set up traps
to perform a cleanup once the shell exits:

```sh
trap '_zoxide_cleanup' EXIT HUP KILL TERM
trap '_zoxide_cleanup; trap - INT; kill -s INT "$$"' INT
trap '_zoxide_cleanup; trap - QUIT; kill -s QUIT "$$"' QUIT
```

NOTE: If you modify your `PS1` at any point, you may need to re-run the above
command. This is due to the fact that the hook is stored in `PS1`, in order to
be evaluated every time the prompt is displayed.

#### zsh

Add the following line to your `~/.zshrc`:

```sh
eval "$(zoxide init zsh)"
```

#### PowerShell

Add the following line to your profile:

```powershell
Invoke-Expression (& {
    $hook = if ($PSVersionTable.PSVersion.Major -lt 6) { 'prompt' } else { 'pwd' }
    (zoxide init --hook $hook powershell) -join "`n"
})
```

## Configuration

### `init` flags

- `--cmd`: change the `z` command (and corresponding aliases) to something else
- `--hook <HOOK>`: change the event that adds a new entry to the database
  (default: `pwd`)
  - `none`: never add entries
    (this will make `zoxide` useless unless you manually configure a hook)
  - `prompt`: add an entry at every prompt
  - `pwd`: add an entry whenever the current directory is changed
- `--no-aliases`: don't define extra aliases like `zi`, `zq`, `za`, and `zr`

### Environment variables

- `$_ZO_DATA_DIR`: directory where `zoxide` will store its data files
  (default: platform-specific; see the [`dirs` documentation] for more information)
- `$_ZO_ECHO`: when set to `1`, `z` will print the matched directory before navigating to it
- `$_ZO_EXCLUDE_DIRS`: list of directories separated by platform-specific characters
  ("`:`" on Linux/macOS, "`;`" on Windows) to be excluded from the database
- `$_ZO_MAXAGE`: sets the maximum total rank after which entries start getting deleted

[`dirs` documentation]: https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html
