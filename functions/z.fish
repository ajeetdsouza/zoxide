function z
  if test (count $argv) -gt 0
    if test "$argv[1]" = "-"
      cd -
    else
      set _Z_RESULT (zoxide query $argv)
      switch "$_Z_RESULT"
        case 'query: *'
          cd (string sub -s 8 -- "$_Z_RESULT")
          commandline -f repaint
        case '*'
          echo -n "$_Z_RESULT"
      end
    end
  else
    cd ~
  end
end
