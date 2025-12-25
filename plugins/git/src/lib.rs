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

pub fn git_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.git".to_string(),
        package_name: "Git".to_string(),
        description: "Git repository operations for workflow automation.".to_string(),
        functions: vec![
            git_get_diff_plugin_function(),
            git_get_status_plugin_function(),
            git_get_branch_plugin_function(),
            git_get_commit_log_plugin_function(),
            git_add_plugin_function(),
            git_commit_plugin_function(),
            git_push_plugin_function(),
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

pub fn core_git_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.git".to_string(),
        "Git".to_string(),
        vec![
            core_git_get_diff_plugin(),
            core_git_get_status_plugin(),
            core_git_get_branch_plugin(),
            core_git_get_commit_log_plugin(),
            core_git_add_plugin(),
            core_git_commit_plugin(),
            core_git_push_plugin(),
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

    // Get staged and unstaged diff
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

    // Default to adding all files if none specified
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
