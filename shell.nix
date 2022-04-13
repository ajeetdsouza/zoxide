let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/80868ba348db2b5e733483454cbb5770f2951077.tar.gz");
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/d8cec85ca62e9f6c1576a8d87ebd17350710b449.tar.gz") {
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
