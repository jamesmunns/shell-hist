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

    /// Parse Fish history
    #[structopt(long="flavor-fish")]
    pub fish: bool,
}

#[derive(Copy, Clone)]
pub enum HistoryFlavor {
    Zsh,
    Bash,
    Fish
}

impl ShellOpts {
    pub fn validate(self) -> HistoryFlavor {
        match (self.zsh, self.bash, self.fish) {
            (false, false, false) => HistoryFlavor::Zsh,
            (true, false, false) => HistoryFlavor::Zsh,
            (false, true, false) => HistoryFlavor::Bash,
            (false, false, true) => HistoryFlavor::Fish,
            (_, _, _) => {
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
