default:
    @just --list

[unix]
fmt:
    nix-shell --cores 0 --pure --run 'cargo-fmt --all'
    nix-shell --cores 0 --pure --run 'nixfmt -- *.nix'

[windows]
fmt:
    cargo +nightly fmt --all

[unix]
lint:
    nix-shell --cores 0 --pure --run 'cargo-fmt --all --check'
    nix-shell --cores 0 --pure --run 'cargo clippy --all-features --all-targets -- -Dwarnings'
    nix-shell --cores 0 --pure --run 'nixfmt --check -- ./*.nix'
    nix-shell --cores 0 --pure --run 'markdownlint ./*.md'
    nix-shell --cores 0 --pure --run 'mandoc -man -Wall -Tlint -- ./man/man1/*.1'
    nix-shell --cores 0 --pure --run 'cargo msrv verify'
    nix-shell --cores 0 --pure --run 'cargo udeps --all-features --all-targets --workspace'

[windows]
lint:
    cargo +nightly fmt --all --check
    cargo +stable clippy --all-features --all-targets -- -Dwarnings

[unix]
test *args:
    nix-shell --cores 0 --pure --run 'cargo nextest run --all-features --no-fail-fast --workspace {{args}}'

[windows]
test *args:
    cargo +stable test --no-fail-fast --workspace {{args}}
