use std::sync::{LazyLock, Mutex};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OutputConfig {
    pub quiet: bool,
    pub verbose: bool,
}

impl OutputConfig {
    pub fn new(quiet: bool, verbose: bool) -> Self {
        Self { quiet, verbose }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Config {
    pub output: OutputConfig,
    pub skip_detail: bool,
    pub hide_input: bool,
    pub strip_comments: bool,
    pub max_header_length: Option<usize>,
}

static CONFIG: LazyLock<Mutex<Config>> = LazyLock::new(|| Mutex::new(Config::default()));

pub fn config() -> Config {
    *CONFIG.lock().unwrap()
}

pub fn set_config(new_config: Config) {
    *CONFIG.lock().unwrap() = new_config;
}

pub fn update_config(f: impl FnOnce(&mut Config)) {
    f(&mut CONFIG.lock().unwrap());
}

/// Restores the previous config when dropped. Intended for tests.
pub struct ConfigGuard {
    previous: Config,
}

impl ConfigGuard {
    pub fn set(new_config: Config) -> Self {
        let previous = config();
        set_config(new_config);
        Self { previous }
    }
}

impl Drop for ConfigGuard {
    fn drop(&mut self) {
        set_config(self.previous);
    }
}
