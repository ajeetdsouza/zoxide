const completion: Fig.Spec = {
  name: "zoxide",
  description: "A smarter cd command for your terminal",
  subcommands: [
    {
      name: "add",
      description: "Add a new directory or increment its rank",
      options: [
        {
          name: ["-h", "--help"],
          description: "Print help",
        },
        {
          name: ["-V", "--version"],
          description: "Print version",
        },
      ],
      args: {
        name: "paths",
        isVariadic: true,
        template: "folders",
      },
    },
    {
      name: "edit",
      description: "Edit the database",
      subcommands: [
        {
          name: "decrement",
          hidden: true,
          options: [
            {
              name: ["-h", "--help"],
              description: "Print help",
            },
            {
              name: ["-V", "--version"],
              description: "Print version",
            },
          ],
          args: {
            name: "path",
          },
        },
        {
          name: "delete",
          hidden: true,
          options: [
            {
              name: ["-h", "--help"],
              description: "Print help",
            },
            {
              name: ["-V", "--version"],
              description: "Print version",
            },
          ],
          args: {
            name: "path",
          },
        },
        {
          name: "increment",
          hidden: true,
          options: [
            {
              name: ["-h", "--help"],
              description: "Print help",
            },
            {
              name: ["-V", "--version"],
              description: "Print version",
            },
          ],
          args: {
            name: "path",
          },
        },
        {
          name: "reload",
          hidden: true,
          options: [
            {
              name: ["-h", "--help"],
              description: "Print help",
            },
            {
              name: ["-V", "--version"],
              description: "Print version",
            },
          ],
        },
      ],
      options: [
        {
          name: ["-h", "--help"],
          description: "Print help",
        },
        {
          name: ["-V", "--version"],
          description: "Print version",
        },
      ],
    },
    {
      name: "import",
      description: "Import entries from another application",
      options: [
        {
          name: "--from",
          description: "Application to import from",
          isRepeatable: true,
          args: {
            name: "from",
            suggestions: [
              "autojump",
              "z",
            ],
          },
        },
        {
          name: "--merge",
          description: "Merge into existing database",
        },
        {
          name: ["-h", "--help"],
          description: "Print help",
        },
        {
          name: ["-V", "--version"],
          description: "Print version",
        },
      ],
      args: {
        name: "path",
        template: "filepaths",
      },
    },
    {
      name: "init",
      description: "Generate shell configuration",
      options: [
        {
          name: "--cmd",
          description: "Changes the prefix of the `z` and `zi` commands",
          isRepeatable: true,
          args: {
            name: "cmd",
            isOptional: true,
          },
        },
        {
          name: "--hook",
          description: "Changes how often zoxide increments a directory's score",
          isRepeatable: true,
          args: {
            name: "hook",
            isOptional: true,
            suggestions: [
              "none",
              "prompt",
              "pwd",
            ],
          },
        },
        {
          name: "--no-cmd",
          description: "Prevents zoxide from defining the `z` and `zi` commands",
        },
        {
          name: ["-h", "--help"],
          description: "Print help",
        },
        {
          name: ["-V", "--version"],
          description: "Print version",
        },
      ],
      args: {
        name: "shell",
        suggestions: [
          "bash",
          "elvish",
          "fish",
          "nushell",
          "posix",
          "powershell",
          "xonsh",
          "zsh",
        ],
      },
    },
    {
      name: "query",
      description: "Search for a directory in the database",
      options: [
        {
          name: "--exclude",
          description: "Exclude the current directory",
          isRepeatable: true,
          args: {
            name: "exclude",
            isOptional: true,
            template: "folders",
          },
        },
        {
          name: ["-a", "--all"],
          description: "Show unavailable directories",
        },
        {
          name: ["-i", "--interactive"],
          description: "Use interactive selection",
          exclusiveOn: [
            "-l",
            "--list",
          ],
        },
        {
          name: ["-l", "--list"],
          description: "List all matching directories",
          exclusiveOn: [
            "-i",
            "--interactive",
          ],
        },
        {
          name: ["-s", "--score"],
          description: "Print score with results",
        },
        {
          name: ["-h", "--help"],
          description: "Print help",
        },
        {
          name: ["-V", "--version"],
          description: "Print version",
        },
      ],
      args: {
        name: "keywords",
        isVariadic: true,
        isOptional: true,
      },
    },
    {
      name: "remove",
      description: "Remove a directory from the database",
      options: [
        {
          name: ["-h", "--help"],
          description: "Print help",
        },
        {
          name: ["-V", "--version"],
          description: "Print version",
        },
      ],
      args: {
        name: "paths",
        isVariadic: true,
        isOptional: true,
        template: "folders",
      },
    },
  ],
  options: [
    {
      name: ["-h", "--help"],
      description: "Print help",
    },
    {
      name: ["-V", "--version"],
      description: "Print version",
    },
  ],
};

export default completion;
