use crate::app::App;
use crate::error::Error;
use crate::passes::imports::PassImports;
use crate::passes::top::PassTop;
use crate::passes::Pass;
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, path};

mod rc {
    pub const SUCCESS: i32 = 0;
    pub const FAILURE: i32 = 1;
}

type PassInst<'a> = Rc<RefCell<dyn Pass<'a> + 'a>>;

fn create_dir(path: &str) -> Result<(), String> {
    match fs::create_dir_all(path) {
        Err(err) => return Err(format!("Cannot create directory '{}': {}", path, err,)),
        Ok(_) => Ok(()),
    }
}

/// Creates the required temporary directories.
fn prepare_out_dirs<'a>(app: &App, passes: &[PassInst<'a>]) -> Result<(), String> {
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
        create_dir(&p.borrow().temp_dir())?;
    }

    Ok(())
}

/// Runs application with the given configuration. Returns reduced source on success.
pub fn run_app<'a>(app: &'a App) -> Result<String, Error> {
    let mut passes: Vec<PassInst<'a>> = vec![];
    if app.passes.imports {
        match PassImports::from_app(app) {
            Ok(p) => passes.push(Rc::new(RefCell::new(p))),
            Err(err) => {
                return Err(Error::new(format!(
                    "Cannot initialize PassImports pass: {}",
                    err
                )))
            }
        }
    }
    if app.passes.top {
        match PassTop::from_app(app) {
            Ok(p) => passes.push(Rc::new(RefCell::new(p))),
            Err(err) => {
                return Err(Error::new(format!(
                    "Cannot initialize PassTop pass: {}",
                    err
                )))
            }
        }
    }

    prepare_out_dirs(app, &passes)?;

    let mut source: Option<String> = None;
    for p in passes.iter() {
        let result = match &source {
            Some(s) => p.borrow_mut().run(Some(s)),
            None => p.borrow_mut().run(None),
        };
        match result {
            Ok(reduced_source) => {
                log::debug!("Reduced source: {}", &reduced_source);
                source = Some(reduced_source);
            }
            Err(Error::NoChange) => log::debug!("Source code has not been reduced"),
            Err(err) => return Err(err),
        };
    }

    source.ok_or(Error::NoChange)
}

pub fn run() -> i32 {
    env_logger::init();
    let app = match App::from_args() {
        Ok(app) => app,
        Err(err) => {
            eprintln!("{}", err);
            return rc::FAILURE;
        }
    };
    match run_app(&app) {
        Ok(_) => rc::SUCCESS,
        Err(Error::NoChange) => {
            println!("Cannot reproduce the failure");
            rc::SUCCESS
        }
        Err(err) => {
            eprintln!("{}", err);
            rc::FAILURE
        }
    }
}
