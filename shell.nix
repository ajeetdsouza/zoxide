let
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/4d513ab5f170d66afa3387bdd718d41aa936ee9f.tar.gz") {
      overlays = [ rust ];
    };
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/ab150c7412db7bea5879ce2776718f53fba37aa2.tar.gz");

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
    pkgs.ksh
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
