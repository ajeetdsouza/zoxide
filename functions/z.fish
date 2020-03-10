function z
  set -l argc (count $argv)
  if test $argc -gt 0
    if test $argc -eq 1 -a "$argv[1]" = "-"
      cd -
      commandline -f repaint
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
    commandline -f repaint
  end
end

