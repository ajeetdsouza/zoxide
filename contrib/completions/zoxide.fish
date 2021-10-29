complete -c zoxide -n "__fish_use_subcommand" -s h -l help -d 'Print help information'
complete -c zoxide -n "__fish_use_subcommand" -s V -l version -d 'Print version information'
complete -c zoxide -n "__fish_use_subcommand" -f -a "add" -d 'Add a new directory or increment its rank'
complete -c zoxide -n "__fish_use_subcommand" -f -a "import" -d 'Import entries from another application'
complete -c zoxide -n "__fish_use_subcommand" -f -a "init" -d 'Generate shell configuration'
complete -c zoxide -n "__fish_use_subcommand" -f -a "query" -d 'Search for a directory in the database'
complete -c zoxide -n "__fish_use_subcommand" -f -a "remove" -d 'Remove a directory from the database'
complete -c zoxide -n "__fish_seen_subcommand_from add" -s h -l help -d 'Print help information'
complete -c zoxide -n "__fish_seen_subcommand_from add" -s V -l version -d 'Print version information'
complete -c zoxide -n "__fish_seen_subcommand_from import" -l from -d 'Application to import from' -r -f -a "{autojump	,z	}"
complete -c zoxide -n "__fish_seen_subcommand_from import" -l merge -d 'Merge into existing database'
complete -c zoxide -n "__fish_seen_subcommand_from import" -s h -l help -d 'Print help information'
complete -c zoxide -n "__fish_seen_subcommand_from import" -s V -l version -d 'Print version information'
complete -c zoxide -n "__fish_seen_subcommand_from init" -l cmd -d 'Renames the \'z\' command and corresponding aliases' -r
complete -c zoxide -n "__fish_seen_subcommand_from init" -l hook -d 'Chooses event upon which an entry is added to the database' -r -f -a "{none	,prompt	,pwd	}"
complete -c zoxide -n "__fish_seen_subcommand_from init" -l no-aliases -d 'Prevents zoxide from defining any commands'
complete -c zoxide -n "__fish_seen_subcommand_from init" -s h -l help -d 'Print help information'
complete -c zoxide -n "__fish_seen_subcommand_from init" -s V -l version -d 'Print version information'
complete -c zoxide -n "__fish_seen_subcommand_from query" -l exclude -d 'Exclude a path from results' -r -f -a "(__fish_complete_directories)"
complete -c zoxide -n "__fish_seen_subcommand_from query" -l all -d 'Show deleted directories'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s i -l interactive -d 'Use interactive selection'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s l -l list -d 'List all matching directories'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s s -l score -d 'Print score with results'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s h -l help -d 'Print help information'
complete -c zoxide -n "__fish_seen_subcommand_from query" -s V -l version -d 'Print version information'
complete -c zoxide -n "__fish_seen_subcommand_from remove" -s i -l interactive -r
complete -c zoxide -n "__fish_seen_subcommand_from remove" -s h -l help -d 'Print help information'
complete -c zoxide -n "__fish_seen_subcommand_from remove" -s V -l version -d 'Print version information'
