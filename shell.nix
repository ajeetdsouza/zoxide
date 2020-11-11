{ pkgs ? import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/20.09.tar.gz") {} }:

let
  my-python3-pkgs = python3-pkgs: with python3-pkgs; [
    black
    mypy
    pylint
  ];
  my-python3 = pkgs.python3.withPackages my-python3-pkgs;
in

pkgs.mkShell {
  name = "env";
  nativeBuildInputs = [
    pkgs.rustc
    pkgs.cargo
  ];
  buildInputs = [
    pkgs.bash
    pkgs.dash
    pkgs.fish
    pkgs.fzf
    pkgs.git
    pkgs.powershell
    pkgs.shellcheck
    pkgs.shfmt
    pkgs.xonsh
    pkgs.zsh
    my-python3
  ];

  # Set Environment Variables
  RUST_BACKTRACE = 1;
}
