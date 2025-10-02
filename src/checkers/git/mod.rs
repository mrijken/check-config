use std::path::PathBuf;

use git2::{BranchType, Error, FetchOptions, Oid, RemoteCallbacks, Repository, ResetType};

use crate::{
    checkers::{
        base::CheckResult,
        file::{get_option_string_value_from_checktable, get_string_value_from_checktable},
    },
    uri::parse_uri,
};

use super::{
    GenericChecker,
    base::{CheckConstructor, CheckDefinitionError, CheckError, Checker},
};

#[derive(Debug)]
pub(crate) struct GitFetched {
    generic_check: GenericChecker,
    destination_dir: PathBuf,
    repo: String,
    branch: Option<String>,
    tag: Option<String>,
    commit_hash: Option<String>,
}

fn exactly_one(a: bool, b: bool, c: bool) -> bool {
    (a as u8 + b as u8 + c as u8) == 1
}

//[[file_checkout]]
// dir = "destination"
// repo = "repo_url"
// ref = "branch, commit hash or tag"
impl CheckConstructor for GitFetched {
    type Output = Self;

    fn from_check_table(
        generic_check: GenericChecker,
        check_table: toml_edit::Table,
    ) -> Result<Self::Output, CheckDefinitionError> {
        let repo = get_string_value_from_checktable(&check_table, "repo")?;
        let branch = get_option_string_value_from_checktable(&check_table, "branch")?;
        let tag = get_option_string_value_from_checktable(&check_table, "tag")?;
        let commit_hash = get_option_string_value_from_checktable(&check_table, "commit_hash")?;

        if !exactly_one(branch.is_some(), tag.is_some(), commit_hash.is_some()) {
            return Err(CheckDefinitionError::InvalidDefinition(
                "one of branch, tag or commit_hash needs to be given".into(),
            ));
        }

        let dir = get_string_value_from_checktable(&check_table, "dir")?;
        let dir = parse_uri(dir.as_str(), Some(generic_check.file_with_checks()))
            .map_err(|e| CheckDefinitionError::InvalidDefinition(e.to_string()))?
            .to_file_path()
            .map_err(|_| CheckDefinitionError::InvalidDefinition("invalid path".into()))?;
        Ok(Self {
            repo,
            branch,
            commit_hash,
            tag,
            destination_dir: dir,
            generic_check,
        })
    }
}
impl Checker for GitFetched {
    fn checker_type(&self) -> String {
        "git_fetched".to_string()
    }

    fn generic_checker(&self) -> &GenericChecker {
        &self.generic_check
    }
    fn checker_object(&self) -> String {
        self.repo.clone()
    }
    fn check_(&self, fix: bool) -> Result<crate::checkers::base::CheckResult, CheckError> {
        let mut action_messages: Vec<String> = vec![];

        // git clone when self.dir does not exists
        let git_clone = !self.destination_dir.exists();

        if git_clone {
            action_messages.push("git clone".into());
        }

        // error when dir is not a git dir
        let not_a_git_dir = !git_clone && !self.destination_dir.join(".git").is_dir();

        if not_a_git_dir {
            action_messages.push("delete dir, because it is not a git dir".into());
        }

        // fetch if branch is not pulled or tag/commit is not present
        let sync_repo = if git_clone || not_a_git_dir {
            false
        } else {
            !is_in_sync(
                &self.destination_dir,
                self.branch.as_deref(),
                self.commit_hash.as_deref(),
                self.tag.as_deref(),
            )
            .map_err(|e| CheckError::GitError(e.to_string()))?
        };

        if sync_repo {
            action_messages.push("git checkout needed".into());
        }

        let action_message = action_messages.join("\n");

        let fix_needed = git_clone || sync_repo;

        let check_result = match (fix, fix_needed) {
            (true, true) => {
                if git_clone {
                    git2::Repository::clone(self.repo.as_str(), self.destination_dir.clone())
                        .map_err(|e| CheckError::GitError(e.to_string()))?;
                }

                if sync_repo || git_clone {
                    sync_with_remote(
                        &self.destination_dir,
                        self.branch.as_deref(),
                        self.commit_hash.as_deref(),
                        self.tag.as_deref(),
                    )
                    .map_err(|e| CheckError::GitError(e.to_string()))?;
                }

                CheckResult::FixExecuted(action_message)
            }
            (true, false) => CheckResult::NoFixNeeded,
            (false, false) => CheckResult::NoFixNeeded,
            (false, true) => CheckResult::FixNeeded(action_message),
        };

        Ok(check_result)
    }
}
/// Check whether a repo (branch, commit, or tag) is up-to-date with its remote
pub fn is_in_sync(
    git_dir: &PathBuf,
    branch: Option<&str>,
    commit_hash: Option<&str>,
    tag: Option<&str>,
) -> Result<bool, Error> {
    let repo = Repository::open(git_dir)?;

    // Default to origin remote
    let mut remote = repo.find_remote("origin")?;

    // Fetch updates from remote (to ensure refs are fresh)
    let callbacks = RemoteCallbacks::new();
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    remote.fetch(
        &[
            "refs/heads/*:refs/remotes/origin/*",
            "refs/tags/*:refs/tags/*",
        ],
        Some(&mut fetch_opts),
        None,
    )?;

    // Compare by branch
    if let Some(branch_name) = branch {
        let local_branch = repo.find_branch(branch_name, BranchType::Local)?;
        let local_commit = local_branch.get().peel_to_commit()?.id();

        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        let remote_commit = repo.find_reference(&remote_ref)?.peel_to_commit()?.id();

        return Ok(local_commit == remote_commit);
    }

    // Compare by commit hash
    if let Some(hash) = commit_hash {
        let oid = Oid::from_str(hash)?;
        let local_commit = repo.find_commit(oid)?;

        // Make sure this commit exists in remote refs
        for remote_ref in repo.references_glob("refs/remotes/origin/*")? {
            let commit = remote_ref?.peel_to_commit()?;
            if commit.id() == local_commit.id() {
                return Ok(true);
            }
        }
        return Ok(false);
    }

    // Compare by tag
    if let Some(tag_name) = tag {
        let local_ref = repo.find_reference(&format!("refs/tags/{}", tag_name))?;
        let local_commit = local_ref.peel_to_commit()?.id();

        let remote_ref = repo.find_reference(&format!("refs/tags/{}", tag_name))?;
        let remote_commit = remote_ref.peel_to_commit()?.id();

        return Ok(local_commit == remote_commit);
    }

    Err(Error::from_str(
        "Must provide a branch, commit_hash, or tag",
    ))
}

// Sync repo to remote (branch, commit, or tag)
pub fn sync_with_remote(
    git_dir: &PathBuf,
    branch: Option<&str>,
    commit_hash: Option<&str>,
    tag: Option<&str>,
) -> Result<(), Error> {
    let repo = Repository::open(git_dir)?;

    // Always fetch first
    let mut remote = repo.find_remote("origin")?;
    let callbacks = RemoteCallbacks::new();
    let mut fetch_opts = FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    remote.fetch(
        &[
            "refs/heads/*:refs/remotes/origin/*",
            "refs/tags/*:refs/tags/*",
        ],
        Some(&mut fetch_opts),
        None,
    )?;

    // If branch is specified: reset to remote branch
    if let Some(branch_name) = branch {
        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        let remote_commit = repo.find_reference(&remote_ref)?.peel_to_commit()?;

        // Force reset local HEAD to remote commit
        repo.reset(remote_commit.as_object(), ResetType::Hard, None)?;
        repo.set_head(&format!("refs/heads/{}", branch_name))?;

        return Ok(());
    }

    // If commit hash is specified: reset to that commit (only if found remotely)
    if let Some(hash) = commit_hash {
        let oid = Oid::from_str(hash)?;
        let commit = repo.find_commit(oid)?;

        // Ensure commit exists in remote refs
        let mut found = false;
        for remote_ref in repo.references_glob("refs/remotes/origin/*")? {
            if remote_ref?.peel_to_commit()?.id() == commit.id() {
                found = true;
                break;
            }
        }

        if !found {
            return Err(Error::from_str("Commit not found in remote"));
        }

        repo.reset(commit.as_object(), ResetType::Hard, None)?;
        repo.set_head_detached(commit.id())?;

        return Ok(());
    }

    // If tag is specified: reset to that tag’s commit
    if let Some(tag_name) = tag {
        let tag_ref = format!("refs/tags/{}", tag_name);
        let commit = repo.find_reference(&tag_ref)?.peel_to_commit()?;

        repo.reset(commit.as_object(), ResetType::Hard, None)?;
        repo.set_head_detached(commit.id())?;

        return Ok(());
    }

    Err(Error::from_str("Must provide branch, commit_hash, or tag"))
}

#[cfg(test)]
mod tests {

    use crate::checkers::{base::CheckResult, test_helpers};

    use super::*;

    use tempfile::tempdir;

    fn get_check_with_result(
        repo: String,
        branch: String,
    ) -> (Result<GitFetched, CheckDefinitionError>, tempfile::TempDir) {
        let generic_check = test_helpers::get_generic_check();

        let mut check_table = toml_edit::Table::new();
        let tmp_dir = tempdir().unwrap();
        let dir = tmp_dir.path().join("file_to_check");
        check_table.insert("dir", dir.to_string_lossy().to_string().into());

        check_table.insert("repo", repo.into());
        check_table.insert("branch", branch.into());
        (
            GitFetched::from_check_table(generic_check, check_table),
            tmp_dir,
        )
    }

    #[test]
    fn test_git_fetched() {
        let (git_fetched_check, _tempdir) = get_check_with_result(
            "https://github.com/mrijken/check-config.git".into(),
            "main".into(),
        );

        let git_fetch_check = git_fetched_check.expect("correct checktable");

        assert_eq!(
            git_fetch_check.check_(false).unwrap(),
            CheckResult::FixNeeded("git clone".into())
        );

        assert_eq!(
            git_fetch_check.check_(true).unwrap(),
            CheckResult::FixExecuted("git clone".into())
        );
        assert_eq!(
            git_fetch_check.check_(false).unwrap(),
            CheckResult::NoFixNeeded
        );
    }
}
