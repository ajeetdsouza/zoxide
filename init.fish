if command -sq zoxide
    zoxide init fish | source
else
    echo 'zoxide: command not found, please install it from https://github.com/ajeetdsouza/zoxide'
end
