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
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
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
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
'*::paths:_files -/' \
&& ret=0
;;
(edit)
_arguments "${_arguments_options[@]}" \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
":: :_zoxide__edit_commands" \
"*::: :->edit" \
&& ret=0

    case $state in
    (edit)
        words=($line[1] "${words[@]}")
        (( CURRENT += 1 ))
        curcontext="${curcontext%:*:*}:zoxide-edit-command-$line[1]:"
        case $line[1] in
            (decrement)
_arguments "${_arguments_options[@]}" \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
':path:' \
&& ret=0
;;
(delete)
_arguments "${_arguments_options[@]}" \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
':path:' \
&& ret=0
;;
(increment)
_arguments "${_arguments_options[@]}" \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
':path:' \
&& ret=0
;;
(reload)
_arguments "${_arguments_options[@]}" \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
&& ret=0
;;
        esac
    ;;
esac
;;
(import)
_arguments "${_arguments_options[@]}" \
'--from=[Application to import from]:FROM:(autojump z)' \
'--merge[Merge into existing database]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
':path:_files' \
&& ret=0
;;
(init)
_arguments "${_arguments_options[@]}" \
'--cmd=[Changes the prefix of the \`z\` and \`zi\` commands]:CMD: ' \
'--hook=[Changes how often zoxide increments a directory'\''s score]:HOOK:(none prompt pwd)' \
'--no-cmd[Prevents zoxide from defining the \`z\` and \`zi\` commands]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
':shell:(bash elvish fish nushell posix powershell xonsh zsh)' \
&& ret=0
;;
(query)
_arguments "${_arguments_options[@]}" \
'--exclude=[Exclude the current directory]:path:_files -/' \
'-a[Show unavailable directories]' \
'--all[Show unavailable directories]' \
'(-l --list)-i[Use interactive selection]' \
'(-l --list)--interactive[Use interactive selection]' \
'(-i --interactive)-l[List all matching directories]' \
'(-i --interactive)--list[List all matching directories]' \
'-s[Print score with results]' \
'--score[Print score with results]' \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
'*::keywords:' \
&& ret=0
;;
(remove)
_arguments "${_arguments_options[@]}" \
'-h[Print help]' \
'--help[Print help]' \
'-V[Print version]' \
'--version[Print version]' \
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
'edit:Edit the database' \
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
(( $+functions[_zoxide__edit__decrement_commands] )) ||
_zoxide__edit__decrement_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide edit decrement commands' commands "$@"
}
(( $+functions[_zoxide__edit__delete_commands] )) ||
_zoxide__edit__delete_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide edit delete commands' commands "$@"
}
(( $+functions[_zoxide__edit_commands] )) ||
_zoxide__edit_commands() {
    local commands; commands=(
'decrement:' \
'delete:' \
'increment:' \
'reload:' \
    )
    _describe -t commands 'zoxide edit commands' commands "$@"
}
(( $+functions[_zoxide__import_commands] )) ||
_zoxide__import_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide import commands' commands "$@"
}
(( $+functions[_zoxide__edit__increment_commands] )) ||
_zoxide__edit__increment_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide edit increment commands' commands "$@"
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
(( $+functions[_zoxide__edit__reload_commands] )) ||
_zoxide__edit__reload_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide edit reload commands' commands "$@"
}
(( $+functions[_zoxide__remove_commands] )) ||
_zoxide__remove_commands() {
    local commands; commands=()
    _describe -t commands 'zoxide remove commands' commands "$@"
}

if [ "$funcstack[1]" = "_zoxide" ]; then
    _zoxide "$@"
else
    compdef _zoxide zoxide
fi
