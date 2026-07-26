use anyhow::{Context, Result};
use std::process::Command;

pub fn get_commit_message_from_hash(commit_hash: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["show", "--format=%B", "-s", commit_hash])
        .output()
        .with_context(|| format!("failed to execute git show for hash {}", commit_hash))?;

    if !output.status.success() {
        anyhow::bail!("failed to retrieve commit message for hash {}", commit_hash);
    }

    let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(message)
}

/// Returns commit messages for the range `from_hash..=to_hash`, inclusive on both ends,
/// in chronological (oldest-first) order. Merge commits are excluded via `--no-merges`.
pub fn get_commit_messages_from_hash_range(from_hash: &str, to_hash: &str) -> Result<Vec<String>> {
    let range = if is_orphan(from_hash)? {
        to_hash.to_string()
    } else {
        format!("{}^..{}", from_hash, to_hash)
    };

    let output = Command::new("git")
        .args([
            "log",
            "--pretty=format:%B%x00", // null byte for delimiter
            "--reverse",
            &range,
        ])
        .output()
        .with_context(|| {
            format!(
                "failed to execute git log for range {}, {}",
                from_hash, to_hash
            )
        })?;

    if !output.status.success() {
        anyhow::bail!(
            "failed to retrieve commit messages for range {}, {}",
            from_hash,
            to_hash
        );
    }

    let messages: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .map(|s| s.trim()) // trim removes the trailing newline git adds to %B
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    Ok(messages)
}

pub fn is_orphan(hash: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-list", "--parents", "-n", "1", hash])
        .output()
        .with_context(|| format!("failed to check parent for hash {}", hash))?;

    if !output.status.success() {
        anyhow::bail!("failed to check parent for hash {}", hash);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.split_whitespace().count() == 1)
}
