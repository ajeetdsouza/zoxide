let
  pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/20.09.tar.gz") {};
  pkgsMaster = import (fetchTarball "https://github.com/NixOS/nixpkgs/archive/470e4a9bbc98b171a7e733dfc9e62531f7b9ceca.tar.gz") {};
  pkgsPython = pkgs.python3.withPackages (pkgs: [ pkgs.black pkgs.mypy pkgs.pylint ]);
in
pkgs.mkShell
  {
    name = "env";
    buildInputs = [
      pkgsMaster.nushell
      pkgsPython
      pkgs.bash
      pkgs.cargo
      pkgs.cargo-audit
      pkgs.dash
      pkgs.fish
      pkgs.fzf
      pkgs.git
      pkgs.powershell
      pkgs.rustc
      pkgs.rustfmt
      pkgs.shellcheck
      pkgs.shfmt
      pkgs.xonsh
      pkgs.zsh
    ];

    RUST_BACKTRACE = 1;
  }
