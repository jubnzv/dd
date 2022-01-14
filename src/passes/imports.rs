//! Imports pass sequentially removes `import` statements from the top-level of the program.
use super::Pass;
use crate::app::App;
use crate::delta;
use crate::treesitter;
use crate::treesitter::Lua;
use std::rc::Rc;

pub struct PassImports<'app> {
    app: &'app App,
    source_code: Option<String>,
    ts_language: Option<Rc<dyn treesitter::Parser>>,
}

impl<'app> PassImports<'app> {
    pub fn from_app(app: &'app App) -> Result<PassImports, String> {
        Ok(PassImports {
            app,
            source_code: None,
            ts_language: None,
        })
    }
}

impl<'app> Pass<'app> for PassImports<'app> {
    fn name(&self) -> String {
        "Imports".to_string()
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

    fn run(&mut self, source_code: Option<&str>) -> Result<String, String> {
        self.source_code = Some(self.read_source(source_code)?);
        self.ts_language = Some(Rc::new(Lua::new(&self.source_code())?));
        let language = self.language();
        let require_nodes = language.get_matches(
            &self.source_code(),
            self.language().imports_query(),
            Some(|c| c.node.kind() == "function_call"),
        );
        log::debug!(
            "Bisecting import nodes: {:?}",
            require_nodes
                .iter()
                .map(|n| { crate::treesitter::node_source(&self.source_code(), n) })
                .collect::<Vec<String>>()
        );
        match delta::ddmin(&require_nodes, self) {
            Ok((_, source)) => Ok(source),
            Err(err) => Err(err),
        }
    }
}
