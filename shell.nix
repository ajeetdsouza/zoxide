let
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/20.09.tar.gz") {};
  pkgs-master = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/ebe28430ba2d6d0e5562bf69c4afe907645fac61.tar.gz") {};
  pkgs-python = pkgs-master.python3.withPackages (pkgs: [ pkgs.black pkgs.mypy pkgs.pylint ]);
in
pkgs.mkShell {
  buildInputs = [
    pkgs-master.cargo-audit
    pkgs-master.elvish
    pkgs-master.nushell
    pkgs-master.shellcheck
    pkgs-master.shfmt
    pkgs-python
    pkgs.bash
    pkgs.cargo
    pkgs.clippy
    pkgs.dash
    pkgs.fish
    pkgs.fzf
    pkgs.git
    pkgs.powershell
    pkgs.rustc
    pkgs.rustfmt
    pkgs.xonsh
    pkgs.zsh
  ];
  RUST_BACKTRACE = 1;
}
