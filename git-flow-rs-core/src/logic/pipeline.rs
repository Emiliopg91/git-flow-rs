use std::time::Instant;

use tokio::sync::mpsc::Sender;

use crate::git::GitWrapper;

use super::errors::PipelineError;

pub enum Precondition {
    RequiresMissingBranch(String),
    RequiresExistingLocalBranch(String),
}

pub enum Step {
    CreateBranch(String),
    Checkout(String),
    Commit(String),
    DeleteBranch(String),
    Merge(String),
    Pull(),
    Push(),
    PushTags(),
    Tag(String),
}

pub struct Pipeline {
    requires: Vec<Precondition>,
    sender: Sender<String>,
    steps: Vec<Step>,
}

impl Pipeline {
    pub fn new(sender: Sender<String>) -> Self {
        Self {
            requires: Vec::new(),
            sender,
            steps: Vec::new(),
        }
    }

    pub fn step(&mut self, step: Step) -> &mut Pipeline {
        self.steps.push(step);
        self
    }

    pub fn requires(&mut self, precondition: Precondition) -> &mut Pipeline {
        self.requires.push(precondition);
        self
    }

    pub async fn run(&self) -> Result<(), PipelineError> {
        let t0 = Instant::now();

        Self::send(&self.sender, "  Starting pipeline").await;
        for precondition in &self.requires {
            match Self::execute_precondition(precondition, &self.sender).await {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
        for step in &self.steps {
            match Self::execute_step(step, &self.sender)
                .await
                .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))
            {
                Ok(_) => continue,
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Self::send(
            &self.sender,
            &format!(
                "  Pipeline finished after {:.3}",
                t0.elapsed().as_secs_f64()
            ),
        )
        .await;

        Ok(())
    }

    async fn send(sender: &Sender<String>, msg: &str) {
        let _ = sender.send(msg.to_string()).await;
    }

    async fn execute_precondition(
        precondition: &Precondition,
        sender: &Sender<String>,
    ) -> Result<(), PipelineError> {
        match precondition {
            Precondition::RequiresMissingBranch(branch) => {
                Self::send(
                    sender,
                    &format!("    Checking if branch is missing {}", branch),
                )
                .await;
                let local = GitWrapper::get_branches().await.map_err(|e| {
                    PipelineError::PreconditionFailed(
                        "Could not get local branches".to_string(),
                        Some(e),
                    )
                })?;
                let remotes = GitWrapper::get_remote_branches().await.map_err(|e| {
                    PipelineError::PreconditionFailed(
                        "Could not get remote branches".to_string(),
                        Some(e),
                    )
                })?;

                if local.contains(branch) || remotes.contains(branch) {
                    return Err(PipelineError::PreconditionFailed(
                        format!("Branch {} already exists", branch),
                        None,
                    ));
                }

                Ok(())
            }
            Precondition::RequiresExistingLocalBranch(branch) => {
                Self::send(sender, &format!("    Checking branch {} exists", branch)).await;
                let local = GitWrapper::get_branches().await.map_err(|e| {
                    PipelineError::PreconditionFailed(
                        "Could not get local branches".to_string(),
                        Some(e),
                    )
                })?;
                if !local.contains(branch) {
                    return Err(PipelineError::PreconditionFailed(
                        format!("Branch {} not found", branch),
                        None,
                    ));
                }
                Ok(())
            }
        }
    }

    async fn execute_step(step: &Step, sender: &Sender<String>) -> Result<(), PipelineError> {
        match step {
            Step::CreateBranch(branch) => {
                Self::send(sender, &format!("    Creating branch {}", branch)).await;
                GitWrapper::create_branch(branch)
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
            Step::Checkout(branch) => {
                if GitWrapper::get_branch().await != *branch {
                    Self::send(sender, &format!("    Checking out {} branch", branch)).await;
                    GitWrapper::checkout(branch)
                        .await
                        .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
                }
            }
            Step::Commit(message) => {
                Self::send(sender, &format!("    Creating commit '{}'", message)).await;
                GitWrapper::commit(message)
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
            Step::DeleteBranch(branch) => {
                Self::send(sender, &format!("    Deleting {} branch", branch)).await;
                GitWrapper::delete_branch(branch)
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
            Step::Merge(branch) => {
                Self::send(sender, &format!("    Merging {} branch", branch)).await;
                GitWrapper::merge(branch)
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
            Step::Pull() => {
                if GitWrapper::get_remote_branches()
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?
                    .contains(&GitWrapper::get_branch().await)
                {
                    Self::send(sender, "    Pulling from remote").await;
                    GitWrapper::pull()
                        .await
                        .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
                }
            }
            Step::Push() => {
                Self::send(sender, "    Push to remote").await;
                GitWrapper::push()
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
            Step::PushTags() => {
                Self::send(sender, "    Push tags to remote").await;
                GitWrapper::push_tags()
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
            Step::Tag(tag) => {
                Self::send(sender, &format!("    Creating tag {}", tag)).await;
                GitWrapper::tag(tag)
                    .await
                    .map_err(|e| PipelineError::ExecutionFailed(e.to_string()))?;
            }
        }

        Ok(())
    }
}
