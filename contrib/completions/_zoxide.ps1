
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'zoxide' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'zoxide'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'zoxide' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('add', 'add', [CompletionResultType]::ParameterValue, 'Add a new directory or increment its rank')
            [CompletionResult]::new('import', 'import', [CompletionResultType]::ParameterValue, 'Import entries from another application')
            [CompletionResult]::new('init', 'init', [CompletionResultType]::ParameterValue, 'Generate shell configuration')
            [CompletionResult]::new('query', 'query', [CompletionResultType]::ParameterValue, 'Search for a directory in the database')
            [CompletionResult]::new('remove', 'remove', [CompletionResultType]::ParameterValue, 'Remove a directory from the database')
            break
        }
        'zoxide;add' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'zoxide;import' {
            [CompletionResult]::new('--from', 'from', [CompletionResultType]::ParameterName, 'Application to import from')
            [CompletionResult]::new('--merge', 'merge', [CompletionResultType]::ParameterName, 'Merge into existing database')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'zoxide;init' {
            [CompletionResult]::new('--cmd', 'cmd', [CompletionResultType]::ParameterName, 'Changes the prefix of the `z` and `zi` commands')
            [CompletionResult]::new('--hook', 'hook', [CompletionResultType]::ParameterName, 'Changes how often zoxide increments a directory''s score')
            [CompletionResult]::new('--no-cmd', 'no-cmd', [CompletionResultType]::ParameterName, 'Prevents zoxide from defining the `z` and `zi` commands')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'zoxide;query' {
            [CompletionResult]::new('--exclude', 'exclude', [CompletionResultType]::ParameterName, 'Exclude a path from results')
            [CompletionResult]::new('--all', 'all', [CompletionResultType]::ParameterName, 'Show deleted directories')
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Use interactive selection')
            [CompletionResult]::new('--interactive', 'interactive', [CompletionResultType]::ParameterName, 'Use interactive selection')
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'List all matching directories')
            [CompletionResult]::new('--list', 'list', [CompletionResultType]::ParameterName, 'List all matching directories')
            [CompletionResult]::new('-s', 's', [CompletionResultType]::ParameterName, 'Print score with results')
            [CompletionResult]::new('--score', 'score', [CompletionResultType]::ParameterName, 'Print score with results')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
        'zoxide;remove' {
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Use interactive selection')
            [CompletionResult]::new('--interactive', 'interactive', [CompletionResultType]::ParameterName, 'Use interactive selection')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
