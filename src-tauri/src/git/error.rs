use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("{message}")]
    CommandFailed {
        message: String,
        hint: Option<String>,
    },
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("Merge conflict")]
    MergeConflict,
    #[error("Nothing to commit")]
    NothingToCommit,
    #[error("Repository not found at {0}")]
    RepoNotFound(String),
}

impl Serialize for GitError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub fn classify_git_error(stderr: &str) -> GitError {
    let s = stderr.to_lowercase();
    if s.contains("authentication failed") {
        GitError::AuthenticationFailed(
            "Check credentials: run `git config credential.helper`".into(),
        )
    } else if s.contains("permission denied (publickey)") {
        GitError::AuthenticationFailed("SSH key not found. Run `ssh-add`".into())
    } else if s.contains("could not resolve host") {
        GitError::CommandFailed {
            message: stderr.trim().to_string(),
            hint: Some("Check network connection and remote URL".into()),
        }
    } else if s.contains("non-fast-forward") {
        GitError::CommandFailed {
            message: stderr.trim().to_string(),
            hint: Some("Remote has new commits. Pull first, or use force push".into()),
        }
    } else if s.contains("nothing to commit") {
        GitError::NothingToCommit
    } else if s.contains("conflict") && s.contains("merge") {
        GitError::MergeConflict
    } else if s.contains("not a git repository") {
        GitError::RepoNotFound("Directory is not a git repository".into())
    } else {
        GitError::CommandFailed {
            message: stderr.trim().to_string(),
            hint: None,
        }
    }
}
