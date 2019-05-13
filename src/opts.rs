use crate::eject;
use dirs::home_dir;
use regex::Regex;
use std::env;
use std::fmt;
use std::path::PathBuf;
use std::slice::Iter;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(flatten)]
    pub display: DisplayOpts,

    #[structopt(flatten)]
    pub shell: ShellOpts,

    /// File to parse. Defaults to history file of selected or detected shell flavor
    #[structopt(short = "f", parse(from_os_str))]
    pub file: Option<PathBuf>,

    /// How many items to show
    #[structopt(short = "n", default_value = "10")]
    pub count: usize,
}

#[derive(StructOpt)]
pub struct DisplayOpts {
    /// Show fuzzy matched output. This is the default option.
    #[structopt(short = "z", long = "display-fuzzy")]
    pub fuzzy: bool,

    /// Show the most common exact commands
    #[structopt(short = "e", long = "display-exact")]
    pub exact: bool,

    /// Show the most common command components
    #[structopt(short = "t", long = "display-heat")]
    pub heat: bool,
}

#[derive(StructOpt)]
pub struct ShellOpts {
    /// Manually select ZSH history, overriding auto-detect
    #[structopt(long = "flavor-zsh")]
    pub zsh: bool,

    /// Manually select Bash history, overriding auto-detect
    #[structopt(long = "flavor-bash")]
    pub bash: bool,
}

#[derive(Copy, Clone)]
pub enum HistoryFlavor {
    Zsh,
    Bash,
}

impl fmt::Display for HistoryFlavor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HistoryFlavor::Bash => write!(f, "bash"),
            HistoryFlavor::Zsh => write!(f, "zsh"),
        }
    }
}

impl ShellOpts {
    pub fn detect_shell() -> Option<HistoryFlavor> {

        let shell_path = env::var("SHELL").ok()?;

        for sh in HistoryFlavor::iter() {
            if shell_path.contains(sh.to_string().as_str()) {
                return Some(*sh);
            }
        }

        None
    }

    pub fn validate(self) -> HistoryFlavor {
        match (self.zsh, self.bash) {
            (false, false) => {
                if let Some(sh) = Self::detect_shell() {
                    sh
                } else {
                    eject("Unable to detect shell, please manually select a shell flavor");
                }
            }
            (true, false) => HistoryFlavor::Zsh,
            (false, true) => HistoryFlavor::Bash,
            (true, true) => {
                eject("Multiple shell modes selected, please select one or none");
            }
        }
    }
}

impl HistoryFlavor {
    pub fn iter() -> Iter<'static, HistoryFlavor> {
        use HistoryFlavor::*;
        const HISTORY_FLAVOR: [HistoryFlavor; 2] = [Bash, Zsh];
        HISTORY_FLAVOR.into_iter()
    }
    pub fn history_path(&self) -> PathBuf {
        use HistoryFlavor::*;
        let name = match self {
            Zsh => ".zsh_history",
            Bash => ".bash_history",
        };

        let mut dir = home_dir().unwrap_or_else(|| {
            eject("Unable to determine home path. Please specify history file path");
        });
        dir.push(name);
        dir
    }

    pub fn regex_and_capture_idx(&self) -> (Regex, usize) {
        use HistoryFlavor::*;
        let (re_res, idx) = match self {
            Zsh => (Regex::new(r"^.*;(sudo )?(.*)$"), 2),
            Bash => (Regex::new(r"^(sudo )?(.*)$"), 2),
        };

        (
            re_res.unwrap_or_else(|_| eject("Failed to compile regex!")),
            idx,
        )
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
                eject("Multiple display modes selected, please select one or none");
            }
        }
    }
}
