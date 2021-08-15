let
  rust = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/ad311f5bb5c5ef475985f1e0f264e831470a8510.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust ]; };
  pkgs-latest = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/1359293549af4f8ca536716d0432f3cdd0afe0c6.tar.gz") {};
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
