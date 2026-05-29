use thiserror::Error;

use crate::git::errors::GitError;

#[derive(Error, Debug)]
pub enum PipelineError {
    #[error("Precondition failed: {0}:\n  {1:?}")]
    PreconditionFailed(String, Option<GitError>),
    #[error("Execution failed:\n  {0}")]
    ExecutionFailed(String),
}
