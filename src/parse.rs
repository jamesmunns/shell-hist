use std::{
    cmp::Ordering,
    collections::{BinaryHeap, BTreeMap},
    fs::File,
    io::{self, prelude::*, BufReader},
    path::PathBuf,
};

use crate::opts::HistoryFlavor;
use crate::eject;


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

        // We guarantee above the children contain this token
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

    /// Get the top `ct` items that have been called or who's children have
    /// been called. This powers the `display-heat` mode
    pub fn top_inclusive(&self, ct: usize, prefix: &str) -> Vec<CtNode> {
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

pub fn parse(path: Option<PathBuf>, flavor: HistoryFlavor) -> Node  {
    let mut tree = Node::new();

    let path = path.unwrap_or_else(|| {
        flavor.history_path()
    });
    let (re, idx) = flavor.regex_and_capture_idx();

    let f = File::open(&path).unwrap_or_else(|_| {
        eject(&format!("Unable to open specified or detected history file: {:?}", path));
    });
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
