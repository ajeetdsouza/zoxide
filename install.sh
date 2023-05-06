#!/bin/sh
# shellcheck shell=dash

# The official zoxide installer.
#
# It runs on Unix shells like {a,ba,da,k,z}sh. It uses the common `local`
# extension. Note: Most shells limit `local` to 1 var per line, contra bash.

main() {
    if [ "${KSH_VERSION-}" = 'Version JM 93t+ 2010-03-05' ]; then
        # The version of ksh93 that ships with many illumos systems does not
        # support the "local" extension.  Print a message rather than fail in
        # subtle ways later on:
        err 'the installer does not work with this ksh93 version; please try bash'
    fi

    set -u

    # Detect and print host target triple.
    ensure get_architecture
    local _arch="${RETVAL}"
    assert_nz "${_arch}" "arch"
    echo "Detected architecture: ${_arch}"

    # Create and enter a temporary directory.
    local _tmp_dir
    _tmp_dir="$(mktemp -d)" || err "mktemp: could not create temporary directory"
    cd "${_tmp_dir}" || err "cd: failed to enter directory: ${_tmp_dir}"

    # Download and extract zoxide.
    ensure download_zoxide "${_arch}"
    local _package="${RETVAL}"
    assert_nz "${_package}" "package"
    echo "Downloaded package: ${_package}"
    case "${_package}" in
    *.tar.gz)
        need_cmd tar
        ensure tar -xf "${_package}"
        ;;
    *.zip)
        need_cmd unzip
        ensure unzip -oq "${_package}"
        ;;
    *)
        err "unsupported package format: ${_package}"
        ;;
    esac

    # Install binary.
    local _bin_dir="${HOME}/.local/bin"
    local _bin_name
    case "${_arch}" in
    *windows*) _bin_name="zoxide.exe" ;;
    *) _bin_name="zoxide" ;;
    esac
    ensure mkdir -p "${_bin_dir}"
    ensure cp "${_bin_name}" "${_bin_dir}"
    ensure chmod +x "${_bin_dir}/${_bin_name}"
    echo "Installed zoxide to ${_bin_dir}"

    # Install manpages.
    local _man_dir="${HOME}/.local/share/man"
    ensure mkdir -p "${_man_dir}/man1"
    ensure cp "man/man1/"* "${_man_dir}/man1/"
    echo "Installed manpages to ${_man_dir}"

    # Print success message and check $PATH.
    echo ""
    echo "zoxide is installed!"
    if ! echo ":${PATH}:" | grep -Fq ":${_bin_dir}:"; then
        echo "NOTE: ${_bin_dir} is not on your \$PATH. zoxide will not work unless it is added to \$PATH."
    fi
}

download_zoxide() {
    local _arch="$1"

    if check_cmd curl; then
        _dld=curl
    elif check_cmd wget; then
        _dld=wget
    else
        need_cmd 'curl or wget'
    fi
    need_cmd grep

    local _releases_url="https://api.github.com/repos/ajeetdsouza/zoxide/releases/latest"
    local _releases
    case "${_dld}" in
    curl) _releases="$(curl -sL "${_releases_url}")" ||
        err "curl: failed to download ${_releases_url}" ;;
    wget) _releases="$(wget -qO- "${_releases_url}")" ||
        err "wget: failed to download ${_releases_url}" ;;
    *) err "unsupported downloader: ${_dld}" ;;
    esac
    (echo "${_releases}" | grep -q 'API rate limit exceeded') &&
        err "you have exceeded GitHub's API rate limit. Please try again later, or use a different installation method: https://github.com/ajeetdsouza/zoxide/#installation"

    local _package_url
    _package_url="$(echo "${_releases}" | grep "browser_download_url" | cut -d '"' -f 4 | grep "${_arch}")" ||
        err "zoxide has not yet been packaged for your architecture (${_arch}), please file an issue: https://github.com/ajeetdsouza/zoxide/issues"

    local _ext
    case "${_package_url}" in
    *.tar.gz) _ext="tar.gz" ;;
    *.zip) _ext="zip" ;;
    *) err "unsupported package format: ${_package_url}" ;;
    esac

    local _package="zoxide.${_ext}"
    case "${_dld}" in
    curl) _releases="$(curl -sLo "${_package}" "${_package_url}")" || err "curl: failed to download ${_package_url}" ;;
    wget) _releases="$(wget -qO "${_package}" "${_package_url}")" || err "wget: failed to download ${_package_url}" ;;
    *) err "unsupported downloader: ${_dld}" ;;
    esac

    RETVAL="${_package}"
}

# The below functions have been extracted with minor modifications from the
# Rustup install script:
#
#   https://github.com/rust-lang/rustup/blob/4c1289b2c3f3702783900934a38d7c5f912af787/rustup-init.sh

get_architecture() {
    local _ostype _cputype _bitness _arch _clibtype
    _ostype="$(uname -s)"
    _cputype="$(uname -m)"
    _clibtype="musl"

    if [ "${_ostype}" = Linux ]; then
        if [ "$(uname -o || true)" = Android ]; then
            _ostype=Android
        fi
    fi

    if [ "${_ostype}" = Darwin ] && [ "${_cputype}" = i386 ]; then
        # Darwin `uname -m` lies
        if sysctl hw.optional.x86_64 | grep -q ': 1'; then
            _cputype=x86_64
        fi
    fi

    if [ "${_ostype}" = SunOS ]; then
        # Both Solaris and illumos presently announce as "SunOS" in "uname -s"
        # so use "uname -o" to disambiguate.  We use the full path to the
        # system uname in case the user has coreutils uname first in PATH,
        # which has historically sometimes printed the wrong value here.
        if [ "$(/usr/bin/uname -o || true)" = illumos ]; then
            _ostype=illumos
        fi

        # illumos systems have multi-arch userlands, and "uname -m" reports the
        # machine hardware name; e.g., "i86pc" on both 32- and 64-bit x86
        # systems.  Check for the native (widest) instruction set on the
        # running kernel:
        if [ "${_cputype}" = i86pc ]; then
            _cputype="$(isainfo -n)"
        fi
    fi

    case "${_ostype}" in
    Android)
        _ostype=linux-android
        ;;
    Linux)
        check_proc
        _ostype=unknown-linux-${_clibtype}
        _bitness=$(get_bitness)
        ;;
    FreeBSD)
        _ostype=unknown-freebsd
        ;;
    NetBSD)
        _ostype=unknown-netbsd
        ;;
    DragonFly)
        _ostype=unknown-dragonfly
        ;;
    Darwin)
        _ostype=apple-darwin
        ;;
    illumos)
        _ostype=unknown-illumos
        ;;
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        _ostype=pc-windows-msvc
        ;;
    *)
        err "unrecognized OS type: ${_ostype}"
        ;;
    esac

    case "${_cputype}" in
    i386 | i486 | i686 | i786 | x86)
        _cputype=i686
        ;;
    xscale | arm)
        _cputype=arm
        if [ "${_ostype}" = "linux-android" ]; then
            _ostype=linux-androideabi
        fi
        ;;
    armv6l)
        _cputype=arm
        if [ "${_ostype}" = "linux-android" ]; then
            _ostype=linux-androideabi
        else
            _ostype="${_ostype}eabihf"
        fi
        ;;
    armv7l | armv8l)
        _cputype=armv7
        if [ "${_ostype}" = "linux-android" ]; then
            _ostype=linux-androideabi
        else
            _ostype="${_ostype}eabihf"
        fi
        ;;
    aarch64 | arm64)
        _cputype=aarch64
        ;;
    x86_64 | x86-64 | x64 | amd64)
        _cputype=x86_64
        ;;
    mips)
        _cputype=$(get_endianness mips '' el)
        ;;
    mips64)
        if [ "${_bitness}" -eq 64 ]; then
            # only n64 ABI is supported for now
            _ostype="${_ostype}abi64"
            _cputype=$(get_endianness mips64 '' el)
        fi
        ;;
    ppc)
        _cputype=powerpc
        ;;
    ppc64)
        _cputype=powerpc64
        ;;
    ppc64le)
        _cputype=powerpc64le
        ;;
    s390x)
        _cputype=s390x
        ;;
    riscv64)
        _cputype=riscv64gc
        ;;
    *)
        err "unknown CPU type: ${_cputype}"
        ;;
    esac

    # Detect 64-bit linux with 32-bit userland
    if [ "${_ostype}" = unknown-linux-musl ] && [ "${_bitness}" -eq 32 ]; then
        case ${_cputype} in
        x86_64)
            # 32-bit executable for amd64 = x32
            if is_host_amd64_elf; then {
                echo "x32 userland is unsupported" 1>&2
                exit 1
            }; else
                _cputype=i686
            fi
            ;;
        mips64)
            _cputype=$(get_endianness mips '' el)
            ;;
        powerpc64)
            _cputype=powerpc
            ;;
        aarch64)
            _cputype=armv7
            if [ "${_ostype}" = "linux-android" ]; then
                _ostype=linux-androideabi
            else
                _ostype="${_ostype}eabihf"
            fi
            ;;
        riscv64gc)
            err "riscv64 with 32-bit userland unsupported"
            ;;
        *) ;;
        esac
    fi

    # Detect armv7 but without the CPU features Rust needs in that build,
    # and fall back to arm.
    # See https://github.com/rust-lang/rustup.rs/issues/587.
    if [ "${_ostype}" = "unknown-linux-musleabihf" ] && [ "${_cputype}" = armv7 ]; then
        if ensure grep '^Features' /proc/cpuinfo | grep -q -v neon; then
            # At least one processor does not have NEON.
            _cputype=arm
        fi
    fi

    _arch="${_cputype}-${_ostype}"
    RETVAL="${_arch}"
}

get_bitness() {
    need_cmd head
    # Architecture detection without dependencies beyond coreutils.
    # ELF files start out "\x7fELF", and the following byte is
    #   0x01 for 32-bit and
    #   0x02 for 64-bit.
    # The printf builtin on some shells like dash only supports octal
    # escape sequences, so we use those.
    local _current_exe_head
    _current_exe_head=$(head -c 5 /proc/self/exe)
    if [ "${_current_exe_head}" = "$(printf '\177ELF\001')" ]; then
        echo 32
    elif [ "${_current_exe_head}" = "$(printf '\177ELF\002')" ]; then
        echo 64
    else
        err "unknown platform bitness"
    fi
}

get_endianness() {
    local cputype=$1
    local suffix_eb=$2
    local suffix_el=$3

    # detect endianness without od/hexdump, like get_bitness() does.
    need_cmd head
    need_cmd tail

    local _current_exe_endianness
    _current_exe_endianness="$(head -c 6 /proc/self/exe | tail -c 1)"
    if [ "${_current_exe_endianness}" = "$(printf '\001')" ]; then
        echo "${cputype}${suffix_el}"
    elif [ "${_current_exe_endianness}" = "$(printf '\002')" ]; then
        echo "${cputype}${suffix_eb}"
    else
        err "unknown platform endianness"
    fi
}

is_host_amd64_elf() {
    need_cmd head
    need_cmd tail
    # ELF e_machine detection without dependencies beyond coreutils.
    # Two-byte field at offset 0x12 indicates the CPU,
    # but we're interested in it being 0x3E to indicate amd64, or not that.
    local _current_exe_machine
    _current_exe_machine=$(head -c 19 /proc/self/exe | tail -c 1)
    [ "${_current_exe_machine}" = "$(printf '\076')" ]
}

check_proc() {
    # Check for /proc by looking for the /proc/self/exe link.
    # This is only run on Linux.
    if ! test -L /proc/self/exe; then
        err "unable to find /proc/self/exe. Is /proc mounted? Installation cannot proceed without /proc."
    fi
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
    fi
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

# Run a command that should never fail. If the command fails execution
# will immediately terminate with an error showing the failing
# command.
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}

assert_nz() {
    if [ -z "$1" ]; then err "found empty string: $2"; fi
}

err() {
    echo "Error: $1" >&2
    exit 1
}

# This is put in braces to ensure that the script does not run until it is
# downloaded completely.
{
    main "$@" || exit 1
}
