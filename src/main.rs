mod app;
mod delta;
mod passes;
mod treesitter;

use passes::Pass;
use std::{fs, path};

// NOTE: This will be replaced to std::process::ExitCode when it is stable.
mod rc {
    pub const SUCCESS: i32 = 1;
    pub const FAILURE: i32 = 1;
}

fn create_dir(path: &str) -> Result<(), String> {
    match fs::create_dir_all(path) {
        Err(err) => return Err(format!("Cannot create directory '{}': {}", path, err,)),
        Ok(_) => Ok(()),
    }
}

fn prepare_out_dirs<'a>(app: &app::App, passes: &[impl Pass<'a>]) -> Result<(), String> {
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

fn run() -> i32 {
    env_logger::init();
    let app = match app::App::new() {
        Ok(app) => app,
        Err(err) => {
            println!("{}", err);
            return rc::FAILURE;
        }
    };

    let mut passes = vec![];
    if app.passes.modules {
        match passes::pass_imports::PassImports::from_app(&app) {
            Ok(pm) => passes.push(pm),
            Err(err) => println!("Cannot initialize pass: {}", err),
        }
    }

    match prepare_out_dirs(&app, &passes) {
        Ok(_) => (),
        Err(err) => {
            println!("{}", err);
            log::error!("{}", err);
            return rc::FAILURE;
        }
    };

    for p in passes.iter() {
        match p.run() {
            Ok(fail_cases) => println!("{:?}", fail_cases),
            Err(err) => println!("{}", err),
        }
    }

    rc::SUCCESS
}

fn main() {
    std::process::exit(run())
}
