{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/20.09.tar.gz") {} }:

let
  python-pkgs = pkgs.python3.withPackages (pkgs: [ pkgs.black pkgs.mypy pkgs.pylint ]);
in

pkgs.mkShell {
  name = "env";
  buildInputs = [
    pkgs.bash
    pkgs.cargo
    pkgs.cargo-audit
    pkgs.dash
    pkgs.fish
    pkgs.fzf
    pkgs.git
    pkgs.powershell
    pkgs.rustc
    pkgs.shellcheck
    pkgs.shfmt
    pkgs.xonsh
    pkgs.zsh
    python-pkgs
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
