let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/6ca3338755233ddcb9bd4c80ecb17f453cfd0730.tar.gz");
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/5d7c1f912f864e56de88c6f81a93173d12878f1e.tar.gz") {
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
