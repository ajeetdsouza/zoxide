# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)

![.github/workflows/release.yml](https://github.com/ajeetdsouza/zoxide/workflows/.github/workflows/release.yml/badge.svg)

A cd command that learns your habits

## Table of contents

- [Introduction](#introduction)
- [Examples](#examples)
- [Getting started](#getting-started)
  - [Installing `zoxide`](#installing-zoxide)
  - [Adding `zoxide` to your shell](#adding-zoxide-to-your-shell)
    - [zsh](#zsh)
    - [bash](#bash)
    - [fish](#fish)
- [Configuration](#configuration)
  - [Environment variables](#environment-variables)

## Introduction

`zoxide` is a new `cd` alternative inspired by [`z`](https://github.com/rupa/z) and [`z.lua`](https://github.com/skywind3000/z.lua). It keeps track of the directories you use most frequently, and uses a ranking algorithm to navigate to the best match.

On my system, compiled with the `x86_64-unknown-linux-musl` target, `hyperfine` reports that `zoxide` runs 10-20x faster than `z.lua`, which, in turn, runs 3x faster than `z`. This is pretty significant, since this command runs once at every shell prompt, and any slowdown there will result in an increased loading time for every prompt.

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
cargo install zoxide
```

Otherwise, try the install script:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ajeetdsouza/zoxide/master/install.sh | sh
```

If you want the interactive fuzzy selection feature, you will also need to install [`fzf`](https://github.com/junegunn/fzf.git).

### Step 2: Adding `zoxide` to your shell

By default, `zoxide` defines the `z`, `zi`, `za`, `zq`, and `zr` aliases. If you'd like to go with just the barebones `z`, pass the `--no-define-aliases` flag to `zoxide init`.

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

## Configuration

### Environment variables

- `$_ZO_DATA`: sets the location of the database (default: `~/.zo`)
- `$_ZO_MAXAGE`: sets the maximum total rank after which entries start getting deleted
