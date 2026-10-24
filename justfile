default:
    @just --list

[unix]
fmt:
    nix-shell --cores 0 --pure --run 'cargo-fmt --all'
    nix-shell --cores 0 --pure --run 'nixfmt -- *.nix'
    nix-shell --cores 0 --pure --run 'shfmt --indent=4 --language-dialect=posix --simplify --write *.sh'
    nix-shell --cores 0 --pure --run 'yamlfmt -- .github/workflows/*.yml'

[windows]
fmt:
    cargo +nightly fmt --all

[unix]
lint:
    nix-shell --cores 0 --pure --run 'cargo-fmt --all --check'
    nix-shell --cores 0 --pure --run 'cargo clippy --all-features --all-targets -- -Dwarnings'
    nix-shell --cores 0 --pure --run 'cargo msrv verify'
    nix-shell --cores 0 --pure --run 'cargo udeps --all-features --all-targets --workspace'
    nix-shell --cores 0 --pure --run 'mandoc -man -Wall -Tlint -- man/man1/*.1'
    nix-shell --cores 0 --pure --run 'markdownlint *.md'
    nix-shell --cores 0 --pure --run 'nixfmt --check -- *.nix'
    nix-shell --cores 0 --pure --run 'shellcheck --enable all *.sh'
    nix-shell --cores 0 --pure --run 'shfmt --diff --indent=4 --language-dialect=posix --simplify *.sh'
    nix-shell --cores 0 --pure --run 'yamlfmt -lint -- .github/workflows/*.yml'

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
