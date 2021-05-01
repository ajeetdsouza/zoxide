ifeq ($(CI), true)
	ci_color_always := --color=always
endif

ifeq ($(OS), Windows_NT)
	NIX := false
else
	NIX := true
endif

.PHONY: build clean install test uninstall

build:
	cargo build --release $(ci_color_always)

clean:
	cargo clean $(ci_color_always)

install:
	cargo install --path=. $(ci_color_always)

ifeq ($(NIX), true)
test:
	nix-shell --pure --run 'cargo fmt -- --check --files-with-diff $(ci_color_always)'
	nix-shell --pure --run 'cargo check --all-features --release $(ci_color_always)'
	nix-shell --pure --run 'cargo clippy --all-features --release $(ci_color_always) -- --deny warnings --deny clippy::all'
	nix-shell --pure --run 'cargo test --all-features --no-fail-fast --release $(ci_color_always)'
	nix-shell --pure --run 'cargo audit --deny warnings $(ci_color_always) --ignore=RUSTSEC-2020-0095'
else
test:
	cargo fmt -- --check --files-with-diff $(ci_color_always)
	cargo check --all-features --release $(ci_color_always)
	cargo clippy --all-features --release $(ci_color_always) -- --deny warnings --deny clippy::all
	cargo test --no-fail-fast --release $(ci_color_always)
	cargo audit --deny warnings $(ci_color_always) --ignore=RUSTSEC-2020-0095
endif

uninstall:
	cargo uninstall $(ci_color_always)
