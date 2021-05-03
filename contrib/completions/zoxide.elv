
edit:completion:arg-completer[zoxide] = [@words]{
    fn spaces [n]{
        repeat $n ' ' | joins ''
    }
    fn cand [text desc]{
        edit:complex-candidate $text &display-suffix=' '(spaces (- 14 (wcswidth $text)))$desc
    }
    command = 'zoxide'
    for word $words[1:-1] {
        if (has-prefix $word '-') {
            break
        }
        command = $command';'$word
    }
    completions = [
        &'zoxide'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
            cand -V 'Prints version information'
            cand --version 'Prints version information'
            cand add 'Add a new directory or increment its rank'
            cand import 'Import entries from another application'
            cand init 'Generate shell configuration'
            cand query 'Search for a directory in the database'
            cand remove 'Remove a directory from the database'
        }
        &'zoxide;add'= {
            cand -h 'Prints help information'
            cand --help 'Prints help information'
        }
        &'zoxide;import'= {
            cand --from 'Application to import from'
            cand --merge 'Merge into existing database'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
        }
        &'zoxide;init'= {
            cand --cmd 'Renames the ''z'' command and corresponding aliases'
            cand --hook 'Chooses event upon which an entry is added to the database'
            cand --no-aliases 'Prevents zoxide from defining any commands'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
        }
        &'zoxide;query'= {
            cand --exclude 'Exclude a path from results'
            cand -i 'Use interactive selection'
            cand --interactive 'Use interactive selection'
            cand -l 'List all matching directories'
            cand --list 'List all matching directories'
            cand -s 'Print score with results'
            cand --score 'Print score with results'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
        }
        &'zoxide;remove'= {
            cand -i 'i'
            cand --interactive 'interactive'
            cand -h 'Prints help information'
            cand --help 'Prints help information'
        }
    ]
    $completions[$command]
}
