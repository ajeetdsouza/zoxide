let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/9096306d4a1c3adcc8d20f2c9dcaee3dee30d1ad.tar.gz");
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/5f902ae769594aaeaf326e8623a48482eeacfe89.tar.gz") {
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
    pkgs.cargo-nextest
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

  CARGO_INCREMENTAL = builtins.getEnv "CI" != "";
  CARGO_TARGET_DIR = "target_nix";
}
