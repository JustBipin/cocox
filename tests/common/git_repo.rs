use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub struct TestRepo {
    dir: TempDir,
    original_dir: PathBuf,
}

impl TestRepo {
    /// Initialize a temporary test repo with git config.
    ///
    /// ## Important: Process-Global State
    /// This function changes the current working directory of the **entire process** /// to the temporary repository. Because Cargo runs tests in parallel threads within
    /// the same process, **any test using `TestRepo` must be marked with `#[serial]`** /// from the `serial_test` crate to prevent race conditions and cross-test interference.
    ///
    /// The working directory will automatically reset back to the original project
    /// directory when this `TestRepo` goes out of scope (via `Drop`).
    pub fn new() -> Self {
        let dir = TempDir::new().expect("failed to create temp dir");
        let path = dir.path();
        let original_dir = env::current_dir().expect("failed to get current dir");

        // dir with git config
        run_git(path, &["init"]);
        run_git(path, &["config", "user.email", "test@test.com"]);
        run_git(path, &["config", "user.name", "Test"]);

        // set working directory to temporary git repo
        env::set_current_dir(path).expect("failed to chdir into test repo");

        Self { dir, original_dir }
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
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("failed to execute git process");

    assert!(
        status.success(),
        "git command '{}' failed with {:#}",
        args.join(" "),
        status
    );
}

fn git_output(dir: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to execute git process");

    if !output.status.success() {
        panic!(
            "git command failed with status: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}
