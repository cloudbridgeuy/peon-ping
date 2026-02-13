use thiserror::Error;

pub type Result<T> = std::result::Result<T, ReleaseError>;

#[derive(Error, Debug)]
pub enum ReleaseError {
    #[error("Not on main branch. Switch to main before releasing.")]
    NotOnMainBranch,

    #[error("Working directory is not clean. Commit or stash changes first.")]
    DirtyWorkingDir,

    #[error("CI checks have not passed on main. Push and wait for CI first.")]
    CiNotPassed,

    #[error("Invalid version format: {0}. Expected semver (e.g. 2.1.0)")]
    InvalidVersion(String),

    #[error("New version {new} must be greater than current version {current}")]
    VersionNotBumped { current: String, new: String },

    #[error("Command not found: {command}\n{help}")]
    CommandNotFound { command: String, help: String },

    #[error("Git operation failed: {0}")]
    GitFailed(String),

    #[error("Workflow failed after {0} retries")]
    WorkflowFailed(u32),

    #[error("Workflow timed out after {0} seconds")]
    WorkflowTimeout(u64),

    #[error("Release aborted by user")]
    Aborted,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
