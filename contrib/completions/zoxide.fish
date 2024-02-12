complete -c zoxide -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_use_subcommand" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_use_subcommand" -f -a "add" -d 'Add a new directory or increment its rank'
complete -c zoxide -n "__fish_use_subcommand" -f -a "edit" -d 'Edit the database'
complete -c zoxide -n "__fish_use_subcommand" -f -a "import" -d 'Import entries from another application'
complete -c zoxide -n "__fish_use_subcommand" -f -a "init" -d 'Generate shell configuration'
complete -c zoxide -n "__fish_use_subcommand" -f -a "query" -d 'Search for a directory in the database'
complete -c zoxide -n "__fish_use_subcommand" -f -a "remove" -d 'Remove a directory from the database'
complete -c zoxide -n "__fish_seen_subcommand_from add" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from add" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from decrement; and not __fish_seen_subcommand_from delete; and not __fish_seen_subcommand_from increment; and not __fish_seen_subcommand_from reload" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from decrement; and not __fish_seen_subcommand_from delete; and not __fish_seen_subcommand_from increment; and not __fish_seen_subcommand_from reload" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from decrement; and not __fish_seen_subcommand_from delete; and not __fish_seen_subcommand_from increment; and not __fish_seen_subcommand_from reload" -f -a "decrement"
complete -c zoxide -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from decrement; and not __fish_seen_subcommand_from delete; and not __fish_seen_subcommand_from increment; and not __fish_seen_subcommand_from reload" -f -a "delete"
complete -c zoxide -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from decrement; and not __fish_seen_subcommand_from delete; and not __fish_seen_subcommand_from increment; and not __fish_seen_subcommand_from reload" -f -a "increment"
complete -c zoxide -n "__fish_seen_subcommand_from edit; and not __fish_seen_subcommand_from decrement; and not __fish_seen_subcommand_from delete; and not __fish_seen_subcommand_from increment; and not __fish_seen_subcommand_from reload" -f -a "reload"
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from decrement" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from decrement" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from delete" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from increment" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from increment" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from reload" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from edit; and __fish_seen_subcommand_from reload" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from import" -l from -d 'Application to import from' -r -f -a "{autojump	'',z	''}"
complete -c zoxide -n "__fish_seen_subcommand_from import" -l merge -d 'Merge into existing database'
complete -c zoxide -n "__fish_seen_subcommand_from import" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from import" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from init" -l cmd -d 'Changes the prefix of the `z` and `zi` commands' -r
complete -c zoxide -n "__fish_seen_subcommand_from init" -l hook -d 'Changes how often zoxide increments a directory\'s score' -r -f -a "{none	'',prompt	'',pwd	''}"
complete -c zoxide -n "__fish_seen_subcommand_from init" -l no-cmd -d 'Prevents zoxide from defining the `z` and `zi` commands'
complete -c zoxide -n "__fish_seen_subcommand_from init" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from init" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from query" -l exclude -d 'Exclude the current directory' -r -f -a "(__fish_complete_directories)"
complete -c zoxide -n "__fish_seen_subcommand_from query" -s a -l all -d 'Show unavailable directories'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s i -l interactive -d 'Use interactive selection'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s l -l list -d 'List all matching directories'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s s -l score -d 'Print score with results'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_seen_subcommand_from remove" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_seen_subcommand_from remove" -s V -l version -d 'Print version'
