let
  rust = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/master.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust ]; };
  pkgs-latest = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/94e725d3299cbeb71419758d671dfb0771b8e318.tar.gz") {};
  pkgs-python = pkgs-latest.python3.withPackages (pkgs: [ pkgs.black pkgs.mypy pkgs.pylint ]);
in
pkgs.mkShell {
  buildInputs = [
    # Rust
    pkgs.rust-bin.stable.latest.default

    # Shells
    pkgs-latest.bash
    pkgs-latest.dash
    pkgs-latest.elvish
    pkgs-latest.nushell
    pkgs-latest.fish
    pkgs-latest.powershell
    pkgs-latest.xonsh
    pkgs-latest.zsh

    # Linters
    pkgs-latest.cargo-audit
    pkgs-latest.shellcheck
    pkgs-latest.shfmt
    pkgs-python

    # Dependencies
    pkgs.cacert
    pkgs.libiconv
    pkgs-latest.fzf
    pkgs-latest.git
  ];

  RUST_BACKTRACE = 1;
}
