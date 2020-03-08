function zoxide-add --on-event fish_prompt
    zoxide add
end

set -q ZOXIDE_KEY_BINDINGS
or set -U ZOXIDE_KEY_BINDINGS 0

abbr -a zi "z -i"
abbr -a za "zoxide add"
abbr -a zq "zoxide query"
abbr -a zr "zoxide remove"

if set -q ZOXIDE_KEY_BINDINGS; and test "$ZOXIDE_KEY_BINDINGS" -eq 1
    bind \ez 'z -i'
    if bind -M insert >/dev/null 2>/dev/null
        bind -M insert \ez 'z -i'
    end
end
