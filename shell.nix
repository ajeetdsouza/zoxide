let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/ad311f5bb5c5ef475985f1e0f264e831470a8510.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust ]; };
  pkgs-latest = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/3ef1d2a9602c18f8742e1fb63d5ae9867092e3d6.tar.gz")
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
