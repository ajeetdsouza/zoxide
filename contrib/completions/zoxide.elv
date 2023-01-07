
use builtin;
use str;

set edit:completion:arg-completer[zoxide] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'zoxide'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'zoxide'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
            cand add 'Add a new directory or increment its rank'
            cand edit 'Edit the database'
            cand import 'Import entries from another application'
            cand init 'Generate shell configuration'
            cand query 'Search for a directory in the database'
            cand remove 'Remove a directory from the database'
        }
        &'zoxide;add'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;edit'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
            cand decrement 'decrement'
            cand delete 'delete'
            cand increment 'increment'
            cand reload 'reload'
        }
        &'zoxide;edit;decrement'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;edit;delete'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;edit;increment'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;edit;reload'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;import'= {
            cand --from 'Application to import from'
            cand --merge 'Merge into existing database'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;init'= {
            cand --cmd 'Changes the prefix of the `z` and `zi` commands'
            cand --hook 'Changes how often zoxide increments a directory''s score'
            cand --no-cmd 'Prevents zoxide from defining the `z` and `zi` commands'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;query'= {
            cand --exclude 'Exclude a path from results'
            cand --all 'Show deleted directories'
            cand -i 'Use interactive selection'
            cand --interactive 'Use interactive selection'
            cand -l 'List all matching directories'
            cand --list 'List all matching directories'
            cand -s 'Print score with results'
            cand --score 'Print score with results'
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
        &'zoxide;remove'= {
            cand -h 'Print help information'
            cand --help 'Print help information'
            cand -V 'Print version information'
            cand --version 'Print version information'
        }
    ]
    $completions[$command]
}
