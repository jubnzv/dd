//! Top pass removes top-level statements from the given program using delta-debugging tecnhique.
use super::Pass;
use crate::app::App;
use crate::delta;
use crate::error::Error;
use crate::treesitter;
use crate::treesitter::Lua;
use std::rc::Rc;

pub struct PassTop<'app> {
    app: &'app App,
    source_code: Option<String>,
    ts_language: Option<Rc<dyn treesitter::Parser>>,
}

impl<'app> PassTop<'app> {
    pub fn from_app(app: &'app App) -> Result<Self, Error> {
        Ok(PassTop {
            app,
            source_code: None,
            ts_language: None,
        })
    }
}

impl<'app> Pass<'app> for PassTop<'app> {
    fn name(&self) -> String {
        "Top".to_string()
    }

    fn temp_dir(&self) -> String {
        return String::from(
            std::path::PathBuf::from(&self.app.output_dir)
                .join(&self.name())
                .to_string_lossy(),
        );
    }

    fn app(&self) -> &App {
        self.app
    }

    fn source_code(&self) -> String {
        self.source_code.as_ref().unwrap().clone()
    }

    fn language(&self) -> Rc<dyn treesitter::Parser> {
        self.ts_language.as_ref().unwrap().clone()
    }

    fn run(&mut self, source_code: Option<&str>) -> Result<String, Error> {
        self.source_code = Some(self.read_source(source_code)?);
        self.ts_language = Some(Rc::new(Lua::new(&self.source_code())?));
        let language = self.language();
        let ast_root = language.ast_root();
        let top_nodes = language.children(ast_root);
        log::debug!("Bisecting {} top nodes", top_nodes.len());
        match delta::ddmin(&top_nodes, self) {
            Ok((_, source)) => Ok(source),
            Err(err) => Err(err),
        }
    }
}
