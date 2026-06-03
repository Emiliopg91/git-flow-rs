use thiserror::Error;

use fwkarq::shell::errors::ShellError;

#[derive(Error, Debug)]
pub enum GitError {
    #[error("Command '{0}' failed:\n  {1}")]
    CommandFailed(String, ShellError),

    #[error("Could not merge {0} branch:\n  {1}")]
    MergeFailed(String, ShellError),

    #[error("Could not checkout {0} branch:\n  {1}")]
    CheckoutFailed(String, ShellError),

    #[error("Could not pull branch from remote:\n  {0}")]
    PullFailed(ShellError),

    #[error("Could not push branch to remote:\n  {0}")]
    PushFailed(ShellError),

    #[error("Could not push tags:\n  {0}")]
    PushTagsFailed(ShellError),

    #[error("Could not create {0} branch:\n  {1}")]
    BranchFailed(String, ShellError),

    #[error("Could not delete local branch {0}:\n  {1}")]
    BranchDeletionFailed(String, ShellError),

    #[error("Could not list local branches:\n  {0}")]
    ListBranchFailed(ShellError),

    #[error("Could not commit local branch:\n  {0}")]
    CommitFailed(ShellError),

    #[error("Could not create tag {0}:\n  {1}")]
    TagFailed(String, ShellError),

    #[error("Could not fetch remotes:\n  {0}")]
    FetchFailed(ShellError),

    #[error("Could not get remote URL:\n  {0}")]
    RemoteUrlFailed(ShellError),

    #[error("No repository found in {0}:\n  {1}")]
    NotARepository(String, ShellError),
}
