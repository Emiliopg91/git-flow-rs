use std::error::Error;

use crate::git::GitWrapper;

pub mod git;
pub mod logic;
pub mod others;

pub async fn initialize_and_validate() -> Result<(), Box<dyn Error>> {
    let has_changes = GitWrapper::has_changes().await?;
    if has_changes {
        eprintln!("Found uncommited changes on repository:");
        eprintln!("Fix it and try again.");
    }

    Ok(())
}
