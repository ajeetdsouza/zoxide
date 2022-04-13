#compdef zoxide

autoload -U is-at-least

_zoxide() {
    typeset -A opt_args
    typeset -a _arguments_options
    local ret=1

    if is-at-least 5.2; then
        _arguments_options=(-s -S -C)
    else
        _arguments_options=(-s -C)
    fi

    local context curcontext="$curcontext" state line
    _arguments "${_arguments_options[@]}" \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
":: :_zoxide_commands" \
"*::: :->zoxide" \
&& ret=0
    case $state in
    (zoxide)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:zoxide-command-$line[1]:"
        case $line[1] in
            (add)
_arguments "${_arguments_options[@]}" \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
'*::paths:_files -/' \
&& ret=0
;;
(import)
_arguments "${_arguments_options[@]}" \
'--from=[Application to import from]:FROM:(autojump z)' \
'--merge[Merge into existing database]' \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
':path:_files' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" \
'--cmd=[Changes the prefix of the `z` and `zi` commands]:CMD: ' \
'--hook=[Changes how often zoxide increments a directory'\''s score]:HOOK:(none prompt pwd)' \
'--no-cmd[Prevents zoxide from defining the `z` and `zi` commands]' \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
':shell:(bash elvish fish nushell posix powershell xonsh zsh)' \
&& ret=0
;;
(query)
_arguments "${_arguments_options[@]}" \
'--exclude=[Exclude a path from results]:path:_files -/' \
'--all[Show deleted directories]' \
'(-l --list)-i[Use interactive selection]' \
'(-l --list)--interactive[Use interactive selection]' \
'(-i --interactive)-l[List all matching directories]' \
'(-i --interactive)--list[List all matching directories]' \
'-s[Print score with results]' \
'--score[Print score with results]' \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
'*::keywords:' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" \
'-i[Use interactive selection]' \
'--interactive[Use interactive selection]' \
'-h[Print help information]' \
'--help[Print help information]' \
'-V[Print version information]' \
'--version[Print version information]' \
'*::paths:_files -/' \
&& ret=0
;;
        esac
    ;;
esac
}

(( $+functions[_zoxide_commands] )) ||
_zoxide_commands() {
    local commands; commands=(
'add:Add a new directory or increment its rank' \
'import:Import entries from another application' \
'init:Generate shell configuration' \
'query:Search for a directory in the database' \
'remove:Remove a directory from the database' \
    )
    _describe -t commands 'zoxide commands' commands "$@"
}
(( $+functions[_zoxide__add_commands] )) ||
_zoxide__add_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide add commands' commands "$@"
}
(( $+functions[_zoxide__import_commands] )) ||
_zoxide__import_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide import commands' commands "$@"
}
(( $+functions[_zoxide__init_commands] )) ||
_zoxide__init_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide init commands' commands "$@"
}
(( $+functions[_zoxide__query_commands] )) ||
_zoxide__query_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide query commands' commands "$@"
}
(( $+functions[_zoxide__remove_commands] )) ||
_zoxide__remove_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide remove commands' commands "$@"
}

_zoxide "$@"
