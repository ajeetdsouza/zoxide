module completions {

  # A smarter cd command for your terminal
  export extern zoxide [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Add a new directory or increment its rank
  export extern "zoxide add" [
    ...paths: path
    --score(-s): string       # The rank to increment the entry if it exists or initialize it with if it doesn't
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Edit the database
  export extern "zoxide edit" [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  export extern "zoxide edit decrement" [
    path: string
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  export extern "zoxide edit delete" [
    path: string
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  export extern "zoxide edit increment" [
    path: string
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  export extern "zoxide edit reload" [
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  def "nu-complete zoxide import from" [] {
    [ "autojump" "z" ]
  }

  # Import entries from another application
  export extern "zoxide import" [
    path: path
    --from: string@"nu-complete zoxide import from" # Application to import from
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
    shell: string@"nu-complete zoxide init shell"
    --no-cmd                  # Prevents zoxide from defining the `z` and `zi` commands
    --cmd: string             # Changes the prefix of the `z` and `zi` commands
    --hook: string@"nu-complete zoxide init hook" # Changes how often zoxide increments a directory's score
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Search for a directory in the database
  export extern "zoxide query" [
    ...keywords: string
    --all(-a)                 # Show unavailable directories
    --interactive(-i)         # Use interactive selection
    --list(-l)                # List all matching directories
    --score(-s)               # Print score with results
    --exclude: path           # Exclude the current directory
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

  # Remove a directory from the database
  export extern "zoxide remove" [
    ...paths: path
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

}

export use completions *
