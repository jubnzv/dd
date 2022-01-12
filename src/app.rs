pub struct PassesConfig {
    pub imports: bool,
    pub top: bool,
}

mod args {
    pub const SCRIPT: &str = "SCRIPT";
    pub const FILE: &str = "FILE";
    pub const PASSES: &str = "PASSES";
    pub const OUTPUT: &str = "OUTPUT";
    pub const TIMEOUT: &str = "TIMEOUT";
    pub const FORCE: &str = "FORCE";
    pub const RECURSIVE: &str = "RECURSIVE";
}

mod defaults {
    pub const OUTPUT_DIR: &str = "/tmp/dd/";
}

pub struct App {
    /// Absolute path to the script that checks failure.
    pub script: String,
    pub file: String,
    pub output_dir: String,
    pub timeout: Option<u32>,
    pub force: bool,
    pub recursive: bool,
    pub passes: PassesConfig,
}

/// Returns absolute path from the given `path`.
fn abs_path(path: &str) -> Result<String, String> {
    if (std::env::consts::OS == "windows" && path.starts_with('\\')) || path.starts_with('/') {
        return Ok(path.to_string());
    };
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(path.into_os_string().into_string().unwrap()),
        Err(err) => Err(err.to_string()),
    }
}

/// Returns passes configuraiton based on the given CLI argument content.
fn parse_passes(arg: Option<&str>) -> Result<PassesConfig, String> {
    if arg.is_none() {
        return Err("No passes enabled".to_string());
    }
    let mut passes = PassesConfig {
        imports: false,
        top: false,
    };
    for pass_name in arg.unwrap().split(';') {
        match pass_name {
            "imports" => passes.imports = true,
            "top" => passes.top = true,
            _ => return Err(format!("Unknown pass: {}", pass_name)),
        }
    }
    Ok(passes)
}

impl App {
    pub fn new() -> Result<App, String> {
        let matches = clap::App::new(env!("CARGO_PKG_NAME"))
            .version("1.0")
            .author("Georgiy Komarov <jubnzv@gmail.com>")
            .arg(
                clap::Arg::with_name(args::SCRIPT)
                    .help("Script that checks failure")
                    .required(true)
                    .index(1),
            )
            .arg(
                clap::Arg::with_name(args::FILE)
                    .help("Path to Lua file")
                    .required(true)
                    .index(2),
            )
            .arg(
                clap::Arg::with_name(args::PASSES)
                    .short("p")
                    .long("passes")
                    .help("Enabled passes")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name(args::OUTPUT)
                    .short("o")
                    .long("output")
                    .help("Path to created temporary directory")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name(args::TIMEOUT)
                    .short("t")
                    .long("timeout")
                    .help("Timeout to execute the script in seconds")
                    .takes_value(true),
            )
            .arg(
                clap::Arg::with_name(args::FORCE)
                    .short("f")
                    .long("force")
                    .help("Remove given temporary directory if exists")
                    .takes_value(false),
            )
            .arg(
                clap::Arg::with_name(args::RECURSIVE)
                    .short("r")
                    .long("recursive")
                    .help("Use the whole directory that contains Lua file")
                    .takes_value(false),
            )
            .arg(
                clap::Arg::with_name("v")
                    .short("v")
                    .multiple(true)
                    .help("Sets the level of verbosity"),
            )
            .get_matches();

        let script = match abs_path(&matches.value_of(args::SCRIPT).unwrap().to_string()) {
            Ok(path) => path,
            Err(err) => return Err(err),
        };

        Ok(App {
            file: matches.value_of(args::FILE).unwrap().to_string(),
            script,
            output_dir: matches
                .value_of(args::OUTPUT)
                .unwrap_or(defaults::OUTPUT_DIR)
                .to_string(),
            timeout: clap::value_t!(matches.value_of(args::TIMEOUT), u32).ok(),
            force: matches.is_present(args::FORCE),
            recursive: matches.is_present(args::RECURSIVE),
            passes: parse_passes(matches.value_of(args::PASSES))?,
        })
    }
}
