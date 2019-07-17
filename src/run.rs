use structopt::clap::Shell;
use structopt::StructOpt;
use strum_macros::{EnumString, EnumVariantNames};

use std::path::PathBuf;

/// Whether colors should be displayed in the terminal
#[derive(Debug, Eq, PartialEq, EnumString, EnumVariantNames)]
#[strum(serialize_all = "kebab_case")]
enum Color {
    /// Show colors if supported by the terminal
    Auto,
    /// Always try and use colored text
    Always,
    /// Never try to use colored text
    Never
}

/// A polite task runner
#[derive(StructOpt, Debug)]
#[structopt(name = "please")]
struct Opt {
    // Flags

    /// Print what please would do without doing it
    #[structopt(long = "dry-run")]
    dry_run: bool, 

    /// Print entire pleasefile
    #[structopt(long)]
    dump: bool,

    /// Print evaluated variables
    #[structopt(long)]
    evaluate: bool,

    /// Highlight echoed recipe lines in bold
    #[structopt(long)]
    highlight: bool,

    /// List available recipes and their arguments
    #[structopt(short, long)]
    list: bool,

    /// Suppress all output
    #[structopt(short, long)]
    quiet: bool,

    /// List names of available recipes
    #[structopt(long)]
    summary: bool,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    // Options

    /// Print colorful output
    #[structopt(long, name = "COLOR", raw(possible_values = "&Color::variants()"), case_insensitive = true, default_value = "auto")]
    color: Color,

    /// Set <VARIABLE> to <VALUE>
    #[structopt(long = "set", raw(number_of_values = "2", value_names = "&[\"VARIABLE\", \"VALUE\"]"))]
    vars: Vec<String>,

    /// Show information about <RECIPE>
    #[structopt(long, short, name = "RECIPE")]
    show: Option<String>,

    /// Use <WORKING-DIRECTORY> as working directory.
    #[structopt(long, short = "d", name = "WORKING-DIRECTORY", parse(from_os_str))]
    working_directory: Option<PathBuf>,

    // Arguments
    /// The recipe(s) to run, defaults to the first recipe in the pleasefile
    #[structopt(name = "ARGUMENTS")]
    arguments: Vec<String>,
}

/// Runs the program
pub fn run() {
    Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, "target");
    let opt = Opt::from_args();

    if opt.color != Color::Auto {
        console::set_colors_enabled(opt.color == Color::Always);
    }

    let log_level = match opt.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    log::set_logger(&loggy::Loggy {
        prefix: "please",
        show_time: false,
    }).expect("unable to set up logger");
    log::set_max_level(log_level);

    log::info!("Looking for pleasefile...");

    let cyan = console::Style::new().cyan();

    println!("{}", cyan.apply_to(&format!("{:#?}", opt)));
}