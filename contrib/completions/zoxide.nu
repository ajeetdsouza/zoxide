module completions {

  # A smarter cd command for your terminal
  export extern zoxide [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Add a new directory or increment its rank
  export extern "zoxide add" [
    --score(-s): string       # The rank to increment the entry if it exists or initialize it with if it doesn't
    --help(-h)                # Print help
    --version(-V)             # Print version
    ...paths: path
  ]

  # Edit the database
  export extern "zoxide edit" [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  export extern "zoxide edit decrement" [
    --help(-h)                # Print help
    --version(-V)             # Print version
    path: string
  ]

  export extern "zoxide edit delete" [
    --help(-h)                # Print help
    --version(-V)             # Print version
    path: string
  ]

  export extern "zoxide edit increment" [
    --help(-h)                # Print help
    --version(-V)             # Print version
    path: string
  ]

  export extern "zoxide edit reload" [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import entries from another application
  export extern "zoxide import" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import from atuin
  export extern "zoxide import atuin" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import from autojump
  export extern "zoxide import autojump" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import from fasd
  export extern "zoxide import fasd" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import from z
  export extern "zoxide import z" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import from z.lua
  export extern "zoxide import z.lua" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Import from zsh-z
  export extern "zoxide import zsh-z" [
    --merge                   # Merge into existing database
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  def "nu-complete zoxide init shell" [] {
    [ "bash" "elvish" "fish" "nushell" "posix" "powershell" "tcsh" "xonsh" "zsh" ]
  }

  def "nu-complete zoxide init hook" [] {
    [ "none" "prompt" "pwd" ]
  }

  # Generate shell configuration
  export extern "zoxide init" [
    --no-cmd                  # Prevents zoxide from defining the `z` and `zi` commands
    --cmd: string             # Changes the prefix of the `z` and `zi` commands
    --hook: string@"nu-complete zoxide init hook" # Changes how often zoxide increments a directory's score
    --help(-h)                # Print help
    --version(-V)             # Print version
    shell: string@"nu-complete zoxide init shell"
  ]

  # Search for a directory in the database
  export extern "zoxide query" [
    --all(-a)                 # Show unavailable directories
    --interactive(-i)         # Use interactive selection
    --list(-l)                # List all matching directories
    --score(-s)               # Print score with results
    --exclude: path           # Exclude the current directory
    --base-dir: path          # Only search within this directory
    --help(-h)                # Print help
    --version(-V)             # Print version
    ...keywords: string
  ]

  # Remove a directory from the database
  export extern "zoxide remove" [
    --help(-h)                # Print help
    --version(-V)             # Print version
    ...paths: path
  ]

}

export use completions *
