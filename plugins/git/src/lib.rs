// Git Plugin for Sapphillon
// SPDX-FileCopyrightText: 2025 Floorp Projects
// SPDX-License-Identifier: MPL-2.0 OR GPL-3.0-or-later

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;
use sapphillon_core::permission::{
    CheckPermissionResult, PluginFunctionPermissions, check_permission,
};
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{
    Permission, PermissionLevel, PermissionType, PluginFunction, PluginPackage,
};
use sapphillon_core::runtime::OpStateWorkflowData;
use std::process::Command;
use std::sync::{Arc, Mutex};

// ============================================================================
// Plugin Function Definitions
// ============================================================================

// --- Basic Operations ---

pub fn git_get_diff_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getDiff".to_string(),
        function_name: "Get Diff".to_string(),
        description: "Get git diff output for a repository".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_get_status_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getStatus".to_string(),
        function_name: "Get Status".to_string(),
        description: "Get git status output for a repository".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_get_branch_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getBranch".to_string(),
        function_name: "Get Branch".to_string(),
        description: "Get current branch name for a repository".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_get_commit_log_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getCommitLog".to_string(),
        function_name: "Get Commit Log".to_string(),
        description: "Get recent commit log for a repository".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, count?: number".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_add_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.add".to_string(),
        function_name: "Add".to_string(),
        description: "Stage files for commit".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, files?: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_commit_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.commit".to_string(),
        function_name: "Commit".to_string(),
        description: "Commit staged changes".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, message: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_push_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.push".to_string(),
        function_name: "Push".to_string(),
        description: "Push commits to remote".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

// --- Branch Operations ---

pub fn git_checkout_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.checkout".to_string(),
        function_name: "Checkout".to_string(),
        description: "Switch to a different branch".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, branch: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_create_branch_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.createBranch".to_string(),
        function_name: "Create Branch".to_string(),
        description: "Create a new branch".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, branchName: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_delete_branch_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.deleteBranch".to_string(),
        function_name: "Delete Branch".to_string(),
        description: "Delete a branch".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, branchName: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_list_branches_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.listBranches".to_string(),
        function_name: "List Branches".to_string(),
        description: "List all branches".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_merge_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.merge".to_string(),
        function_name: "Merge".to_string(),
        description: "Merge a branch into current branch".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, branch: string".to_string(),
        returns: "string".to_string(),
    }
}

// --- Remote Operations ---

pub fn git_pull_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.pull".to_string(),
        function_name: "Pull".to_string(),
        description: "Pull changes from remote".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_fetch_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.fetch".to_string(),
        function_name: "Fetch".to_string(),
        description: "Fetch changes from remote without merging".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_get_remotes_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getRemotes".to_string(),
        function_name: "Get Remotes".to_string(),
        description: "List all remote repositories".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_set_remote_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.setRemote".to_string(),
        function_name: "Set Remote".to_string(),
        description: "Add or update a remote repository".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, name: string, url: string".to_string(),
        returns: "string".to_string(),
    }
}

// --- Information Retrieval ---

pub fn git_get_last_commit_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getLastCommit".to_string(),
        function_name: "Get Last Commit".to_string(),
        description: "Get details of the last commit".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_get_file_history_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getFileHistory".to_string(),
        function_name: "Get File History".to_string(),
        description: "Get commit history for a specific file".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, filePath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_blame_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.blame".to_string(),
        function_name: "Blame".to_string(),
        description: "Show who last modified each line of a file".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, filePath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_show_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.show".to_string(),
        function_name: "Show".to_string(),
        description: "Show details of a specific commit".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, commitHash: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_get_tags_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.getTags".to_string(),
        function_name: "Get Tags".to_string(),
        description: "List all tags".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

// --- Workflow Utilities ---

pub fn git_stash_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.stash".to_string(),
        function_name: "Stash".to_string(),
        description: "Stash current changes".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_stash_pop_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.stashPop".to_string(),
        function_name: "Stash Pop".to_string(),
        description: "Apply and remove the latest stash".to_string(),
        permissions: vec![],
        arguments: "repoPath: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_reset_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.reset".to_string(),
        function_name: "Reset".to_string(),
        description: "Reset current HEAD to a specific state".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, mode: string, ref?: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_revert_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.revert".to_string(),
        function_name: "Revert".to_string(),
        description: "Revert a specific commit".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, commitHash: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_cherry_pick_plugin_function() -> PluginFunction {
    PluginFunction {
        function_id: "app.sapphillon.core.git.cherryPick".to_string(),
        function_name: "Cherry Pick".to_string(),
        description: "Apply a specific commit to current branch".to_string(),
        permissions: vec![],
        arguments: "repoPath: string, commitHash: string".to_string(),
        returns: "string".to_string(),
    }
}

pub fn git_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.git".to_string(),
        package_name: "Git".to_string(),
        description: "Git repository operations for workflow automation.".to_string(),
        functions: vec![
            // Basic operations
            git_get_diff_plugin_function(),
            git_get_status_plugin_function(),
            git_get_branch_plugin_function(),
            git_get_commit_log_plugin_function(),
            git_add_plugin_function(),
            git_commit_plugin_function(),
            git_push_plugin_function(),
            // Branch operations
            git_checkout_plugin_function(),
            git_create_branch_plugin_function(),
            git_delete_branch_plugin_function(),
            git_list_branches_plugin_function(),
            git_merge_plugin_function(),
            // Remote operations
            git_pull_plugin_function(),
            git_fetch_plugin_function(),
            git_get_remotes_plugin_function(),
            git_set_remote_plugin_function(),
            // Information retrieval
            git_get_last_commit_plugin_function(),
            git_get_file_history_plugin_function(),
            git_blame_plugin_function(),
            git_show_plugin_function(),
            git_get_tags_plugin_function(),
            // Workflow utilities
            git_stash_plugin_function(),
            git_stash_pop_plugin_function(),
            git_reset_plugin_function(),
            git_revert_plugin_function(),
            git_cherry_pick_plugin_function(),
        ],
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

// ============================================================================
// Core Plugin Functions (for Deno runtime integration)
// ============================================================================

// --- Basic Operations ---

pub fn core_git_get_diff_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getDiff".to_string(),
        "Get Diff".to_string(),
        "Get git diff output for a repository".to_string(),
        op_git_get_diff(),
        Some(include_str!("00_git.js").to_string()),
    )
}

pub fn core_git_get_status_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getStatus".to_string(),
        "Get Status".to_string(),
        "Get git status output for a repository".to_string(),
        op_git_get_status(),
        None,
    )
}

pub fn core_git_get_branch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getBranch".to_string(),
        "Get Branch".to_string(),
        "Get current branch name for a repository".to_string(),
        op_git_get_branch(),
        None,
    )
}

pub fn core_git_get_commit_log_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getCommitLog".to_string(),
        "Get Commit Log".to_string(),
        "Get recent commit log for a repository".to_string(),
        op_git_get_commit_log(),
        None,
    )
}

pub fn core_git_add_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.add".to_string(),
        "Add".to_string(),
        "Stage files for commit".to_string(),
        op_git_add(),
        None,
    )
}

pub fn core_git_commit_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.commit".to_string(),
        "Commit".to_string(),
        "Commit staged changes".to_string(),
        op_git_commit(),
        None,
    )
}

pub fn core_git_push_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.push".to_string(),
        "Push".to_string(),
        "Push commits to remote".to_string(),
        op_git_push(),
        None,
    )
}

// --- Branch Operations ---

pub fn core_git_checkout_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.checkout".to_string(),
        "Checkout".to_string(),
        "Switch to a different branch".to_string(),
        op_git_checkout(),
        None,
    )
}

pub fn core_git_create_branch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.createBranch".to_string(),
        "Create Branch".to_string(),
        "Create a new branch".to_string(),
        op_git_create_branch(),
        None,
    )
}

pub fn core_git_delete_branch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.deleteBranch".to_string(),
        "Delete Branch".to_string(),
        "Delete a branch".to_string(),
        op_git_delete_branch(),
        None,
    )
}

pub fn core_git_list_branches_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.listBranches".to_string(),
        "List Branches".to_string(),
        "List all branches".to_string(),
        op_git_list_branches(),
        None,
    )
}

pub fn core_git_merge_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.merge".to_string(),
        "Merge".to_string(),
        "Merge a branch into current branch".to_string(),
        op_git_merge(),
        None,
    )
}

// --- Remote Operations ---

pub fn core_git_pull_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.pull".to_string(),
        "Pull".to_string(),
        "Pull changes from remote".to_string(),
        op_git_pull(),
        None,
    )
}

pub fn core_git_fetch_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.fetch".to_string(),
        "Fetch".to_string(),
        "Fetch changes from remote without merging".to_string(),
        op_git_fetch(),
        None,
    )
}

pub fn core_git_get_remotes_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getRemotes".to_string(),
        "Get Remotes".to_string(),
        "List all remote repositories".to_string(),
        op_git_get_remotes(),
        None,
    )
}

pub fn core_git_set_remote_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.setRemote".to_string(),
        "Set Remote".to_string(),
        "Add or update a remote repository".to_string(),
        op_git_set_remote(),
        None,
    )
}

// --- Information Retrieval ---

pub fn core_git_get_last_commit_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getLastCommit".to_string(),
        "Get Last Commit".to_string(),
        "Get details of the last commit".to_string(),
        op_git_get_last_commit(),
        None,
    )
}

pub fn core_git_get_file_history_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getFileHistory".to_string(),
        "Get File History".to_string(),
        "Get commit history for a specific file".to_string(),
        op_git_get_file_history(),
        None,
    )
}

pub fn core_git_blame_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.blame".to_string(),
        "Blame".to_string(),
        "Show who last modified each line of a file".to_string(),
        op_git_blame(),
        None,
    )
}

pub fn core_git_show_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.show".to_string(),
        "Show".to_string(),
        "Show details of a specific commit".to_string(),
        op_git_show(),
        None,
    )
}

pub fn core_git_get_tags_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.getTags".to_string(),
        "Get Tags".to_string(),
        "List all tags".to_string(),
        op_git_get_tags(),
        None,
    )
}

// --- Workflow Utilities ---

pub fn core_git_stash_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.stash".to_string(),
        "Stash".to_string(),
        "Stash current changes".to_string(),
        op_git_stash(),
        None,
    )
}

pub fn core_git_stash_pop_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.stashPop".to_string(),
        "Stash Pop".to_string(),
        "Apply and remove the latest stash".to_string(),
        op_git_stash_pop(),
        None,
    )
}

pub fn core_git_reset_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.reset".to_string(),
        "Reset".to_string(),
        "Reset current HEAD to a specific state".to_string(),
        op_git_reset(),
        None,
    )
}

pub fn core_git_revert_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.revert".to_string(),
        "Revert".to_string(),
        "Revert a specific commit".to_string(),
        op_git_revert(),
        None,
    )
}

pub fn core_git_cherry_pick_plugin() -> CorePluginFunction {
    CorePluginFunction::new(
        "app.sapphillon.core.git.cherryPick".to_string(),
        "Cherry Pick".to_string(),
        "Apply a specific commit to current branch".to_string(),
        op_git_cherry_pick(),
        None,
    )
}

pub fn core_git_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.git".to_string(),
        "Git".to_string(),
        vec![
            // Basic operations
            core_git_get_diff_plugin(),
            core_git_get_status_plugin(),
            core_git_get_branch_plugin(),
            core_git_get_commit_log_plugin(),
            core_git_add_plugin(),
            core_git_commit_plugin(),
            core_git_push_plugin(),
            // Branch operations
            core_git_checkout_plugin(),
            core_git_create_branch_plugin(),
            core_git_delete_branch_plugin(),
            core_git_list_branches_plugin(),
            core_git_merge_plugin(),
            // Remote operations
            core_git_pull_plugin(),
            core_git_fetch_plugin(),
            core_git_get_remotes_plugin(),
            core_git_set_remote_plugin(),
            // Information retrieval
            core_git_get_last_commit_plugin(),
            core_git_get_file_history_plugin(),
            core_git_blame_plugin(),
            core_git_show_plugin(),
            core_git_get_tags_plugin(),
            // Workflow utilities
            core_git_stash_plugin(),
            core_git_stash_pop_plugin(),
            core_git_reset_plugin(),
            core_git_revert_plugin(),
            core_git_cherry_pick_plugin(),
        ],
    )
}

// ============================================================================
// Permission Definitions
// ============================================================================

pub fn git_plugin_permissions() -> Vec<Permission> {
    vec![Permission {
        display_name: "Exec Access".to_string(),
        description: "Allows the plugin to execute git commands.".to_string(),
        permission_type: PermissionType::Execute as i32,
        permission_level: PermissionLevel::Unspecified as i32,
        resource: vec![],
    }]
}

// ============================================================================
// Permission Check Helpers
// ============================================================================

fn _permission_check_backend(
    allow: Vec<PluginFunctionPermissions>,
    function_id: &str,
    required_perms: Vec<Permission>,
) -> Result<(), JsErrorBox> {
    let required_permissions = sapphillon_core::permission::Permissions {
        permissions: required_perms,
    };

    let allowed_permissions = {
        let permissions_vec = allow;
        permissions_vec
            .into_iter()
            .find(|p| p.plugin_function_id == function_id || p.plugin_function_id == "*")
            .map(|p| p.permissions)
            .unwrap_or_else(|| sapphillon_core::permission::Permissions {
                permissions: vec![],
            })
    };

    let permission_check_result = check_permission(&allowed_permissions, &required_permissions);

    match permission_check_result {
        CheckPermissionResult::Ok => Ok(()),
        CheckPermissionResult::MissingPermission(perm) => Err(JsErrorBox::new(
            "Error",
            format!("PermissionDenied. Missing Permissions: {}", perm),
        )),
    }
}

fn permission_check(
    state: &mut OpState,
    function_id: &str,
    required_perms: Vec<Permission>,
) -> Result<(), JsErrorBox> {
    let data = state
        .borrow::<Arc<Mutex<OpStateWorkflowData>>>()
        .lock()
        .unwrap();
    let allowed = match &data.get_allowed_permissions() {
        Some(p) => p.clone(),
        None => vec![],
    };
    _permission_check_backend(allowed, function_id, required_perms)?;
    Ok(())
}

// ============================================================================
// Git Command Execution Helpers
// ============================================================================

fn run_git_command(repo_path: &str, args: &[&str]) -> Result<String, JsErrorBox> {
    let output = Command::new("git")
        .args(["-C", repo_path])
        .args(args)
        .output()
        .map_err(|e| JsErrorBox::new("Error", format!("Failed to execute git command: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JsErrorBox::new(
            "Error",
            format!("Git command failed: {}", stderr),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

// ============================================================================
// Op2 Functions (Deno Runtime Operations)
// ============================================================================

// --- Basic Operations ---

#[op2]
#[string]
pub fn op_git_get_diff(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getDiff",
        git_plugin_permissions(),
    )?;

    let diff_staged = run_git_command(&repo_path, &["diff", "--cached"])?;
    let diff_unstaged = run_git_command(&repo_path, &["diff"])?;

    let result = serde_json::json!({
        "staged": diff_staged,
        "unstaged": diff_unstaged,
        "combined": format!("{}\n{}", diff_staged, diff_unstaged),
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_get_status(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getStatus",
        git_plugin_permissions(),
    )?;

    let status = run_git_command(&repo_path, &["status", "--porcelain"])?;

    let result = serde_json::json!({
        "status": status,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_get_branch(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getBranch",
        git_plugin_permissions(),
    )?;

    let branch = run_git_command(&repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])?;

    let result = serde_json::json!({
        "branch": branch.trim(),
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_get_commit_log(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] count: Option<String>,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getCommitLog",
        git_plugin_permissions(),
    )?;

    let count_str = count.unwrap_or_else(|| "10".to_string());
    let log = run_git_command(
        &repo_path,
        &["log", "--oneline", &format!("-{}", count_str)],
    )?;

    let result = serde_json::json!({
        "log": log,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_add(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] files: Option<String>,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.add",
        git_plugin_permissions(),
    )?;

    let files_arg = files.unwrap_or_else(|| ".".to_string());
    let output = run_git_command(&repo_path, &["add", &files_arg])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_commit(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] message: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.commit",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["commit", "-m", &message])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_push(state: &mut OpState, #[string] repo_path: String) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.push",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["push"])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Branch Operations ---

#[op2]
#[string]
pub fn op_git_checkout(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] branch: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.checkout",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["checkout", &branch])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_create_branch(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] branch_name: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.createBranch",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["checkout", "-b", &branch_name])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_delete_branch(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] branch_name: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.deleteBranch",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["branch", "-d", &branch_name])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_list_branches(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.listBranches",
        git_plugin_permissions(),
    )?;

    let local = run_git_command(&repo_path, &["branch", "--list"])?;
    let remote = run_git_command(&repo_path, &["branch", "-r"])?;

    let result = serde_json::json!({
        "local": local,
        "remote": remote,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_merge(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] branch: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.merge",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["merge", &branch])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Remote Operations ---

#[op2]
#[string]
pub fn op_git_pull(state: &mut OpState, #[string] repo_path: String) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.pull",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["pull"])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_fetch(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.fetch",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["fetch", "--all"])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_get_remotes(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getRemotes",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["remote", "-v"])?;

    let result = serde_json::json!({
        "remotes": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_set_remote(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] name: String,
    #[string] url: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.setRemote",
        git_plugin_permissions(),
    )?;

    // Try to add, if it fails, try set-url
    let result = run_git_command(&repo_path, &["remote", "add", &name, &url]);
    let output = match result {
        Ok(o) => o,
        Err(_) => run_git_command(&repo_path, &["remote", "set-url", &name, &url])?,
    };

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Information Retrieval ---

#[op2]
#[string]
pub fn op_git_get_last_commit(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getLastCommit",
        git_plugin_permissions(),
    )?;

    let hash = run_git_command(&repo_path, &["rev-parse", "HEAD"])?;
    let message = run_git_command(&repo_path, &["log", "-1", "--format=%s"])?;
    let author = run_git_command(&repo_path, &["log", "-1", "--format=%an"])?;
    let date = run_git_command(&repo_path, &["log", "-1", "--format=%ci"])?;

    let result = serde_json::json!({
        "hash": hash.trim(),
        "message": message.trim(),
        "author": author.trim(),
        "date": date.trim(),
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_get_file_history(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getFileHistory",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["log", "--oneline", "--follow", "--", &file_path])?;

    let result = serde_json::json!({
        "history": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_blame(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.blame",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["blame", &file_path])?;

    let result = serde_json::json!({
        "blame": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_show(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] commit_hash: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.show",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["show", &commit_hash])?;

    let result = serde_json::json!({
        "show": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_get_tags(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.getTags",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["tag", "-l"])?;

    let result = serde_json::json!({
        "tags": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

// --- Workflow Utilities ---

#[op2]
#[string]
pub fn op_git_stash(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.stash",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["stash"])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_stash_pop(
    state: &mut OpState,
    #[string] repo_path: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.stashPop",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["stash", "pop"])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_reset(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] mode: String,
    #[string] git_ref: Option<String>,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.reset",
        git_plugin_permissions(),
    )?;

    let mode_flag = format!("--{}", mode);
    let output = if let Some(r) = git_ref {
        run_git_command(&repo_path, &["reset", &mode_flag, &r])?
    } else {
        run_git_command(&repo_path, &["reset", &mode_flag])?
    };

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_revert(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] commit_hash: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.revert",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["revert", "--no-edit", &commit_hash])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}

#[op2]
#[string]
pub fn op_git_cherry_pick(
    state: &mut OpState,
    #[string] repo_path: String,
    #[string] commit_hash: String,
) -> Result<String, JsErrorBox> {
    permission_check(
        state,
        "app.sapphillon.core.git.cherryPick",
        git_plugin_permissions(),
    )?;

    let output = run_git_command(&repo_path, &["cherry-pick", &commit_hash])?;

    let result = serde_json::json!({
        "success": true,
        "output": output,
    });

    Ok(serde_json::to_string(&result).unwrap())
}
