use std::{
    cmp::Ordering,
    collections::{BTreeMap, BinaryHeap},
    fs::File,
    io::{self, prelude::*, BufReader},
    path::PathBuf,
};

use dirs::home_dir;
use regex::Regex;

use crate::opts::HistoryFlavor;

/// CtNode are post-processed partial/full commands with an associated non-specific count
#[derive(Eq, Debug)]
pub struct CtNode {
    pub count: usize,
    pub full_text: String,
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

/// A Line is a CtNode that knows its' rank
#[derive(Debug)]
pub struct Line {
    pub pct: f64,
    pub node: CtNode,
}

/// A Node is a recursive structure that counts the number of times
/// it has been called (`count_exact`), and itself + sum(all children) have
/// been called (`count_inclusive`).
#[derive(Debug)]
pub struct Node {
    pub children: BTreeMap<String, Node>,
    pub count_inclusive: usize,
    pub count_exact: usize,
}

impl Node {
    pub fn new() -> Self {
        Self {
            children: BTreeMap::new(),
            count_inclusive: 0,
            count_exact: 0,
        }
    }

    /// Recursively chomp string tokens
    pub fn chomp(&mut self, toks: &[String]) {
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

    /// Get the top `ct` items that have been called exactly. This
    /// powers the `display-exact` mode
    pub fn top_exclusive(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
        let mut topn: BinaryHeap<CtNode> = BinaryHeap::new();
        self.children.iter().for_each(|(cmd, node)| {
            let next_txt = format!("{}{} ", prefix, cmd);
            node.top_exclusive(ct, &next_txt)
                .drain(..)
                .for_each(|t| topn.push(t));
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

    /// Get the top `ct` items that have been called or who's children have
    /// been called. This powers the `display-heat` mode
    pub fn top_inclusive(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
        let mut topn: BinaryHeap<CtNode> = BinaryHeap::new();
        self.children.iter().for_each(|(cmd, node)| {
            let next_txt = format!("{}{} ", prefix, cmd);
            node.top_inclusive(ct, &next_txt)
                .drain(..)
                .for_each(|t| topn.push(t));
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

    /// Get the top `ct` items that have been called or who's children have
    /// been called. However, attempt to filter out nodes that are never directly
    /// called, to get rid of items that ALWAYS have a subcommand, like `git`.
    /// This function powers the `display-fuzzy` command.
    pub fn top_inclusive_filt(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
        let mut topn: BinaryHeap<CtNode> = BinaryHeap::new();
        self.children.iter().for_each(|(cmd, node)| {
            let next_txt = format!("{}{} ", prefix, cmd);
            node.top_inclusive_filt(ct, &next_txt)
                .drain(..)
                .for_each(|t| topn.push(t));

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

/// A common parser for history files with a rexex
fn parse_file(history: String, re_str: &'static str, idx: usize) -> Node {
    let re = Regex::new(re_str).unwrap();
    let mut tree = Node::new();

    let _: io::Result<()> = history
        .split("\n")
        .collect::<Vec<&str>>()
        .into_iter()
        .filter_map(|line| Some(re.captures(&line)?.get(idx)?.as_str().to_string()))
        .try_for_each(|lineout| {
            let toks = lineout
                .split_whitespace()
                .map(|t| t.to_string())
                .collect::<Vec<String>>();
            println!("Line: {:?}", toks);

            tree.chomp(&toks);
            Ok(())
        });

    tree
}

/// Wrapper function to abstract away history loading
fn load_history(path: PathBuf, flavor: HistoryFlavor) -> String {
    use crate::HistoryFlavor::*;
    match flavor {
        Zsh | Bash => {
            let mut f = File::open(path).unwrap();
            let mut history = String::new();
            f.read_to_string(&mut history).unwrap();
            history
        }
        // Fish is weird...
        Fish => {
            let history = std::process::Command::new("fish")
                .args(&["-c", "history"])
                .output()
                .unwrap();
            String::from_utf8(history.stdout).unwrap()
        }
    }
}

pub fn parse<'a>(path: Option<PathBuf>, flavor: HistoryFlavor) -> Node {
    use crate::HistoryFlavor::*;

    // Load history somehow
    let history = load_history(
        path.unwrap_or_else(|| {
            let mut dir = home_dir().unwrap();
            dir.push(match flavor {
                Zsh => ".zsh_history",
                Bash => ".bash_history",
                Fish => "", // Can be ignored
            });

            dir
        }),
        flavor,
    );

    println!("{}", &history);

    match flavor {
        Zsh => parse_file(history, r"^.*;(sudo )?(.*)$", 2),
        Bash => parse_file(history, r"^(sudo )?(.*)$", 2),
        Fish => parse_file(history, r"(.*)", 0),
    }
}
