if (( $+commands[zoxide] )); then
  eval "$(zoxide init zsh)"
else
  echo 'zoxide: command not found, please install it from https://github.com/ajeetdsouza/zoxide'
fi
