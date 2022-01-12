mod app;
mod delta;
mod passes;
mod treesitter;

use app::App;
use passes::Pass;
use std::process::exit;
use std::{fs, path};

mod rc {
    pub const SUCCESS: i32 = 0;
    pub const FAILURE: i32 = 1;
}

fn create_dir(path: &str) -> Result<(), String> {
    match fs::create_dir_all(path) {
        Err(err) => return Err(format!("Cannot create directory '{}': {}", path, err,)),
        Ok(_) => Ok(()),
    }
}

/// Creates the required temporary directories.
fn prepare_out_dirs<'a>(app: &App, passes: &[Box<dyn Pass + 'a>]) -> Result<(), String> {
    if path::Path::new(&app.output_dir).exists() {
        if app.force {
            if let Err(err) = fs::remove_dir_all(&app.output_dir) {
                return Err(format!(
                    "Cannot remove directory '{}': {}",
                    &app.output_dir, err
                ));
            }
        } else {
            return Err(format!(
                "Directory '{}' already exists. Use --force to remove it.",
                app.output_dir
            ));
        }
    };

    // Create main dd directory.
    create_dir(&app.output_dir)?;

    // Create directories for the passes.
    for p in passes.iter() {
        create_dir(&p.temp_dir())?;
    }

    Ok(())
}

/// Runs application with the given configuration.
fn run_app<'a>(app: &'a App) -> Result<(), String> {
    let mut passes: Vec<Box<dyn Pass + 'a>> = vec![];
    if app.passes.imports {
        match passes::imports::PassImports::from_app(app) {
            Ok(p) => passes.push(Box::new(p)),
            Err(err) => return Err(format!("Cannot initialize PassImports pass: {}", err)),
        }
    }
    if app.passes.top {
        match passes::top::PassTop::from_app(app) {
            Ok(p) => passes.push(Box::new(p)),
            Err(err) => return Err(format!("Cannot initialize PassTop pass: {}", err)),
        }
    }

    prepare_out_dirs(app, &passes)?;

    for p in passes.iter() {
        match p.run() {
            Ok(fail_cases) => println!("{:?}", fail_cases),
            Err(err) => return Err(err),
        }
    }

    Ok(())
}

fn run() -> i32 {
    env_logger::init();
    let app = match App::new() {
        Ok(app) => app,
        Err(err) => {
            println!("{}", err);
            return rc::FAILURE;
        }
    };

    match run_app(&app) {
        Ok(_) => rc::SUCCESS,
        Err(err) => {
            println!("{}", err);
            rc::FAILURE
        }
    }
}

fn main() {
    exit(run())
}
