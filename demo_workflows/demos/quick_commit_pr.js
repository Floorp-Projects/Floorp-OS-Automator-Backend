/**
 * Workflow: Quick Commit & PR
 *
 * Automates the process of:
 * 1. Getting current workspace path and branch
 * 2. Getting git diff
 * 3. Generating commit message with AI
 * 4. Committing and Pushing changes
 * 5. Opening/Creating a PR tab in Floorp
 * 6. Filling PR title/body with AI
 * 7. Clicking Create PR
 */

async function workflow() {
  console.log("=== Quick Commit & PR Workflow ===");

  try {
    // Step 1: Get current workspace from VSCode
    console.log("Step 1: Getting workspace path...");
    let repoPath = "";
    try {
      repoPath = vscode.get_workspace_path();
      console.log("DEBUG: Raw workspace path returned: [" + repoPath + "]");
      console.log("DEBUG: Path length: " + repoPath.length);
    } catch (e) {
      console.log("Failed to get workspace path: " + e);
      throw new Error("Could not determine repository path from VSCode");
    }

    let currentBranch = "main";
    try {
      const branchJson = git.getBranch(repoPath);
      console.log("DEBUG: Raw branchJson: " + branchJson);
      const branchInfo = JSON.parse(branchJson);
      currentBranch = branchInfo.branch;
      console.log("DEBUG: Parsed branch: [" + currentBranch + "]");
    } catch (gitError) {
      console.log("Failed to get branch: " + gitError);
    }

    console.log(
      "Final values - repoPath: [" +
        repoPath +
        "], branch: [" +
        currentBranch +
        "]"
    );

    // Step 2: Get git diff for AI analysis
    console.log("Step 2: Getting git diff...");
    let diffContent = "";
    try {
      const diffJson = git.getDiff(repoPath);
      const diffInfo = JSON.parse(diffJson);
      diffContent = diffInfo.combined || "";
      console.log("Diff length: " + diffContent.length + " chars");
    } catch (e) {
      console.log("Failed to get diff: " + e);
    }

    // Step 3: Generate commit message using AI
    console.log("Step 3: Generating commit message with AI...");
    let commitMessage = "chore: automated commit";
    if (diffContent.length > 0) {
      try {
        commitMessage = iniad.generateCommitMessage(diffContent);
        console.log("AI generated commit message: " + commitMessage);
      } catch (e) {
        console.log("AI commit message failed: " + e + ", using default.");
      }
    }

    // Step 4: Stage, commit, and push
    console.log("Step 4: Git add, commit, push...");
    try {
      const addResult = git.add(repoPath);
      console.log("Git add: " + addResult);
    } catch (e) {
      console.log("Git add failed: " + e);
    }

    try {
      const commitResult = git.commit(repoPath, commitMessage);
      console.log("Git commit: " + commitResult);
    } catch (e) {
      console.log("Git commit failed (may be nothing to commit): " + e);
    }

    try {
      const pushResult = git.push(repoPath);
      console.log("Git push: " + pushResult);
    } catch (e) {
      console.log("Git push failed: " + e);
    }

    // Step 5: Prepare Floorp tab for PR creation
    console.log("Step 5: Preparing Floorp tab for PR...");
    // Use ?expand=1 to automatically expand the PR form without clicking "Create pull request" button
    const targetUrl =
      "https://github.com/Floorp-Projects/Floorp/compare/main..." +
      currentBranch +
      "?expand=1";
    console.log("Target URL: " + targetUrl);
    const tabsJson = floorp.listBrowserTabs();
    const tabs = JSON.parse(tabsJson);
    let targetTab = null;

    // Look for existing compare tab (with or without expand param)
    for (var i = 0; i < tabs.length; i++) {
      if (
        tabs[i].url &&
        tabs[i].url.includes("compare/main..." + currentBranch)
      ) {
        targetTab = tabs[i];
        break;
      }
    }

    let instanceId = "";
    if (targetTab) {
      console.log("Found existing target tab: " + targetTab.title);
      const attachResultJson = floorp.attachToTab(targetTab.browserId);
      const attachResult = JSON.parse(attachResultJson);
      instanceId = attachResult.instanceId;

      // Navigate to ?expand=1 URL to ensure form is expanded
      console.log("Navigating existing tab to expanded URL...");
      try {
        floorp.tabNavigate(instanceId, targetUrl);
        console.log("Navigation initiated to: " + targetUrl);
      } catch (navError) {
        console.log("Navigation warning: " + navError);
      }
    } else {
      console.log("Creating new tab...");
      try {
        const createResultJson = floorp.createTab(targetUrl, false);
        const createResult = JSON.parse(createResultJson);
        instanceId = createResult.instanceId;
        console.log("Created new tab, instanceId: " + instanceId);
      } catch (createError) {
        console.log("Failed to create tab: " + createError);
        throw createError;
      }
    }

    // Step 6: Wait for PR form (with extended waiting)
    console.log("Step 6: Waiting for PR form...");
    try {
      floorp.tabWaitForElement(instanceId, "#pull_request_title", "20000");
      console.log("PR form found!");
    } catch (e) {
      console.log("Wait for element warning (first attempt): " + e);
      // Try waiting again
      try {
        floorp.tabWaitForElement(instanceId, "#pull_request_title", "10000");
        console.log("PR form found on second attempt!");
      } catch (e2) {
        console.log("Wait for element warning (second attempt): " + e2);
      }
    }

    // Step 7: Generate PR title/body with AI
    console.log("Step 7: Generating PR title/body using AI...");
    let aiResult;
    try {
      const aiJson = iniad.generatePrDescription(diffContent);
      console.log("DEBUG: AI response JSON: " + aiJson);
      aiResult = JSON.parse(aiJson);
      console.log("DEBUG: Parsed aiResult.title: " + aiResult.title);
      console.log(
        "DEBUG: Parsed aiResult.body length: " +
          (aiResult.body ? aiResult.body.length : 0)
      );
    } catch (aiError) {
      console.log("AI generation failed: " + aiError);
      aiResult = {
        title: commitMessage,
        body:
          "Automated PR" +
          String.fromCharCode(10, 10) +
          diffContent.substring(0, 1000),
      };
      console.log("DEBUG: Using fallback title: " + aiResult.title);
    }

    // Step 8: Fill PR form
    console.log("Step 8: Filling PR form...");
    console.log("DEBUG: About to fill title with: [" + aiResult.title + "]");

    let titleFillSuccess = false;
    let bodyFillSuccess = false;

    try {
      const titleResultJson = floorp.tabFillForm(
        instanceId,
        "#pull_request_title",
        aiResult.title
      );
      console.log("Title fill result: " + titleResultJson);
      const titleResult = JSON.parse(titleResultJson);
      titleFillSuccess = titleResult.ok === true;
    } catch (e) {
      console.log("Fill title error: " + e);
    }

    console.log("DEBUG: About to fill body...");
    try {
      const bodyResultJson = floorp.tabFillForm(
        instanceId,
        "#pull_request_body",
        aiResult.body
      );
      console.log("Body fill result: " + bodyResultJson);
      const bodyResult = JSON.parse(bodyResultJson);
      bodyFillSuccess = bodyResult.ok === true;
    } catch (e) {
      console.log("Fill body error: " + e);
    }

    // Abort if form fill failed
    if (!titleFillSuccess || !bodyFillSuccess) {
      console.log(
        "ERROR: Form fill failed! Title: " +
          titleFillSuccess +
          ", Body: " +
          bodyFillSuccess
      );
      console.log("Aborting PR creation to prevent incomplete PR.");
      return {
        ok: false,
        message:
          "Form fill failed. Title filled: " +
          titleFillSuccess +
          ", Body filled: " +
          bodyFillSuccess,
        error:
          "フォーム入力に失敗しました。ページが正しく読み込まれていない可能性があります。",
      };
    }

    console.log("Form fill successful! Proceeding to create PR...");

    // Step 9: Click Create PR button
    console.log("Step 9: Clicking Create PR button...");
    try {
      floorp.tabClick(instanceId, ".hx_create-pr-button");
      console.log("Clicked creation button.");
    } catch (e) {
      console.log("Failed to click button: " + e);
      try {
        floorp.tabClick(instanceId, "button.btn-primary");
        console.log("Clicked fallback button.");
      } catch (e2) {
        console.log("Failed to click fallback: " + e2);
      }
    }

    return {
      ok: true,
      message: "Full automation complete: commit, push, PR created",
      commitMessage: commitMessage,
      prTitle: aiResult.title,
    };
  } catch (error) {
    return {
      ok: false,
      message: "Workflow failed: " + error,
    };
  }
}
workflow();
