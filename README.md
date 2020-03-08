# zoxide

[![crates.io](https://img.shields.io/crates/v/zoxide)](https://crates.io/crates/zoxide)

A cd command that learns your habits

## Introduction

`zoxide` is a new `cd` alternative inspired by [`z`](https://github.com/rupa/z) and [`z.lua`](https://github.com/skywind3000/z.lua). It keeps track of the directories you use most frequently, and uses a ranking algorithm to navigate to the best match.

On my system, compiled with the `x86_64-unknown-linux-musl` target, `hyperfine` reports that `zoxide` runs 10-20x faster than `z.lua`, which, in turn, runs 3x faster than `z`. This is pretty significant, since this command runs once at every shell prompt, and any slowdown there will result in an increased loading time for every prompt.

## Examples

```sh
z foo       # cd to top directory matching foo
z foo bar   # cd to top directory matching foo and bar

z foo/      # can also cd into actual directories

zi foo      # cd with interactive selection using fzf

zq foo      # echo the best match, don't cd

za /foo     # add /foo to the database
zr /foo     # remove /foo from the database
```

## Getting started

### Installing `zoxide`

If you have Rust, this should be as simple as:

```sh
cargo install zoxide
```

Otherwise, try the install script:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/ajeetdsouza/zoxide/master/install.sh | sh
```

If you want the interactive fuzzy selection feature, you will also need to install [`fzf`](https://github.com/junegunn/fzf.git).

### Adding `zoxide` to your shell

#### zsh

If you use a package manager, installation should be as simple as adding `ajeetdsouza/zoxide` to your configuration. If you'd rather not use a package manager, simple include the following in your `.zshrc`:

```sh
_zoxide_precmd() {
    zoxide add
}

precmd_functions+=_zoxide_precmd

z() {
    if [ $# -ne 0 ]; then
        _Z_RESULT=$(zoxide query "$@")
        case $_Z_RESULT in
            "query: "*)
                cd "${_Z_RESULT:7}"
                ;;
            *)
                echo "${_Z_RESULT}"
                ;;
        esac
    fi
}

alias zi="z -i"

alias za="zoxide add"
alias zq="zoxide query"
alias zr="zoxide remove"
```

## Configuration

### Environment variables

- `$_ZO_DATA`: sets the location of the database (default: `~/.zo`)
- `$_ZO_MAXAGE`: sets the maximum total rank after which entries start getting deleted
