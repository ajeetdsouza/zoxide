# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_zoxide_global_optspecs
	string join \n h/help V/version
end

function __fish_zoxide_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_zoxide_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_zoxide_using_subcommand
	set -l cmd (__fish_zoxide_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c zoxide -n "__fish_zoxide_needs_command" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_needs_command" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_needs_command" -f -a "add" -d 'Add a new directory or increment its rank'
complete -c zoxide -n "__fish_zoxide_needs_command" -f -a "edit" -d 'Edit the database'
complete -c zoxide -n "__fish_zoxide_needs_command" -f -a "import" -d 'Import entries from another application'
complete -c zoxide -n "__fish_zoxide_needs_command" -f -a "init" -d 'Generate shell configuration'
complete -c zoxide -n "__fish_zoxide_needs_command" -f -a "query" -d 'Search for a directory in the database'
complete -c zoxide -n "__fish_zoxide_needs_command" -f -a "remove" -d 'Remove a directory from the database'
complete -c zoxide -n "__fish_zoxide_using_subcommand add" -s s -l score -d 'The rank to increment the entry if it exists or initialize it with if it doesn\'t' -r
complete -c zoxide -n "__fish_zoxide_using_subcommand add" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand add" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and not __fish_seen_subcommand_from decrement delete increment reload" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and not __fish_seen_subcommand_from decrement delete increment reload" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and not __fish_seen_subcommand_from decrement delete increment reload" -f -a "decrement"
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and not __fish_seen_subcommand_from decrement delete increment reload" -f -a "delete"
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and not __fish_seen_subcommand_from decrement delete increment reload" -f -a "increment"
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and not __fish_seen_subcommand_from decrement delete increment reload" -f -a "reload"
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from decrement" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from decrement" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from delete" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from delete" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from increment" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from increment" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from reload" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand edit; and __fish_seen_subcommand_from reload" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand import" -l from -d 'Application to import from' -r -f -a "{autojump\t'',z\t''}"
complete -c zoxide -n "__fish_zoxide_using_subcommand import" -l merge -d 'Merge into existing database'
complete -c zoxide -n "__fish_zoxide_using_subcommand import" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand import" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand init" -l cmd -d 'Changes the prefix of the `z` and `zi` commands' -r
complete -c zoxide -n "__fish_zoxide_using_subcommand init" -l hook -d 'Changes how often zoxide increments a directory\'s score' -r -f -a "{none\t'',prompt\t'',pwd\t''}"
complete -c zoxide -n "__fish_zoxide_using_subcommand init" -l no-cmd -d 'Prevents zoxide from defining the `z` and `zi` commands'
complete -c zoxide -n "__fish_zoxide_using_subcommand init" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand init" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -l exclude -d 'Exclude the current directory' -r -f -a "(__fish_complete_directories)"
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -s a -l all -d 'Show unavailable directories'
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -s i -l interactive -d 'Use interactive selection'
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -s l -l list -d 'List all matching directories'
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -s s -l score -d 'Print score with results'
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand query" -s V -l version -d 'Print version'
complete -c zoxide -n "__fish_zoxide_using_subcommand remove" -s h -l help -d 'Print help'
complete -c zoxide -n "__fish_zoxide_using_subcommand remove" -s V -l version -d 'Print version'
