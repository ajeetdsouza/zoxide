<!-- markdownlint-configure-file {
  "MD013": {
    "code_blocks": false,
    "tables": false
  },
  "MD033": false,
  "MD041": false
} -->

<div align="center">

# zoxide

[![crates.io][crates.io-badge]][crates.io]
[![Downloads][downloads-badge]][releases]
[![Built with Nix][builtwithnix-badge]][builtwithnix]

zoxide is a **smarter cd command**, inspired by z and autojump.

It remembers which directories you use most frequently, so you can "jump" to
them in just a few keystrokes.<br />
zoxide works on all major shells.

[Getting started](#getting-started) •
[Installation](#installation) •
[Configuration](#configuration) •
[Integrations](#third-party-integrations)

</div>

## Getting started

![Tutorial][tutorial]

```sh
z foo              # cd into highest ranked directory matching foo
z foo bar          # cd into highest ranked directory matching foo and bar

z ~/foo            # z also works like a regular cd command
z foo/             # cd into relative path
z ..               # cd one level up
z -                # cd into previous directory

zi foo             # cd with interactive selection (using fzf)

z foo<SPACE><TAB>  # show interactive completions (zoxide v0.7.10+, bash/fish/zsh only)
```

Read more about the matching algorithm [here][algorithm-matching].

## Installation

### *Step 1: Install zoxide*

zoxide runs on most major platforms. If your platform isn't listed below,
please [open an issue][issues].

<details>
<summary>Linux</summary>

To install zoxide, run this command in your terminal:

```sh
curl -sS https://webinstall.dev/zoxide | bash
```

Alternatively, you can use a package manager:

| Distribution       | Repository              | Instructions                                                                                   |
| ------------------ | ----------------------- | ---------------------------------------------------------------------------------------------- |
| ***Any***          | **[crates.io]**         | `cargo install zoxide --locked`                                                                |
| *Any*              | [conda-forge]           | `conda install -c conda-forge zoxide`                                                          |
| *Any*              | [Linuxbrew]             | `brew install zoxide`                                                                          |
| Alpine Linux 3.13+ | [Alpine Linux Packages] | `apk add zoxide`                                                                               |
| Arch Linux         | [Arch Linux Community]  | `pacman -S zoxide`                                                                             |
| CentOS 7+          | [Copr]                  | `dnf copr enable atim/zoxide` <br /> `dnf install zoxide`                                      |
| Debian 11+         | [Debian Packages]       | `apt install zoxide`                                                                           |
| Devuan 4.0+        | [Devuan Packages]       | `apt install zoxide`                                                                           |
| Fedora 32+         | [Fedora Packages]       | `dnf install zoxide`                                                                           |
| Gentoo             | [GURU Overlay]          | `eselect repository enable guru` <br /> `emerge --sync guru` <br /> `emerge app-shells/zoxide` |
| Manjaro            |                         | `pacman -S zoxide`                                                                             |
| NixOS              | [nixpkgs]               | `nix-env -iA nixpkgs.zoxide`                                                                   |
| Parrot OS          |                         | `apt install zoxide`                                                                           |
| Raspbian 11+       | [Raspbian Packages]     | `apt install zoxide`                                                                           |
| Ubuntu 21.04+      | [Ubuntu Packages]       | `apt install zoxide`                                                                           |
| Void Linux         | [Void Linux Packages]   | `xbps-install -S zoxide`                                                                       |

</details>

<details>
<summary>macOS</summary>

To install zoxide, use a package manager:

| Repository      | Instructions                          |
| --------------- | ------------------------------------- |
| **[crates.io]** | `cargo install zoxide --locked`       |
| [conda-forge]   | `conda install -c conda-forge zoxide` |
| [Homebrew]      | `brew install zoxide`                 |
| [MacPorts]      | `port install zoxide`                 |

</details>

<details>
<summary>Windows</summary>

To install zoxide, run this command in your command prompt:

```sh
curl.exe -A "MS" https://webinstall.dev/zoxide | powershell
```

Alternatively, you can use a package manager:

| Repository      | Instructions                          |
| --------------- | ------------------------------------- |
| **[crates.io]** | `cargo install zoxide --locked`       |
| [Chocolatey]    | `choco install zoxide`                |
| [conda-forge]   | `conda install -c conda-forge zoxide` |
| [Scoop]         | `scoop install zoxide`                |

</details>

<details>
<summary>BSD</summary>

To install zoxide, use a package manager:

| Distribution  | Repository      | Instructions                    |
| ------------- | --------------- | ------------------------------- |
| ***Any***     | **[crates.io]** | `cargo install zoxide --locked` |
| DragonFly BSD | [DPorts]        | `pkg install zoxide`            |
| FreeBSD       | [FreshPorts]    | `pkg install zoxide`            |
| NetBSD        | [pkgsrc]        | `pkgin install zoxide`          |

</details>

<details>
<summary>Android</summary>

To install zoxide, use a package manager:

| Repository | Instructions         |
| ---------- | -------------------- |
| [Termux]   | `pkg install zoxide` |

</details>

### *Step 2: Add zoxide to your shell*

To start using zoxide, add it to your shell.

<details>
<summary>Bash</summary>

Add this to your configuration (usually `~/.bashrc`):

```sh
eval "$(zoxide init bash)"
```

</details>

<details>
<summary>Elvish</summary>

Add this to your configuration (usually `~/.elvish/rc.elv`):

```sh
eval (zoxide init elvish | slurp)
```

Note: zoxide only supports elvish v0.16.0 and above.

</details>

<details>
<summary>Fish</summary>

Add this to your configuration (usually `~/.config/fish/config.fish`):

```fish
zoxide init fish | source
```

</details>

<details>
<summary>Nushell</summary>

Add this to your configuration (find it by running `config path` in Nushell):

```toml
startup = ["zoxide init nushell --hook prompt | save ~/.zoxide.nu", "source ~/.zoxide.nu"]
```

Note: zoxide only supports Nushell v0.37.0 and above.

</details>

<details>
<summary>PowerShell</summary>

Add this to your configuration (find it by running `echo $profile` in
PowerShell):

```powershell
Invoke-Expression (& {
    $hook = if ($PSVersionTable.PSVersion.Major -lt 6) { 'prompt' } else { 'pwd' }
    (zoxide init --hook $hook powershell) -join "`n"
})
```

</details>

<details>
<summary>Xonsh</summary>

Add this to your configuration (usually `~/.xonshrc`):

```python
execx($(zoxide init xonsh), 'exec', __xonsh__.ctx, filename='zoxide')
```

</details>

<details>
<summary>Zsh</summary>

Add this to your configuration (usually `~/.zshrc`):

```sh
eval "$(zoxide init zsh)"
```

For completions to work, this line must be added _after_ calling `compinit`.

</details>

<details>
<summary>Any POSIX shell</summary>

Add this to your configuration:

```sh
eval "$(zoxide init posix --hook prompt)"
```

</details>

### *Step 3: Install fzf (optional)*

[fzf] is a command-line fuzzy finder, used by zoxide for interactive
selection. It can be installed from [here][fzf-installation].

### *Step 4: Import your data (optional)*

If you currently use any of the following utilities, you may want to import
your data into zoxide:

<details>
<summary>autojump</summary>

```sh
zoxide import --from autojump path/to/db
```

</details>

<details>
<summary>z, z.lua, or zsh-z</summary>

```sh
zoxide import --from z path/to/db
```

</details>

## Configuration

### Flags

When calling `zoxide init`, the following flags are available:

- `--cmd`
  - Changes the prefix of predefined aliases (`z`, `zi`).
  - `--cmd j` would change the aliases to (`j`, `ji`).
  - `--cmd cd` would replace the `cd` command (doesn't work on Nushell / POSIX shells).
- `--hook <HOOK>`
  - Changes how often zoxide increments a directory's score:
    | Hook     | Description                       |
    | -------- | --------------------------------- |
    | `none`   | Never                             |
    | `prompt` | At every shell prompt             |
    | `pwd`    | Whenever the directory is changed |
- `--no-aliases`
  - Don't define aliases (`z`, `zi`).
  - These functions will still be available in your shell as `__zoxide_z` and
    `__zoxide_zi`, should you choose to redefine them.

### Environment variables

Environment variables<sup>[?][wiki-env]</sup> can be used for configuration.
They must be set before `zoxide init` is called.

- `_ZO_DATA_DIR`
  - Specifies the directory in which the database is stored.
  - The default value varies across OSes:
    | OS          | Path                                     | Example                                    |
    | ----------- | ---------------------------------------- | ------------------------------------------ |
    | Linux / BSD | `$XDG_DATA_HOME` or `$HOME/.local/share` | `/home/alice/.local/share`                 |
    | macOS       | `$HOME/Library/Application Support`      | `/Users/Alice/Library/Application Support` |
    | Windows     | `{FOLDERID_RoamingAppData}`              | `C:\Users\Alice\AppData\Roaming`           |
- `_ZO_ECHO`
  - When set to 1, `z` will print the matched directory before navigating to
    it.
- `_ZO_EXCLUDE_DIRS`
  - Excludes the specified directories from the database.
  - This is provided as a list of [globs][glob], separated by OS-specific
    characters:
    | OS                  | Separator | Example                 |
    | ------------------- | --------- | ----------------------- |
    | Linux / macOS / BSD | `:`       | `$HOME:$HOME/private/*` |
    | Windows             | `;`       | `$HOME;$HOME/private/*` |
  - By default, this is set to `"$HOME"`.
- `_ZO_FZF_OPTS`
  - Custom options to pass to [fzf] during interactive selection. See
    [`man fzf`][fzf-man] for the list of options.
- `_ZO_MAXAGE`
  - Configures the [aging algorithm][algorithm-aging], which limits the maximum
    number of entries in the database.
  - By default, this is set to 10000.
- `_ZO_RESOLVE_SYMLINKS`
  - When set to 1, `z` will resolve symlinks before adding directories to the
    database.

## Third-party integrations

| Application        | Description                             | Plugin                     |
| ------------------ | --------------------------------------- | -------------------------- |
| [emacs]            | Text editor                             | [zoxide.el]                |
| [nnn]              | File manager                            | [nnn-autojump]             |
| [ranger]           | File manager                            | [ranger-zoxide]            |
| [telescope.nvim]   | Fuzzy finder for Neovim                 | [telescope-zoxide]         |
| [vim]              | Text editor                             | [zoxide.vim]               |
| [xplr]             | File manager                            | [zoxide.xplr]              |
| [xxh]              | Transports shell configuration over SSH | [xxh-plugin-prerun-zoxide] |
| [zsh-autocomplete] | Realtime completions for zsh            | Supported by default       |

[algorithm-aging]: https://github.com/ajeetdsouza/zoxide/wiki/Algorithm#aging
[algorithm-matching]: https://github.com/ajeetdsouza/zoxide/wiki/Algorithm#matching
[alpine linux packages]: https://pkgs.alpinelinux.org/packages?name=zoxide
[arch linux community]: https://archlinux.org/packages/community/x86_64/zoxide/
[builtwithnix-badge]: https://img.shields.io/badge/builtwith-nix-7d81f7?style=flat-square
[builtwithnix]: https://builtwithnix.org/
[chocolatey]: https://community.chocolatey.org/packages/zoxide
[conda-forge]: https://anaconda.org/conda-forge/zoxide
[copr]: https://copr.fedorainfracloud.org/coprs/atim/zoxide/
[crates.io-badge]: https://img.shields.io/crates/v/zoxide?style=flat-square
[crates.io]: https://crates.io/crates/zoxide
[debian packages]: https://packages.debian.org/stable/admin/zoxide
[devuan packages]: https://pkginfo.devuan.org/cgi-bin/package-query.html?c=package&q=zoxide
[downloads-badge]: https://img.shields.io/github/downloads/ajeetdsouza/zoxide/total?style=flat-square
[dports]: https://github.com/DragonFlyBSD/DPorts/tree/master/sysutils/zoxide
[emacs]: https://www.gnu.org/software/emacs/
[fedora packages]: https://src.fedoraproject.org/rpms/rust-zoxide
[freshports]: https://www.freshports.org/sysutils/zoxide/
[fzf-installation]: https://github.com/junegunn/fzf#installation
[fzf-man]: https://manpages.ubuntu.com/manpages/en/man1/fzf.1.html
[fzf]: https://github.com/junegunn/fzf
[glob]: https://man7.org/linux/man-pages/man7/glob.7.html
[guru overlay]: https://github.com/gentoo-mirror/guru
[homebrew]: https://formulae.brew.sh/formula/zoxide
[issues]: https://github.com/ajeetdsouza/zoxide/issues/new
[linuxbrew]: https://formulae.brew.sh/formula-linux/zoxide
[macports]: https://ports.macports.org/port/zoxide/summary
[neovim]: https://github.com/neovim/neovim
[nixpkgs]: https://github.com/NixOS/nixpkgs/blob/master/pkgs/tools/misc/zoxide/default.nix
[nnn-autojump]: https://github.com/jarun/nnn/blob/master/plugins/autojump
[nnn]: https://github.com/jarun/nnn
[pkgsrc]: https://pkgsrc.se/sysutils/zoxide
[ranger-zoxide]: https://github.com/jchook/ranger-zoxide
[ranger]: https://github.com/ranger/ranger
[raspbian packages]: https://archive.raspbian.org/raspbian/pool/main/r/rust-zoxide/
[releases]: https://github.com/ajeetdsouza/zoxide/releases
[scoop]: https://github.com/ScoopInstaller/Main/tree/master/bucket/zoxide.json
[telescope-zoxide]: https://github.com/jvgrootveld/telescope-zoxide
[telescope.nvim]: https://github.com/nvim-telescope/telescope.nvim
[termux]: https://github.com/termux/termux-packages/tree/master/packages/zoxide
[tutorial]: contrib/tutorial.webp
[ubuntu packages]: https://packages.ubuntu.com/hirsute/zoxide
[vim]: https://github.com/vim/vim
[void linux packages]: https://github.com/void-linux/void-packages/tree/master/srcpkgs/zoxide
[wiki-env]: https://github.com/ajeetdsouza/zoxide/wiki/HOWTO:-set-environment-variables "HOWTO: set environment variables"
[xplr]: https://github.com/sayanarijit/xplr
[xxh-plugin-prerun-zoxide]: https://github.com/xxh/xxh-plugin-prerun-zoxide
[xxh]: https://github.com/xxh/xxh
[zoxide.el]: https://gitlab.com/Vonfry/zoxide.el
[zoxide.vim]: https://github.com/nanotee/zoxide.vim
[zoxide.xplr]: https://github.com/sayanarijit/zoxide.xplr
[zsh-autocomplete]: https://github.com/marlonrichert/zsh-autocomplete
