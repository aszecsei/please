use failure::ResultExt;

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

/// a polite task runner.
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

    /// Generates completion for the specified shell
    #[structopt(long, name = "SHELL", raw(possible_values = "&Shell::variants()"), case_insensitive = true)]
    completions: Option<Shell>,

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
    #[structopt(long = "working-directory", short = "d", name = "WORKING-DIRECTORY", parse(from_os_str))]
    working_directory: Option<PathBuf>,

    // Arguments
    /// The recipe(s) to run, defaults to the first recipe in the pleasefile
    #[structopt(name = "ARGUMENTS")]
    arguments: Vec<String>,
}

/// Runs the program
pub fn run() -> Result<(), failure::Error> {
    human_panic::setup_panic!();
    let opt = Opt::from_args();

    if let Some(shell) = opt.completions {
        Opt::clap().gen_completions_to(
            env!("CARGO_PKG_NAME"),
            shell,
            &mut std::io::stdout()
        );
        return Ok(());
    }

    if opt.color != Color::Auto {
        console::set_colors_enabled(opt.color == Color::Always);
    }

    let log_level = match opt.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    
    let file_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(fern::log_file("please.log")?)
        .level(log::LevelFilter::Trace);

    let stderr_config = fern::Dispatch::new()
        .format(|out, message, record| {
            let subtle = console::Style::new().dim();
            let level_style = match record.level() {
                log::Level::Error => console::Style::new().red().reverse(),
                log::Level::Warn => console::Style::new().yellow(),
                log::Level::Info => console::Style::new().blue(),
                log::Level::Debug => console::Style::new().magenta(),
                log::Level::Trace => console::Style::new().cyan(),
            };
            let level_part = level_style.apply_to(format!("[{}]", record.level()));
            let file_part = if record.level() >= log::Level::Debug || cfg!(debug_assertions) {
                subtle.apply_to(format!("\t{}\t> {}:{}\t> ", record.target(), record.file().unwrap_or(""), record.line().unwrap_or(0)))
            } else {
                subtle.apply_to(String::from("\t"))
            };
            out.finish(format_args!(
                "{}{}{}",
                level_part,
                file_part,
                message,
            ))
        })
        .level(log_level)
        .chain(std::io::stderr());
    
    fern::Dispatch::new()
        .chain(file_config)
        .chain(stderr_config)
        .apply()
        .with_context(|_| "Unable to instantiate logger")?;

    log::debug!("{:#?}", opt);

    log::info!("Looking for pleasefile...");

    let mut parsed_files = Vec::new();
    let mut cwd = std::env::current_dir()
        .with_context(|_| "Unable to read current directory")?;
    loop {
        let filename = cwd.join("please");

        log::debug!("Looking for {:?}", filename);

        let file = std::fs::read_to_string(filename);
        if let Ok(file) = file {
            // TODO: Parse file

            parsed_files.push(file);
        }

        let try_parent = cwd.parent();

        if try_parent.is_none() {
            break;
        }
        cwd = try_parent.unwrap().to_path_buf();
    }

    log::info!("Parsed {} files", parsed_files.len());

    Ok(())
}