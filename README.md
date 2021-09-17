<!-- markdownlint-configure-file {
  "MD013": {
    "code_blocks": false,
    "tables": false
  },
  "MD033": false
} -->

# `zoxide`

> A smarter cd command for your terminal

[![crates.io][crates.io-badge]][crates.io]
[![Downloads][downloads-badge]][releases]
[![Built with Nix][builtwithnix-badge]][builtwithnix]

`zoxide` is a blazing fast replacement for your `cd` command, inspired by
`z` and `autojump`. It keeps track of the directories you use most
frequently, and uses a ranking algorithm to navigate to the best match.

![Tutorial][tutorial]

## Examples

```sh
z foo        # cd into highest ranked directory matching foo
z foo bar    # cd into highest ranked directory matching foo and bar

z ~/foo      # z also works like a regular cd command
z foo/       # cd into relative path
z ..         # cd one level up
z -          # cd into previous directory

zi foo       # cd with interactive selection (using fzf)
```

Read more about the matching algorithm [here][algorithm-matching].

## Getting started

### *Step 1: Install `zoxide`*

`zoxide` runs on most major platforms. If your platform isn't listed below,
please [open an issue][issues].

<details>
<summary>Linux</summary>

To install `zoxide`, run this command in your terminal:

```sh
curl -sS https://webinstall.dev/zoxide | bash
```

Alternatively, you can use a package manager:

| Distribution       | Repository              | Instructions                                                                                   |
| ------------------ | ----------------------- | ---------------------------------------------------------------------------------------------- |
| ***Any***          | **[crates.io]**         | `cargo install zoxide`                                                                         |
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
| Raspbian           |                         | `apt install zoxide`                                                                           |
| Ubuntu 21.04+      | [Ubuntu Packages]       | `apt install zoxide`                                                                           |
| Void Linux         | [Void Linux Packages]   | `xbps-install -S zoxide`                                                                       |

</details>

<details>
<summary>macOS</summary>

To install `zoxide`, use a package manager:

| Repository      | Instructions                          |
| --------------- | ------------------------------------- |
| **[crates.io]** | `cargo install zoxide`                |
| [conda-forge]   | `conda install -c conda-forge zoxide` |
| [Homebrew]      | `brew install zoxide`                 |
| [MacPorts]      | `port install zoxide`                 |

</details>

<details>
<summary>Windows</summary>

To install `zoxide`, run this command in your command prompt:

```sh
curl.exe -A "MS" https://webinstall.dev/zoxide | powershell
```

Alternatively, you can use a package manager:

| Repository      | Instructions                          |
| --------------- | ------------------------------------- |
| **[crates.io]** | `cargo install zoxide`                |
| [Chocolatey]    | `choco install zoxide`                |
| [conda-forge]   | `conda install -c conda-forge zoxide` |
| [Scoop]         | `scoop install zoxide`                |

</details>

<details>
<summary>BSD</summary>

To install `zoxide`, use a package manager:

| Distribution  | Repository      | Instructions           |
| ------------- | --------------- | ---------------------- |
| ***Any***     | **[crates.io]** | `cargo install zoxide` |
| DragonFly BSD | [DPorts]        | `pkg install zoxide`   |
| FreeBSD       | [FreshPorts]    | `pkg install zoxide`   |
| NetBSD        | [pkgsrc]        | `pkgin install zoxide` |

</details>

<details>
<summary>Android</summary>

To install `zoxide`, use a package manager:

| Repository | Instructions         |
| ---------- | -------------------- |
| [Termux]   | `pkg install zoxide` |

</details>

### *Step 2: Add `zoxide` to your shell*

To start using `zoxide`, add it to your shell.

<details>
<summary><code>bash</code></summary>

Add this to your configuration (usually `~/.bashrc`):

```sh
eval "$(zoxide init bash)"
```

</details>

<details>
<summary><code>elvish</code></summary>

Add this to your configuration (usually `~/.elvish/rc.elv`):

```sh
eval (zoxide init elvish | slurp)
```

Note: zoxide only supports elvish v0.16.0 and above.

</details>

<details>
<summary><code>fish</code></summary>

Add this to your configuration (usually `~/.config/fish/config.fish`):

```fish
zoxide init fish | source
```

</details>

<details>
<summary><code>nushell</code></summary>

Add this to your configuration (find it by running `config path` in Nushell):

```toml
startup = ["zoxide init nushell --hook prompt | save ~/.zoxide.nu", "source ~/.zoxide.nu"]
```

Note: zoxide only supports Nushell v0.37.0 and above.

</details>

<details>
<summary><code>powershell</code></summary>

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
<summary><code>xonsh</code></summary>

Add this to your configuration (usually `~/.xonshrc`):

```python
execx($(zoxide init xonsh), 'exec', __xonsh__.ctx, filename='zoxide')
```

</details>

<details>
<summary><code>zsh</code></summary>

Add this to your configuration (usually `~/.zshrc`):

```sh
eval "$(zoxide init zsh)"
```

</details>

<details>
<summary>any POSIX shell</summary>

Add this to your configuration:

```sh
eval "$(zoxide init posix --hook prompt)"
```

</details>

### *Step 3: Install `fzf` (optional)*

[`fzf`][fzf] is a command-line fuzzy finder, used by `zoxide` for interactive
selection. It can be installed from [here][fzf-installation].

### *Step 4: Import your data (optional)*

If you currently use any of the following utilities, you may want to import
your data into `zoxide`:

<details>
<summary><code>autojump</code></summary>

```sh
zoxide import --from autojump path/to/db
```

</details>

<details>
<summary><code>z</code>, <code>z.lua</code>, or <code>zsh-z</code></summary>

```sh
zoxide import --from z path/to/db
```

</details>

## Configuration

### Flags

When calling `zoxide init`, the following flags are available:

- `--cmd`
  - Changes the prefix of predefined aliases (`z`, `zi`).
  - e.g. `--cmd j` would change the aliases to `j` and `ji` respectively.
- `--hook <HOOK>`
  - Changes how often `zoxide` increments a directory's score:
    | Hook     | Description                       |
    | -------- | --------------------------------- |
    | `none`   | Never                             |
    | `prompt` | At every shell prompt             |
    | `pwd`    | Whenever the directory is changed |
- `--no-aliases`
  - Don't define extra aliases (`z`, `zi`).
  - These functions will still be available in your shell as `__zoxide_z` and
    `__zoxide_zi`, should you choose to redefine them.

### Environment variables

Be sure to set these before calling `zoxide init`.

- `_ZO_DATA_DIR`
  - Specifies the directory in which `zoxide` should store its database.
  - The default value varies across OSes:
    | OS          | Path                                     | Example                                    |
    | ----------- | ---------------------------------------- | ------------------------------------------ |
    | Linux / BSD | `$XDG_DATA_HOME` or `$HOME/.local/share` | `/home/alice/.local/share`                 |
    | macOS       | `$HOME/Library/Application Support`      | `/Users/Alice/Library/Application Support` |
    | Windows     | `{FOLDERID_RoamingAppData}`              | `C:\Users\Alice\AppData\Roaming`           |
- `_ZO_ECHO`
  - When set to `1`, `z` will print the matched directory before navigating to
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
  - Custom options to pass to [`fzf`][fzf]. See `man fzf` for the list of
    options.
- `_ZO_MAXAGE`
  - Configures the [aging algorithm][algorithm-aging], which limits the maximum
    number of entries in the database.
  - By default, this is set to `10000`.
- `_ZO_RESOLVE_SYMLINKS`
  - When set to `1`, `z` will resolve symlinks before adding directories to the
    database.

## Third-party integrations

- [`emacs`][emacs]. You can use `zoxide` for navigation with the
  [`zoxide.el`][zoxide-el] plugin.
- [`nnn`][nnn] is a terminal file manager. You can use `zoxide` for navigation
  with the official [`autojump`][nnn-autojump] plugin.
- [`ranger`][ranger] is a terminal file manager. You can use `zoxide` for
  navigation with the [`ranger-zoxide`][ranger-zoxide] plugin.
- [`telescope.nvim`][telescope-nvim] is a fuzzy finder for `neovim`. You can
  use it with `zoxide` via the [`telescope-zoxide`][telescope-zoxide] plugin.
- [`vim`][vim] / [`neovim`][neovim]. You can use `zoxide` for navigation with
  the [`zoxide.vim`][zoxide-vim] plugin.
- [`xxh`][xxh] transports your shell configuration over SSH. You can use
  `zoxide` over SSH via the [`xxh-plugin-prerun-zoxide`][xxh-zoxide] plugin.
- [`zsh-autocomplete`][zsh-autocomplete] adds realtime completions to `zsh`. It
  supports `zoxide` out of the box.

[algorithm-aging]: https://github.com/ajeetdsouza/zoxide/wiki/Algorithm#aging
[algorithm-matching]: https://github.com/ajeetdsouza/zoxide/wiki/Algorithm#matching
[alpine linux packages]: https://pkgs.alpinelinux.org/packages?name=zoxide
[arch linux community]: https://archlinux.org/packages/community/x86_64/zoxide/
[builtwithnix-badge]: https://img.shields.io/badge/builtwith-nix-7d81f7
[builtwithnix]: https://builtwithnix.org/
[chocolatey]: https://community.chocolatey.org/packages/zoxide
[conda-forge]: https://anaconda.org/conda-forge/zoxide
[copr]: https://copr.fedorainfracloud.org/coprs/atim/zoxide/
[crates.io-badge]: https://img.shields.io/crates/v/zoxide
[crates.io]: https://crates.io/crates/zoxide
[debian packages]: https://packages.debian.org/stable/admin/zoxide
[devuan packages]: https://pkginfo.devuan.org/cgi-bin/package-query.html?c=package&q=zoxide
[downloads-badge]: https://img.shields.io/github/downloads/ajeetdsouza/zoxide/total
[dports]: https://github.com/DragonFlyBSD/DPorts/tree/master/sysutils/zoxide
[emacs]: https://www.gnu.org/software/emacs/
[fedora packages]: https://src.fedoraproject.org/rpms/rust-zoxide
[freshports]: https://www.freshports.org/sysutils/zoxide/
[fzf-installation]: https://github.com/junegunn/fzf#installation
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
[releases]: https://github.com/ajeetdsouza/zoxide/releases
[scoop]: https://github.com/ScoopInstaller/Main/tree/master/bucket/zoxide.json
[telescope-nvim]: https://github.com/nvim-telescope/telescope.nvim
[telescope-zoxide]: https://github.com/jvgrootveld/telescope-zoxide
[termux]: https://github.com/termux/termux-packages/tree/master/packages/zoxide
[tutorial]: contrib/tutorial.webp
[ubuntu packages]: https://packages.ubuntu.com/hirsute/zoxide
[vim]: https://github.com/vim/vim
[void linux packages]: https://github.com/void-linux/void-packages/tree/master/srcpkgs/zoxide
[xxh-zoxide]: https://github.com/xxh/xxh-plugin-prerun-zoxide
[xxh]: https://github.com/xxh/xxh
[zoxide-el]: https://gitlab.com/Vonfry/zoxide.el
[zoxide-vim]: https://github.com/nanotee/zoxide.vim
[zsh-autocomplete]: https://github.com/marlonrichert/zsh-autocomplete
