let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/ad311f5bb5c5ef475985f1e0f264e831470a8510.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust ]; };
  pkgs-latest = import (fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/b4692e4197869c42c46d77e31af7e687e1892f55.tar.gz")
    { };
in pkgs.mkShell {
  buildInputs = [
    # Rust
    pkgs.rust-bin.stable.latest.default

    # Shells
    pkgs-latest.elvish
    pkgs-latest.fish
    pkgs-latest.nushell
    pkgs-latest.xonsh
    pkgs.bash
    pkgs.dash
    pkgs.powershell
    pkgs.zsh

    # Tools
    pkgs-latest.cargo-audit
    pkgs-latest.mandoc
    pkgs-latest.nixfmt
    pkgs-latest.nodePackages.markdownlint-cli
    pkgs-latest.python3Packages.black
    pkgs-latest.python3Packages.mypy
    pkgs-latest.python3Packages.pylint
    pkgs-latest.shellcheck
    pkgs-latest.shfmt

    # Dependencies
    pkgs.cacert
    pkgs.libiconv
    pkgs.fzf
    pkgs.git
  ];

  RUST_BACKTRACE = 1;
}
