let
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/22a6958f46fd8e14830d02856ff63b1d0e5cc3e4.tar.gz") {
      overlays = [ rust ];
    };
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/a61fcd9910229d097ffef92b5a2440065e3b64d5.tar.gz");

  rust-nightly =
    pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal);
  cargo-udeps = pkgs.writeShellScriptBin "cargo-udeps" ''
    export RUSTC="${rust-nightly}/bin/rustc";
    export CARGO="${rust-nightly}/bin/cargo";
    exec "${pkgs.cargo-udeps}/bin/cargo-udeps" "$@"
  '';
in pkgs.mkShell {
  buildInputs = [
    # Rust
    (pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.rustfmt))
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
    cargo-udeps
    pkgs.cargo-msrv
    pkgs.cargo-nextest
    pkgs.cargo-udeps
    pkgs.just
    pkgs.mandoc
    pkgs.nixfmt
    pkgs.nodePackages.markdownlint-cli
    pkgs.python3Packages.black
    pkgs.python3Packages.mypy
    pkgs.python3Packages.pylint
    pkgs.shellcheck
    pkgs.shfmt
    pkgs.yamlfmt

    # Dependencies
    pkgs.cacert
    pkgs.fzf
    pkgs.git
    pkgs.libiconv
  ];

  CARGO_TARGET_DIR = "target_nix";
}
