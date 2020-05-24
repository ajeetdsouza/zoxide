#!/bin/sh

set -u

usage() {
	cat 1>&2 <<EOF
The installer for zoxide

USAGE
	zoxide-install
EOF
}

main() {
	need_cmd uname
	need_cmd curl
	need_cmd grep
	need_cmd cut
	need_cmd xargs
	need_cmd chmod
	install
}

install() {
	local _ostype _cputype _clibtype _target _cargobuild _install_path
	_ostype="$(uname -s)"
	_cputype="$(uname -m)"
	_cargobuild="no"
	_install_path="/usr/local/bin"

	case $_cputype in
	x86_64 | x86-64 | amd64)
		_cputype="x86_64"
		_clibtype="musl"
		;;
	*)
		warning "No binaries are available for your CPU architecture ($_cputype)"
		_clibtype="gnu"
		_cargobuild="yes"
		;;
	esac

	case $_ostype in
	Linux)
		_ostype=unknown-linux-$_clibtype
		;;
	Darwin)
		_ostype=apple-darwin
		;;
	*)
		warning "No binaries are available for your operating system ($_ostype)"
		_cargobuild="yes"
		;;
	esac

	if [ $_cargobuild = "yes" ]; then
		cargo_build
		exit 0
	fi

	_target="$_cputype-$_ostype"
	warning "Detected target: $_target"

	success "Downloading zoxide..."

	## Downloading the binaries
	ensure rm -rf "zoxide-$_target"
	curl -s https://api.github.com/repos/ajeetdsouza/zoxide/releases/latest | grep "browser_download_url" | cut -d '"' -f 4 | grep "$_target" | xargs -n 1 curl -LJO

	ensure mv "zoxide-$_target" "zoxide_bin"

	ensure sudo mv zoxide_bin "$_install_path/zoxide"
	ensure chmod +x "$_install_path/zoxide"

	success "zoxide is installed!"
	info "Please ensure that $_install_path is added to your \$PATH."
}

success() {
	printf "\033[32m%s\033[0m\n" "$1" >&1
}

info() {
	printf "%s\n" "$1" >&1
}

warning() {
	printf "\033[33m%s\033[0m\n" "$1" >&2
}

error() {
	printf "\033[31;1m%s\033[0m\n" "$1" >&2
	exit 1
}

cmd_chk() {
	command -v "$1" >/dev/null 2>&1
}

## Ensures that the command executes without error
ensure() {
	if ! "$@"; then error "command failed: $*"; fi
}

need_cmd() {
	if ! cmd_chk "$1"; then
		error "need $1 (command not found)"
	fi
}

prompt_confirm() {
	if [ ! -t 1 ]; then
		error "Unable to run interactively. Please execute this script using interactive shell"
	fi

	while true; do
		read -rp "Is this okay? (y/N): " _choice
		_choice=$(echo "$_choice" | tr '[:upper:]' '[:lower:]')

		case "$_choice" in
		y | yes) break ;;
		n | no) error "Operation aborted" ;;
		esac
	done
}

cargo_build() {
	success "Compiling from source..."

	if ! cmd_chk "cargo"; then
		success "Cargo will be installed."
		prompt_confirm

		ensure curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

		# shellcheck source=/dev/null
		. "$HOME/.cargo/env"
	fi

	RUSTFLAGS="-C target-cpu=native" ensure cargo install zoxide
}

main "$@" || exit 1
