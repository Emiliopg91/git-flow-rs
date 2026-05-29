pub mod errors;

use std::{ffi::OsStr, path::Path};

use fwkarq::shell::{Shell, ShellOutput};

use self::errors::GitError;

pub struct GitWrapper {}

pub async fn run_shell_command<I, S>(
    executable: &str,
    args: I,
    check: bool,
) -> fwkarq::shell::errors::Result<ShellOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Shell::command(executable)
        .unwrap()
        .args(args)
        .run(check)
        .await
}

impl GitWrapper {
    pub async fn check_if_repository<P>(path: P) -> bool
    where
        P: AsRef<Path>,
    {
        let res = Shell::command("git")
            .unwrap()
            .cwd(path)
            .args(["status"])
            .run(true)
            .await;

        res.is_ok()
    }

    pub async fn has_changes() -> Result<bool, GitError> {
        Ok(!run_shell_command("git", ["status", "--porcelain"], true)
            .await
            .map_err(|e| GitError::CommandFailed("git status --porcelain".to_string(), e))?
            .stdout
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .is_empty())
    }

    pub async fn get_branch() -> String {
        let res2 = run_shell_command("git", ["rev-parse", "--abbrev-ref", "HEAD"], true)
            .await
            .unwrap_or(ShellOutput {
                elapsed: 0 as f64,
                status: 0,
                stdout: "main".to_string(),
                stderr: "".to_string(),
            });
        res2.stdout.trim().to_string()
    }

    pub async fn get_main_branch() -> String {
        let res2 = run_shell_command("git", ["symbolic-ref", "refs/remotes/origin/HEAD"], true)
            .await
            .unwrap_or(ShellOutput {
                elapsed: 0 as f64,
                status: 0,
                stdout: "main".to_string(),
                stderr: "".to_string(),
            });
        let output = res2.stdout.trim();
        output.split("/").last().unwrap().to_string()
    }

    pub async fn checkout(branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["checkout", branch], true)
            .await
            .map_err(|e| GitError::CheckoutFailed(branch.into(), e))?;
        Ok(())
    }

    pub async fn merge(branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["merge", branch], true)
            .await
            .map_err(|e| GitError::MergeFailed(branch.into(), e))?;

        Ok(())
    }

    pub async fn pull() -> Result<(), GitError> {
        run_shell_command("git", ["pull"], true)
            .await
            .map_err(GitError::PullFailed)?;

        Ok(())
    }

    pub async fn push() -> Result<(), GitError> {
        let cur_branch = Self::get_branch().await;
        let mut args = ["push".to_string()].to_vec();

        if !Self::get_remote_branches().await?.contains(&cur_branch) {
            args.push("--set-upstream".to_string());
            args.push("origin".to_string());
            args.push(cur_branch.clone());
        }

        run_shell_command("git", &args, true)
            .await
            .map_err(GitError::PushFailed)?;

        Ok(())
    }

    pub async fn push_tags() -> Result<(), GitError> {
        run_shell_command("git", ["push".to_string(), "--tags".to_string()], true)
            .await
            .map_err(GitError::PushTagsFailed)?;

        Ok(())
    }

    pub async fn commit(msg: &str) -> Result<(), GitError> {
        run_shell_command("git", ["commit", "--allow-empty", "-m", msg], true)
            .await
            .map_err(GitError::CommitFailed)?;

        Ok(())
    }

    pub async fn tag(name: &str) -> Result<(), GitError> {
        run_shell_command("git", ["tag", name], true)
            .await
            .map_err(|e| GitError::TagFailed(name.to_string(), e))?;

        Ok(())
    }

    pub async fn fetch(all: bool, prune: bool) -> Result<(), GitError> {
        let mut args = ["fetch".to_string()].to_vec();

        if all {
            args.push("--all".to_string());
        }

        if prune {
            args.push("--prune".to_string());
        }

        run_shell_command("git", &args, true)
            .await
            .map_err(GitError::FetchFailed)?;

        Ok(())
    }

    pub async fn create_branch(branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["checkout", "-b", branch], true)
            .await
            .map_err(|e| GitError::BranchFailed(branch.to_string(), e))?;
        Ok(())
    }

    pub async fn delete_branch(branch: &str) -> Result<(), GitError> {
        run_shell_command("git", ["branch", "-D", branch], true)
            .await
            .map_err(|e| GitError::BranchDeletionFailed(branch.to_string(), e))?;

        Ok(())
    }

    pub async fn get_branches() -> Result<Vec<String>, GitError> {
        let res = run_shell_command("git", ["branch"], true)
            .await
            .map_err(GitError::ListBranchFailed)?;

        Ok(res
            .stdout
            .lines()
            .map(|s| s.replace("* ", "").trim().to_string())
            .collect())
    }

    pub async fn get_remote_branches() -> Result<Vec<String>, GitError> {
        let res = run_shell_command("git", ["branch", "-r"], true)
            .await
            .map_err(GitError::ListBranchFailed)?;

        Ok(res
            .stdout
            .lines()
            .map(|s| {
                s.replace("* ", "")
                    .replace("origin/", "")
                    .trim()
                    .to_string()
            })
            .collect())
    }

    fn filter_local_branches(branches: &[String], tpe: &String) -> Vec<String> {
        branches
            .iter()
            .filter_map(|s| {
                let cleaned = s.trim().trim_start_matches('*').trim();

                let (kind, name) = cleaned.split_once('/')?;

                if kind == tpe {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn get_releases() -> Result<Vec<String>, GitError> {
        let branches = Self::get_branches().await?;
        Ok(Self::filter_local_branches(
            &branches,
            &"release".to_string(),
        ))
    }

    pub async fn get_features() -> Result<Vec<String>, GitError> {
        let branches = Self::get_branches().await?;
        Ok(Self::filter_local_branches(
            &branches,
            &"feature".to_string(),
        ))
    }

    pub async fn get_bugfixes() -> Result<Vec<String>, GitError> {
        let branches = Self::get_branches().await?;
        Ok(Self::filter_local_branches(
            &branches,
            &"bugfix".to_string(),
        ))
    }

    pub async fn get_hotfixes() -> Result<Vec<String>, GitError> {
        let branches = Self::get_branches().await?;
        Ok(Self::filter_local_branches(
            &branches,
            &"hotfix".to_string(),
        ))
    }

    fn filter_remote_branches(branches: &[String], tpe: &String) -> Vec<String> {
        branches
            .iter()
            .filter_map(|s| {
                let cleaned = s.trim().trim_start_matches('*').trim();

                let mut parts = cleaned.splitn(2, '/');

                let kind = parts.next()?;
                let name = parts.next()?;

                if kind == tpe {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    pub async fn get_remote_releases() -> Result<Vec<String>, GitError> {
        let branches = Self::get_remote_branches().await?;
        Ok(Self::filter_remote_branches(
            &branches,
            &"release".to_string(),
        ))
    }

    pub async fn get_remote_features() -> Result<Vec<String>, GitError> {
        let branches = Self::get_remote_branches().await?;
        Ok(Self::filter_remote_branches(
            &branches,
            &"feature".to_string(),
        ))
    }

    pub async fn get_remote_bugfixes() -> Result<Vec<String>, GitError> {
        let branches = Self::get_remote_branches().await?;
        Ok(Self::filter_remote_branches(
            &branches,
            &"bugfix".to_string(),
        ))
    }

    pub async fn get_remote_hotfixes() -> Result<Vec<String>, GitError> {
        let branches = Self::get_remote_branches().await?;
        Ok(Self::filter_remote_branches(
            &branches,
            &"hotfix".to_string(),
        ))
    }
}
