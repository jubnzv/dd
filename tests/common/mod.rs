use dd::app::parse_passes;
use dd::app::App;
use dd::driver::run_app;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::{tempdir, TempDir};

mod test_settings {
    pub const SHELL: &str = "/bin/bash";
}

pub(crate) struct Test {
    app: App,
    file_tempdir: TempDir,
    script_tempdir: TempDir,
}

impl Test {
    pub(crate) fn new() -> Self {
        let mut app = App::new();
        app.force = true;
        app.output_dir = tempdir().unwrap().path().display().to_string();
        Test {
            app,
            file_tempdir: tempdir().unwrap(),
            script_tempdir: tempdir().unwrap(),
        }
    }

    pub(crate) fn script(mut self, script: &str) -> Self {
        self.app.script = self
            .file_tempdir
            .path()
            .join("script")
            .display()
            .to_string();
        fs::write(
            &self.app.script,
            format!("#!{}\n{}", test_settings::SHELL, script),
        )
        .unwrap();
        fs::set_permissions(&self.app.script, fs::Permissions::from_mode(0o755)).unwrap();
        self
    }

    pub(crate) fn source(mut self, source: &str) -> Self {
        self.app.file = self.file_tempdir.path().join("in").display().to_string();
        fs::write(&self.app.file, source).unwrap();
        self
    }

    pub(crate) fn timeout(mut self, timeout: u32) -> Self {
        self.app.timeout = Some(timeout);
        self
    }

    pub(crate) fn passes(mut self, passes_config: &str) -> Self {
        self.app.passes = parse_passes(Some(passes_config)).unwrap();
        self
    }

    pub(crate) fn run(self) -> Result<String, String> {
        run_app(&self.app)
    }

    pub(crate) fn check(self, expected: &str) {
        match self.run() {
            Err(err) => panic!("Error while running the test: {}", err),
            Ok(got) => assert_eq!(got, expected),
        }
    }
}
