#![feature(linked_list_extras)]

mod layout;
mod penalty;
mod annealing;
mod simulator;

extern crate getopts;

use std::env;
use std::fs::File;
use std::io::Read;
use getopts::Options;
use std::process::Command;

fn main()
{
    let mut opts = Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("d", "debug", "show debug logging");
    opts.optopt("t", "top", "number of top layouts to print (default: 1)", "TOP_LAYOUTS");
    opts.optopt("s", "swaps-per-iteration", "maximum number of swaps per iteration (default: 3)", "SWAPS");

    let args: Vec<String> = env::args().collect();
    let progname = &args[0];
    if args.len() < 2 {
        print_usage(progname, opts);
        return;
    }
    let command = &args[1];
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    // --help
    if matches.opt_present("h") {
        print_usage(progname, opts);
        return;
    }

    // Read corpus.
    let corpus_filename = match matches.free.get(0) {
        Some(f) => f,
        None => {
            print_usage(progname, opts);
            return;
        },
    };
    let mut f = match File::open(corpus_filename) {
        Ok(f) => f,
        Err(e) => {
            println!("Error: {}", e);
            panic!("could not read corpus");
        },
    };
    let mut corpus = String::new();
    match f.read_to_string(&mut corpus) {
        Ok(_) => (),
        Err(e) => {
            println!("Error: {}", e);
            panic!("could not read corpus");
        }
    };

    // Read layout, if applicable.
    let _layout;
    let layout = match matches.free.get(1) {
        None => &layout::SHAKA_LAYOUT,
        Some(layout_filename) => {
            let mut f = match File::open(layout_filename) {
                Ok(f) => f,
                Err(e) => {
                    println!("Error: {}", e);
                    panic!("could not read layout");
                }
            };
            let mut layout_str = String::new();
            match f.read_to_string(&mut layout_str) {
                Ok(_) => (),
                Err(e) => {
                    println!("Error: {}", e);
                    panic!("could not read layout");
                }
            };
            _layout = layout::Layout::from_string(&layout_str[..]);
            &_layout
        },
    };

    // Parse options.
    let debug = matches.opt_present("d");
    let top   = numopt(matches.opt_str("t"), 1usize);
    let swaps = numopt(matches.opt_str("s"), 3usize);

    match command.as_ref() {
        "run" => run(&corpus[..], layout, debug, top, swaps),
        "run-ref" => run_ref(&corpus[..]),
        "refine" => refine(&corpus[..], layout, debug, top, swaps),
        _ => print_usage(progname, opts),
    };
}

fn run(s: &str, layout: &layout::Layout, debug: bool, top: usize, swaps: usize)
{
    notify("Starting run");

    let penalties = penalty::init();
    let init_pos_map = layout::SHAKA_LAYOUT.get_position_map();
    let quartads = penalty::prepare_quartad_list(s, &init_pos_map);
    let len = s.len();

    //loop {
        simulator::simulate(&quartads, len, layout, &penalties, debug, top, swaps);
    //}

    notify("Run finished");
}

fn run_ref(s: &str)
{
    notify("Starting reference run");
    let penalties = penalty::init();
    let init_pos_map = layout::INIT_LAYOUT.get_position_map();
    let quartads = penalty::prepare_quartad_list(s, &init_pos_map);
    let len = s.len();

    let penalty = penalty::calculate_penalty(&quartads, len, &layout::SHAKA_LAYOUT, &penalties, true);
    println!("Reference: SHAKA");
    simulator::print_result(&layout::SHAKA_LAYOUT, &penalty);
    println!("");

    let penalty = penalty::calculate_penalty(&quartads, len, &layout::INIT_LAYOUT, &penalties, true);
    println!("Reference: INITIAL");
    simulator::print_result(&layout::INIT_LAYOUT, &penalty);
    notify("Reference run finished");
}

fn refine(s: &str, layout: &layout::Layout, debug: bool, top: usize, swaps: usize)
{
    notify("Starting refining");
    let penalties = penalty::init();
    let init_pos_map = layout::SHAKA_LAYOUT.get_position_map();
    let quartads = penalty::prepare_quartad_list(s, &init_pos_map);
    let len = s.len();

    simulator::refine(&quartads, len, layout, &penalties, debug, top, swaps);
    notify("Refining finished");
}

fn print_usage(progname: &String, opts: Options)
{
    let brief = format!("Usage: {} (run|run-ref) <corpus> [OPTIONS]", progname);
    print!("{}", opts.usage(&brief));
}

fn numopt<T>(s: Option<String>, default: T)
-> T
where T: std::str::FromStr + std::fmt::Display
{
    match s {
        None => default,
        Some(num) => match num.parse::<T>() {
            Ok(n) => n,
            Err(_) => {
                println!("Error: invalid option value {}. Using default value {}.", num, default);
                default
            },
        },
    }
}

fn notify(message: &str)
{
    Command::new("notify-send").args(&[message]).output().expect("failed to execute process");
}
