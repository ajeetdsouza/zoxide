# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)
![.github/workflows/release.yml](https://github.com/ajeetdsouza/zoxide/workflows/.github/workflows/release.yml/badge.svg)

A faster way to navigate your filesystem

## Table of contents

- [Introduction](#introduction)
- [Examples](#examples)
- [Getting started](#getting-started)
  - [Installing `zoxide`](#step-1-installing-zoxide)
  - [Installing `fzf` (optional)](#step-2-installing-fzf-optional)
  - [Adding `zoxide` to your shell](#step-3-adding-zoxide-to-your-shell)
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

![demo](./demo.gif)

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

#### On Arch Linux [![Arch Linux](https://repology.org/badge/version-for-repo/aur/zoxide.svg?header=aur)](https://aur.archlinux.org/packages/zoxide)

```sh
yay -S zoxide
```

#### On CentOS

```sh
dnf copr enable atim/zoxide
dnf install zoxide
```

#### On DragonFly BSD [![DPorts](https://repology.org/badge/version-for-repo/dports/zoxide.svg?header=dports)](https://github.com/DragonFlyBSD/DPorts/tree/master/sysutils/zoxide)

```sh
pkg install zoxide
```

#### On Fedora 32+ [![Fedora](https://repology.org/badge/version-for-repo/fedora_rawhide/zoxide.svg?header=fedora)](https://src.fedoraproject.org/rpms/rust-zoxide)

```sh
dnf install zoxide
```

#### On FreeBSD [![FreeBSD](https://repology.org/badge/version-for-repo/freebsd/zoxide.svg?header=freebsd)](https://www.freshports.org/sysutils/zoxide/)

```sh
pkg install zoxide
```

#### On macOS / Linux (via Homebrew / Linuxbrew) [![Homebrew](https://repology.org/badge/version-for-repo/homebrew/zoxide.svg?header=homebrew)](https://formulae.brew.sh/formula/zoxide)

```sh
brew install zoxide
```

#### On macOS (via MacPorts) [![MacPorts](https://repology.org/badge/version-for-repo/macports/zoxide.svg?header=macports)](https://ports.macports.org/port/zoxide/summary)

```sh
port install zoxide
```

#### On NixOS [![nixpkgs unstable](https://repology.org/badge/version-for-repo/nix_unstable/zoxide.svg?header=nixpkgs%20unstable)](https://nixos.org/nixos/packages.html?attr=zoxide&channel=nixpkgs-unstable)

```sh
nix-env -iA nixpkgs.zoxide
```

#### On Void Linux [![Void Linux](https://repology.org/badge/version-for-repo/void_x86_64/zoxide.svg?header=void%20linux)](https://github.com/void-linux/void-packages/tree/master/srcpkgs/zoxide)

```sh
xbps-install -S zoxide
```

#### On Windows (via [Scoop](https://github.com/lukesampson/scoop)) [![Scoop](https://repology.org/badge/version-for-repo/scoop/zoxide.svg?header=scoop)](https://github.com/ScoopInstaller/Main/tree/master/bucket/zoxide.json)

```powershell
scoop install zoxide
```

#### Other (via Cargo) [![crates.io package](https://repology.org/badge/version-for-repo/crates_io/zoxide.svg?header=crates.io)](https://crates.io/crates/zoxide)

```sh
cargo install zoxide -f
```

#### Other (via precompiled binary) [![GitHub releases](https://img.shields.io/github/v/release/ajeetdsouza/zoxide?color=blue&label=github%20releases&sort=semver)](https://github.com/ajeetdsouza/zoxide/releases)

Use the installation script:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ajeetdsouza/zoxide/master/install.sh | sh
```

Alternatively, you can also download a precompiled binary from the
[releases](https://github.com/ajeetdsouza/zoxide/releases) page and add it to
your `PATH`.

### Step 2: Installing `fzf` (optional)

If you want to use interactive fuzzy selection, you will also need to install
[`fzf`](https://github.com/junegunn/fzf#installation).

### Step 3: Adding `zoxide` to your shell

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

#### PowerShell

Add the following line to your profile:

```powershell
Invoke-Expression (& {
    $hook = if ($PSVersionTable.PSVersion.Major -lt 6) { 'prompt' } else { 'pwd' }
    (zoxide init --hook $hook powershell) -join "`n"
})
```

#### zsh

Add the following line to your `~/.zshrc`:

```sh
eval "$(zoxide init zsh)"
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
- `$_ZO_FZF_OPTS`: custom flags to pass to `fzf`
- `$_ZO_MAXAGE`: sets the maximum total age after which entries start getting deleted
- `$_ZO_RESOLVE_SYMLINKS`: when set to `1`, `z add` will resolve symlinks.

[`dirs` documentation]: https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html
