let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/203dc4fc3fe2a5df1aa481a3fc8a1bb27074d677.tar.gz");
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/eac07edbd20ed4908b98790ba299250b5527ecdf.tar.gz") {
      overlays = [ rust ];
    };
in pkgs.mkShell {
  buildInputs = [
    # Rust
    pkgs.rust-bin.stable.latest.default

    # Shells
    pkgs.bash
    pkgs.dash
    pkgs.elvish
    pkgs.fish
    pkgs.nushell
    pkgs.powershell
    pkgs.xonsh
    pkgs.zsh

    # Tools
    pkgs.cargo-audit
    pkgs.mandoc
    pkgs.nixfmt
    pkgs.nodePackages.markdownlint-cli
    pkgs.python3Packages.black
    pkgs.python3Packages.mypy
    pkgs.python3Packages.pylint
    pkgs.shellcheck
    pkgs.shfmt

    # Dependencies
    pkgs.cacert
    pkgs.fzf
    pkgs.git
    pkgs.libiconv
  ];

  RUST_BACKTRACE = 1;
}
