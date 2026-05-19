use assert_cmd::Command;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use tempfile::TempDir;

static SERIAL: Mutex<()> = Mutex::new(());
pub struct TestRepo {
    dir: TempDir,
    original_dir: PathBuf,
    _guard: MutexGuard<'static, ()>,
}

impl TestRepo {
    /// Initialize a temporary test repo with git config
    ///Change working directory to that repo
    /// working directory resets to project/original repo when out of scope.
    pub fn new() -> Self {
        let guard = SERIAL.lock().unwrap();
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path();
        let original_dir = env::current_dir().expect("failed to get current dir");

        // dir with git config
        run_git(path, &["init"]);
        run_git(path, &["config", "user.email", "test@test.com"]);
        run_git(path, &["config", "user.name", "Test"]);

        // set working directory to temorary git repo
        env::set_current_dir(path).expect("failed to chdir into test repo");

        Self {
            dir,
            original_dir,
            _guard: guard,
        }
    }

    pub fn commit(&self, message: &str) -> String {
        let path = self.dir.path();
        let file = path.join("f.txt");
        fs::write(&file, message).unwrap();
        run_git(path, &["add", "."]);
        run_git(path, &["commit", "-m", message]);
        git_output(path, &["rev-parse", "HEAD"])
    }
}

// set current dir to project repo when out of scope
impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);
    }
}

fn run_git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git command failed");
}

fn git_output(dir: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git command failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
