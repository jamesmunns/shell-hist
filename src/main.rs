use structopt::StructOpt;

mod opts;
use opts::DisplayMode;

mod parse;
use parse::{Node, CtNode, Line, parse};

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

fn main() {
    const BARS_WIDE: usize = 8;

    let opt = opts::Options::from_args();
    let mode = opt.display.validate();

    let (title, func): (&str, fn(&Node, usize, &str) -> Vec<CtNode>) = match mode {
        DisplayMode::Fuzzy => ("Fuzzy", Node::top_inclusive_filt),
        DisplayMode::Exact => ("Exact", Node::top_exclusive),
        DisplayMode::Heat => ("Heatmap", Node::top_inclusive),
    };

    let t = parse(opt.file, opt.shell.validate());
    // println!("{:#?}", t);

    let lines = ct_node_to_list_line(func(&t, opt.count, ""));

    println!();
    println!("  {} Commands ", title);
    println!();
    println!("|  HEAT    |  COUNT   |  COMMAND ");
    println!("| -------- | -------- | ---------");

    for i in &lines {
        println!("| {} | {:8} | {}", pct_to_bar(i.pct, BARS_WIDE), i.node.count, i.node.full_text);
    }
    println!();

}

fn ct_node_to_list_line(mut in_dat: Vec<CtNode>) -> Vec<Line> {
    let max = if let Some(item) = in_dat.first() {
        item.count as f64
    } else {
        return vec![];
    };

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

pub fn eject(reason: &str) -> ! {
    eprintln!("{}", reason);
    std::process::exit(-1);
}
