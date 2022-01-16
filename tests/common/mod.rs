use dd::app::parse_passes;
use dd::app::App;
use dd::driver::run_app;
use dd::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::{tempdir, TempDir};

mod test_settings {
    pub const SHELL: &str = "/bin/bash";
}

pub(crate) struct Test {
    app: App,
    file_tempdir: TempDir,
    _script_tempdir: TempDir,
}

impl Test {
    pub(crate) fn new() -> Self {
        let mut app = App::new();
        app.force = true;
        app.output_dir = tempdir().unwrap().path().display().to_string();
        Test {
            app,
            file_tempdir: tempdir().unwrap(),
            _script_tempdir: tempdir().unwrap(),
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

    #[allow(dead_code)]
    pub(crate) fn timeout(mut self, timeout: u32) -> Self {
        self.app.timeout = Some(timeout);
        self
    }

    pub(crate) fn passes(mut self, passes_config: &str) -> Self {
        self.app.passes = parse_passes(Some(passes_config)).unwrap();
        self
    }

    fn run(self) -> Result<String, Error> {
        run_app(&self.app)
    }

    pub(crate) fn check_reduced(self, expected: &str) {
        match self.run() {
            Err(err) => panic!("Error while running the test: {}", err),
            Ok(got) => assert_eq!(got.replace("\n", ""), expected.replace("\n", "")),
        }
    }

    pub(crate) fn check_not_reduced(self) {
        match self.run() {
            Err(Error::NoChange) => (),
            Err(err) => panic!("Error while running the test: {}", err),
            Ok(src) => panic!("Source code has been reduced: {}", src),
        }
    }
}
