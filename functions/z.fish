function z
    if test (count $argv) -gt 0
        set _Z_RESULT (zoxide query $argv)
        switch "$_Z_RESULT"
            case 'query: *'
                cd (string sub -s 8 -- "$_Z_RESULT")
            case '*'
                echo "$_Z_RESULT"
        end
        commandline -f repaint
    end
end
