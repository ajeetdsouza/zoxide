# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)

A cd command that learns your habits

## Table of contents

- [Introduction](#introduction)
- [Examples](#examples)
- [Getting started](#getting-started)
  - [Installing `zoxide`](#installing-zoxide)
  - [Adding `zoxide` to your shell](#adding-zoxide-to-your-shell)
    - [zsh](#zsh)
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

#### zsh

Using [antibody](https://github.com/getantibody/antibody):

```sh
antibody bundle ajeetdsouza/zoxide
```

Using [zinit](https://github.com/zdharma/zinit):

```sh
zinit light ajeetdsouza/zoxide
```

Using [antigen](https://github.com/zsh-users/antigen):

```sh
antigen bundle zsh-users/zsh-syntax-highlighting
```

Using [zgen](https://github.com/tarjoilija/zgen):

```sh
zgen load ajeetdsouza/zoxide
```

Using [zplug](https://github.com/zplug/zplug):

```sh
zplug "zsh-users/zsh-history-substring-search"
```

If you'd rather not use a package manager, add the contents of [zoxide.plugin.zsh](zoxide.plugin.zsh) to your `.zshrc`.

#### fish

Using [fisher](https://github.com/jorgebucaran/fisher):

```sh
fisher add ajeetdsouza/zoxide
```

Using [oh-my-fish](https://github.com/oh-my-fish/oh-my-fish):

```sh
omf install https://github.com/ajeetdsouza/zoxide
```

## Configuration

### Environment variables

- `$_ZO_DATA`: sets the location of the database (default: `~/.zo`)
- `$_ZO_MAXAGE`: sets the maximum total rank after which entries start getting deleted
