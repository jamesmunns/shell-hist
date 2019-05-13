use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    pub display: DisplayOpts,

    #[structopt(flatten)]
    pub shell: ShellOpts,

    /// File to parse. Defaults to history file of selected shell flavor
    #[structopt(short = "f", parse(from_os_str))]
    pub file: Option<PathBuf>,

    /// How many items to show
    #[structopt(short = "n", default_value = "10")]
    pub count: usize,
}


#[derive(StructOpt)]
pub struct DisplayOpts {
    /// Show fuzzy matched output. This is the default option.
    #[structopt(short="z", long="display-fuzzy")]
    pub fuzzy: bool,

    /// Show the most common exact commands
    #[structopt(short="e", long="display-exact")]
    pub exact: bool,

    /// Show the most common command components
    #[structopt(short="t", long="display-heat")]
    pub heat: bool,
}

#[derive(StructOpt)]
pub struct ShellOpts {
    /// Parse Zsh history. This is the default option.
    #[structopt(long="flavor-zsh")]
    pub zsh: bool,

    /// Parse Bash history
    #[structopt(long="flavor-bash")]
    pub bash: bool,
}

pub enum HistoryFlavor {
    Zsh,
    Bash,
}

impl ShellOpts {
    pub fn validate(self) -> HistoryFlavor {
        let shell_path = env::var("SHELL");

        match (self.zsh, self.bash) {
            (false, false) => {
                match shell_path {
                    Ok(path) => {
                        match &path[..] {
                            "/bin/zsh" => HistoryFlavor::Zsh,
                            "/bin/bash" => HistoryFlavor::Bash,
                            &_ => {
                                eprintln!("Do not know what shell you are using");
                                std::process::exit(-1);
                            }
                        }
                    }
                    Err(_e) => {
                        eprintln!("Could not read the SHELL env variable");
                        std::process::exit(-1);
                    }
                }
            },
            (true, false) => HistoryFlavor::Zsh,
            (false, true) => HistoryFlavor::Bash,
            (true, true) => {
                eprintln!("Multiple shell modes selected, please select one or none");
                std::process::exit(-1);
            }
        }
    }
}

pub enum DisplayMode {
    Fuzzy,
    Exact,
    Heat,
}

impl DisplayOpts {
    pub fn validate(self) -> DisplayMode {
        match (self.fuzzy, self.exact, self.heat) {
            (false, false, false) => DisplayMode::Fuzzy,
            (true, false, false) => DisplayMode::Fuzzy,
            (false, true, false) => DisplayMode::Exact,
            (false, false, true) => DisplayMode::Heat,
            _ => {
                eprintln!("Multiple display modes selected, please select one or none");
                std::process::exit(-1);
            }
        }
    }
}
