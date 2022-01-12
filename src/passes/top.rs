//! Top pass removes top-level statements from the given program using delta-debugging tecnhique.
use super::Pass;
use crate::app::App;
use crate::delta;
use crate::treesitter;
use crate::treesitter::Lua;
use std::rc::Rc;

pub struct PassTop<'app> {
    app: &'app App,
    original_source: String,
    ts_language: Rc<dyn treesitter::Parser>,
}

impl<'app> PassTop<'app> {
    pub fn from_app(app: &'app App) -> Result<PassTop, String> {
        let source_code = match std::fs::read_to_string(&app.file) {
            Ok(source) => source,
            Err(err) => return Err(format!("{}", err)),
        };
        let lua = match Lua::new(&source_code) {
            Ok(lua) => lua,
            Err(err) => return Err(err),
        };
        let ts_language = Rc::new(lua);
        Ok(PassTop {
            app,
            original_source: source_code,
            ts_language,
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

    fn original_source(&self) -> String {
        self.original_source.clone()
    }

    fn language(&self) -> Rc<dyn treesitter::Parser> {
        self.ts_language.clone()
    }

    fn run(&self) -> Result<String, String> {
        let ast_root = self.ts_language.ast_root();
        let top_nodes = self.ts_language.children(ast_root);
        log::debug!("Bisecting {} top nodes", top_nodes.len());
        match delta::ddmin(&top_nodes, self) {
            Ok((_, source)) => Ok(source),
            Err(err) => Err(err),
        }
    }
}
