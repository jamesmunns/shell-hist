# `shell-hist`

Take a look at your most common shell commands, based on your history file

![shell-hist Screenshot](./assets/screenshot.png)

```text
james@laptop ➜ ~ shell-hist --help

shell-hist 0.1.0
James Munns <james.munns@ferrous-systems.com>
A CLI tool for inspecting shell history

USAGE:
    shell-hist [FLAGS] [OPTIONS]

FLAGS:
        --flavor-bash      Parse Bash history
    -e, --display-exact    Show the most common exact commands
    -z, --display-fuzzy    Show fuzzy matched output. This is the default option.
    -h, --help             Prints help information
    -t, --display-heat     Show the most common command components
    -V, --version          Prints version information
        --flavor-zsh       Parse Zsh history. This is the default option.

OPTIONS:
    -n <count>        How many items to show [default: 10]
    -f <file>         File to parse. Defaults to history file of selected shell flavor
```

## Installation

```
cargo install shell-hist
```

## License

This project is licensed under the terms of both the [MIT License] and the [Apache License v2.0]

Copies of the licenses used by this project may also be found here:

* [MIT License Hosted]
* [Apache License v2.0 Hosted]

[MIT License]: ./LICENSE-MIT
[Apache License v2.0]: ./LICENSE-APACHE
[MIT License Hosted]: https://opensource.org/licenses/MIT
[Apache License v2.0 Hosted]: http://www.apache.org/licenses/LICENSE-2.0

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
