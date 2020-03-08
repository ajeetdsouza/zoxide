function zoxide-add --on-event fish_prompt
    if command -q zoxide
        zoxide add
    end
end

if command -q zoxide
    abbr -a zi "z -i"
    abbr -a za "zoxide add"
    abbr -a zq "zoxide query"
    abbr -a zr "zoxide remove"

    bind \ez 'z -i'
end
