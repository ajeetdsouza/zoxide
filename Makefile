ifeq ($(CI), true)
	ci_color_always := --color=always
endif

ifeq ($(RELEASE), true)
	build_flags := --release
endif

ifeq ($(OS), Windows_NT)
	NIX := false
else
	NIX := true
endif

.PHONY: build clean install test uninstall

ifeq ($(NIX), true)
build:
	nix-shell --pure --run 'cargo build $(build_flags) $(ci_color_always)'
else
build:
	cargo build $(build_flags) $(ci_color_always)
endif

ifeq ($(NIX), true)
clean:
	nix-shell --pure --run 'cargo clean $(ci_color_always)'
else
clean:
	cargo clean $(ci_color_always)
endif

ifeq ($(NIX), true)
install:
	nix-shell --pure --run 'cargo install --path=. $(ci_color_always)'
else
install:
	cargo install --path=. $(ci_color_always)
endif

ifeq ($(NIX), true)
test:
	nix-shell --pure --run 'cargo fmt -- --check --files-with-diff $(ci_color_always)'
	nix-shell --pure --run 'cargo check --all-features $(build_flags) $(ci_color_always)'
	nix-shell --pure --run 'cargo clippy --all-features $(build_flags) $(ci_color_always) -- --deny warnings --deny clippy::all'
	nix-shell --pure --run 'cargo test --all-features --no-fail-fast $(build_flags) $(ci_color_always)'
	nix-shell --pure --run 'cargo audit --deny warnings $(ci_color_always) --ignore=RUSTSEC-2020-0095'
else
test:
	cargo fmt -- --check --files-with-diff $(ci_color_always)
	cargo check --all-features $(build_flags) $(ci_color_always)
	cargo clippy --all-features $(build_flags) $(ci_color_always) -- --deny warnings --deny clippy::all
	cargo test --no-fail-fast $(build_flags) $(ci_color_always)
	cargo audit --deny warnings $(ci_color_always) --ignore=RUSTSEC-2020-0095
endif

ifeq ($(NIX), true)
uninstall:
	nix-shell --pure --run 'cargo uninstall $(ci_color_always)'
else
uninstall:
	cargo uninstall $(ci_color_always)
endif
