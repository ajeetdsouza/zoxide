function z
    if test (count $argv) -gt 0
        set _Z_RESULT (zoxide query $argv)
        switch "$_Z_RESULT"
            case 'query: *'
                cd (string sub -s 8 -- "$_Z_RESULT")
                commandline -f repaint
            case '*'
                if test -n "$_Z_RESULT"
                    echo "$_Z_RESULT"
                end
        end
    end
end

