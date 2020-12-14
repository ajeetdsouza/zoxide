<!-- omit in toc -->
# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)
![.github/workflows/release.yml](https://github.com/ajeetdsouza/zoxide/workflows/.github/workflows/release.yml/badge.svg)

A faster way to navigate your filesystem

<!-- omit in toc -->
## Table of contents

- [Introduction](#introduction)
- [Examples](#examples)
- [Getting started](#getting-started)
  - [Step 1: Install zoxide](#step-1-install-zoxide)
  - [Step 2: Install fzf (optional)](#step-2-install-fzf-optional)
  - [Step 3: Add zoxide to your shell](#step-3-add-zoxide-to-your-shell)
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

### Step 1: Install zoxide

zoxide works across all major platforms. If your distribution isn't included in the list below, you can directly install the binary from GitHub:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ajeetdsouza/zoxide/master/install.sh | sh
```

If you would rather not run a script, you can download the binary from the [Releases](https://github.com/ajeetdsouza/zoxide/releases) page and add it anywhere in your `$PATH`.

<!-- omit in toc -->
#### On Linux

| Distribution      | Repository              | Instructions                                              |
| ----------------- | ----------------------- | --------------------------------------------------------- |
| **Any**           | [crates.io]             | `cargo install zoxide`                                    |
| **Any**           | [Linuxbrew]             | `brew install zoxide`                                     |
| Alpine Linux Edge | [Alpine Linux Packages] | `apk add zoxide`                                          |
| Arch Linux        | [AUR]                   | `yay -Sy zoxide-bin`                                      |
| CentOS            | [Copr]                  | `dnf copr enable atim/zoxide` <br /> `dnf install zoxide` |
| Debian Unstable   | [Debian Packages]       | `apt install zoxide`                                      |
| Fedora            | [Fedora Packages]       | `dnf install zoxide`                                      |
| NixOS             | [nixpkgs]               | `nix-env -iA nixpkgs.zoxide`                              |
| Void Linux        | [Void Linux Packages]   | `xbps-install -S zoxide`                                  |

<!-- omit in toc -->
#### On macOS

| Repository  | Instructions           |
| ----------- | ---------------------- |
| [crates.io] | `cargo install zoxide` |
| [Homebrew]  | `brew install zoxide`  |
| [MacPorts]  | `port install zoxide`  |

<!-- omit in toc -->
#### On Windows

| Repository  | Instructions           |
| ----------- | ---------------------- |
| [crates.io] | `cargo install zoxide` |
| [Scoop]     | `scoop install zoxide` |

<!-- omit in toc -->
#### On BSD

| Distribution  | Repository   | Instructions           |
| ------------- | ------------ | ---------------------- |
| **Any**       | [crates.io]  | `cargo install zoxide` |
| DragonFly BSD | [DPorts]     | `pkg install zoxide`   |
| FreeBSD       | [FreshPorts] | `pkg install zoxide`   |
| NetBSD        | [pkgsrc]     | `pkgin install zoxide` |

<!-- omit in toc -->
#### On Android

| Repository | Instructions           |
| ---------- | ---------------------- |
| [Termux]   | `pkg install zoxide`   |

### Step 2: Install fzf (optional)

[fzf](https://github.com/junegunn/fzf) is a command-line fuzzy finder, used by
zoxide for interactive selection. Installation instructions can be found
[here](https://github.com/junegunn/fzf#installation).

### Step 3: Add zoxide to your shell

If you currently use `z`, `z.lua`, or `zsh-z`, you may want to first import
your existing database into `zoxide`:

```sh
zoxide import /path/to/db
```

<!-- omit in toc -->
#### bash

Add the following line to your `~/.bashrc`:

```sh
eval "$(zoxide init bash)"
```

<!-- omit in toc -->
#### fish

Add the following line to your `~/.config/fish/config.fish`:

```sh
zoxide init fish | source
```

<!-- omit in toc -->
#### PowerShell

Add the following line to your profile:

```powershell
Invoke-Expression (& {
    $hook = if ($PSVersionTable.PSVersion.Major -lt 6) { 'prompt' } else { 'pwd' }
    (zoxide init --hook $hook powershell) -join "`n"
})
```

<!-- omit in toc -->
#### xonsh

Add the following line to your profile (usually `~/.xonshrc`):

```xonsh
execx($(zoxide init xonsh), 'exec', __xonsh__.ctx, filename='zoxide')
```

<!-- omit in toc -->
#### zsh

Add the following line to your `~/.zshrc`:

```sh
eval "$(zoxide init zsh)"
```

<!-- omit in toc -->
#### Any POSIX shell

Add the following line to your shell's configuration file:

```sh
eval "$(zoxide init posix --hook prompt)"
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

[Alpine Linux Packages]: https://pkgs.alpinelinux.org/package/edge/community/x86_64/zoxide
[AUR]: https://aur.archlinux.org/packages/zoxide-bin
[Copr]: https://copr.fedorainfracloud.org/coprs/atim/zoxide/
[crates.io]: https://crates.io/crates/zoxide
[Debian Packages]: https://packages.debian.org/sid/zoxide
[DPorts]: https://github.com/DragonFlyBSD/DPorts/tree/master/sysutils/zoxide
[FreshPorts]: https://www.freshports.org/sysutils/zoxide/
[Fedora Packages]: https://src.fedoraproject.org/rpms/rust-zoxide
[Homebrew]: https://formulae.brew.sh/formula/zoxide
[Linuxbrew]: https://formulae.brew.sh/formula-linux/zoxide
[MacPorts]: https://ports.macports.org/port/zoxide/summary
[nixpkgs]: https://nixos.org/nixos/packages.html?attr=zoxide&channel=nixpkgs-unstable
[pkgsrc]: https://pkgsrc.se/sysutils/zoxide
[Scoop]: https://github.com/ScoopInstaller/Main/tree/master/bucket/zoxide.json
[Termux]: https://github.com/termux/termux-packages/tree/master/packages/zoxide
[Void Linux Packages]: https://github.com/void-linux/void-packages/tree/master/srcpkgs/zoxide

[`dirs` documentation]: https://docs.rs/dirs/latest/dirs/fn.data_local_dir.html
