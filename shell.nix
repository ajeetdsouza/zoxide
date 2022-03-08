let
  rust = import (builtins.fetchTarball
    "https://github.com/oxalica/rust-overlay/archive/46d8d20fce510c6a25fa66f36e31f207f6ea49e4.tar.gz");
  pkgs = import (builtins.fetchTarball
    "https://github.com/NixOS/nixpkgs/archive/fae46e66a5df220327b45e0d7c27c6961cf922ce.tar.gz") {
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
