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

pub fn get_commit_messages_from_hash_range(from_hash: &str, to_hash: &str) -> Result<Vec<String>> {
    let range = format!("{}..{}", from_hash, to_hash);

    // git's A..B range is exclusive of A (returns commits reachable from B
    // but not from A). To make the range inclusive we fetch A's message
    // separately and prepend it below.
    let from_hash_message = get_commit_message_from_hash(from_hash)?;

    let output = Command::new("git")
        .args([
            "log",
            "--no-merges",
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

    let mut messages: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .map(|s| s.trim()) // trim removes the trailing newline git adds to %B
        .filter(|s| !s.is_empty())
        .map(|s| s.to_owned())
        .collect();

    // prepend the first message of the range
    messages.insert(0, from_hash_message);

    Ok(messages)
}
