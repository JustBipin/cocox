#[derive(Debug, Clone, Copy, Default)]
pub struct OutputConfig {
    pub quiet: bool,
    pub verbose: bool,
}

impl OutputConfig {
    pub fn new(quiet: bool, verbose: bool) -> Self {
        Self { quiet, verbose }
    }
}
