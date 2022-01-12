//! Imports pass sequentially removes `import` statements from the top-level of the program.
use super::Pass;
use crate::app::App;
use crate::delta;
use crate::treesitter;
use crate::treesitter::Lua;
use std::rc::Rc;

pub struct PassImports<'app> {
    app: &'app App,
    original_source: String,
    ts_language: Rc<dyn treesitter::Parser>,
}

impl<'app> PassImports<'app> {
    pub fn from_app(app: &'app App) -> Result<PassImports, String> {
        let source_code = match std::fs::read_to_string(&app.file) {
            Ok(source) => source,
            Err(err) => return Err(format!("{}", err)),
        };
        let lua = match Lua::new(&source_code) {
            Ok(lua) => lua,
            Err(err) => return Err(err),
        };
        let ts_language = Rc::new(lua);
        Ok(PassImports {
            app,
            original_source: source_code,
            ts_language,
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

    fn original_source(&self) -> String {
        self.original_source.clone()
    }

    fn language(&self) -> Rc<dyn treesitter::Parser> {
        self.ts_language.clone()
    }

    fn run(&self) -> Result<String, String> {
        let require_nodes = self.ts_language.get_matches(
            &self.original_source,
            self.ts_language.imports_query(),
            Some(|c| c.node.kind() == "function_call"),
        );
        log::debug!(
            "Bisecting import nodes: {:?}",
            require_nodes
                .iter()
                .map(|n| { crate::treesitter::node_source(&self.original_source, n) })
                .collect::<Vec<String>>()
        );
        match delta::ddmin(&require_nodes, self) {
            Ok((_, source)) => Ok(source),
            Err(err) => Err(err),
        }
    }
}
