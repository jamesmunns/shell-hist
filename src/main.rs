const BARS: &[char] = &[
    ' ',
    '▏',
    '▎',
    '▍',
    '▌',
    '▋',
    '▊',
    '▉',
    '█',
];

use structopt::StructOpt;

#[derive(StructOpt)]
struct DisplayOpts {
    /// Show fuzzy matched output. This is the default option.
    #[structopt(short="z", long="display-fuzzy")]
    fuzzy: bool,

    /// Show the most common exact commands
    #[structopt(short="e", long="display-exact")]
    exact: bool,

    /// Show the most common command components
    #[structopt(short="t", long="display-heat")]
    heat: bool,
}

#[derive(StructOpt)]
struct ShellOpts {
    /// Parse Zsh history. This is the default option.
    #[structopt(long="flavor-zsh")]
    zsh: bool,

    /// Parse Bash history
    #[structopt(long="flavor-bash")]
    bash: bool,
}

impl ShellOpts {
    fn validate(self) -> HistoryFlavor {
        match (self.zsh, self.bash) {
            (false, false) => HistoryFlavor::Zsh,
            (true, false) => HistoryFlavor::Zsh,
            (false, true) => HistoryFlavor::Bash,
            (true, true) => {
                eprintln!("Multiple shell modes selected, please select one or none");
                std::process::exit(-1);
            }
        }
    }
}

enum DisplayMode {
    Fuzzy,
    Exact,
    Heat,
}

impl DisplayOpts {
    fn validate(self) -> DisplayMode {
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

use std::path::PathBuf;

#[derive(StructOpt)]
struct Options {
    #[structopt(flatten)]
    display: DisplayOpts,

    #[structopt(flatten)]
    shell: ShellOpts,

    /// File to parse. Defaults to history file of selected shell flavor
    #[structopt(short = "f", parse(from_os_str))]
    file: Option<PathBuf>,

    /// How many items to show
    #[structopt(short = "n", default_value = "10")]
    count: usize,
}

fn main() {
    const BARS_WIDE: usize = 8;

    let opt = Options::from_args();
    let mode = opt.display.validate();

    let (title, func): (&str, fn(&Node, usize, &str) -> Vec<CtNode>) = match mode {
        DisplayMode::Fuzzy => ("Fuzzy", Node::top_inclusive_filt),
        DisplayMode::Exact => ("Exact", Node::top_exclusive),
        DisplayMode::Heat => ("Heatmap", Node::top_inclusive),
    };

    let t = parse(opt.file, opt.shell.validate());
    // println!("{:#?}", t);

    let lines = ct_node_to_list_line(func(&t, opt.count, ""));

    println!("");
    println!("  {} Commands ", title);
    println!("");
    println!("|  HEAT    |  COUNT   |  COMMAND ");
    println!("| -------- | -------- | ---------");

    for i in lines.iter() {
        println!("| {} | {:8} | {}", pct_to_bar(i.pct, BARS_WIDE), i.node.count, i.node.full_text);
    }
    println!("");

}

use std::collections::BTreeMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Eq, Debug)]
struct CtNode {
    count: usize,
    full_text: String,
}

#[derive(Debug)]
struct Line {
    pct: f64,
    node: CtNode,
}

impl Ord for CtNode {
    fn cmp(&self, other: &CtNode) -> Ordering {
        // Why does this need to be backwards?
        other.count.cmp(&self.count)
    }
}

impl PartialOrd for CtNode {
    fn partial_cmp(&self, other: &CtNode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CtNode {
    fn eq(&self, other: &CtNode) -> bool {
        self.count == other.count
    }
}

#[derive(Debug)]
struct Node {
    pub children: BTreeMap<String, Node>,
    pub count_inclusive: usize,
    pub count_exact: usize,
}

impl Node {
    fn new() -> Self {
        Self {
            children: BTreeMap::new(),
            count_inclusive: 0,
            count_exact: 0,
        }
    }

    fn chomp(&mut self, toks: &[String]) {
        self.count_inclusive += 1;
        if toks.is_empty() {
            self.count_exact += 1;
            return;
        }

        if !self.children.contains_key(&toks[0]) {
            self.children.insert(toks[0].clone(), Node::new());
        }
        let child = self.children.get_mut(&toks[0]).unwrap();

        child.chomp(&toks[1..]);
    }

    fn top_exclusive(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
        let mut topn: BinaryHeap<CtNode> = BinaryHeap::new();
        self.children.iter().for_each(|(cmd, node)| {
            let next_txt = format!("{}{} ", prefix, cmd);
            node.top_exclusive(ct, &next_txt)
                .drain(..)
                .for_each(|t| {
                    topn.push(t)
                });
            topn.push(CtNode {
                count: node.count_exact,
                full_text: next_txt.trim_end().to_owned(),
            });

        });
        while topn.len() > ct {
            topn.pop();
        }

        let mut x = topn.drain().collect::<Vec<CtNode>>();
        x.sort();
        x
    }

    fn top_inclusive(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
        let mut topn: BinaryHeap<CtNode> = BinaryHeap::new();
        self.children.iter().for_each(|(cmd, node)| {
            let next_txt = format!("{}{} ", prefix, cmd);
            node.top_inclusive(ct, &next_txt)
                .drain(..)
                .for_each(|t| {
                    topn.push(t)
                });
            topn.push(CtNode {
                count: node.count_inclusive,
                full_text: next_txt.trim_end().to_owned(),
            });

        });
        while topn.len() > ct {
            topn.pop();
        }

        let mut x = topn.drain().collect::<Vec<CtNode>>();
        x.sort();
        x
    }

    fn top_inclusive_filt(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
        let mut topn: BinaryHeap<CtNode> = BinaryHeap::new();
        self.children.iter().for_each(|(cmd, node)| {
            let next_txt = format!("{}{} ", prefix, cmd);
            node.top_inclusive_filt(ct, &next_txt)
                .drain(..)
                .for_each(|t| {
                    topn.push(t)
                });

            if (node.count_exact != 0) && (((node.count_exact * 10) / node.count_inclusive) >= 1) {
                topn.push(CtNode {
                    count: node.count_inclusive,
                    full_text: next_txt.trim_end().to_owned(),
                });
            }

        });
        while topn.len() > ct {
            topn.pop();
        }

        let mut x = topn.drain().collect::<Vec<CtNode>>();
        x.sort();
        x
    }
}

fn ct_node_to_list_line(mut in_dat: Vec<CtNode>) -> Vec<Line> {
    let max = in_dat.first().unwrap().count as f64;
    in_dat.drain(..).map(|line| {
        Line {
            pct: (line.count as f64) / max,
            node: line,
        }
    })
    .collect()
}

fn pct_to_bar(pct: f64, width: usize) -> String {
    let mult = (BARS.len() - 1) * width;
    let ct = pct * (mult as f64);
    let ct = ct.round();
    let mut ct = ct as usize;

    let mut out = String::with_capacity(width);

    for _ in 0..width {
        let idx = std::cmp::min(ct, BARS.len() - 1);
        ct -= idx;
        out.push(BARS[idx]);
    }

    out
}

use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;
use dirs::home_dir;

use regex::Regex;

enum HistoryFlavor {
    Zsh,
    Bash,
}

fn parse<'a>(path: Option<PathBuf>, flavor: HistoryFlavor) -> Node  {
    let mut tree = Node::new();
    use HistoryFlavor::*;
    let (name, re, idx) = match flavor {
        Zsh => {
            (".zsh_history", Regex::new(r"^.*;(sudo )?(.*)$").unwrap(), 2)
        },
        Bash => {
            (".bash_history", Regex::new(r"^(sudo )?(.*)$").unwrap(), 2)
        }
    };

    let path = path.unwrap_or_else(|| {
        let mut dir = home_dir().unwrap();
        dir.push(name);
        dir
    });

    let f = File::open(&path).unwrap();
    let f = BufReader::new(f);

    let _: io::Result<()> = f
        .lines()
        .filter_map(|line| line.ok())
        .filter_map(|line| Some(
            re
                .captures(&line)?
                .get(idx)?
                .as_str()
                .to_string()
            )
        )
        .try_for_each(|lineout| {
            let toks = lineout.split_whitespace().map(|t| t.to_string()).collect::<Vec<String>>();

            tree.chomp(&toks);
            Ok(())
        }
    );

    tree
}
