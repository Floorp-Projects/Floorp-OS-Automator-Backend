/**
 * Demo Workflow: Verify VSCode Plugin Functions
 *
 * This workflow verifies all functions of the VSCode plugin:
 * 1. vscode.write_file
 * 2. vscode.open_file
 * 3. vscode.get_active_file_content
 * 4. vscode.open_folder
 * 5. vscode.close_workspace
 */

async function workflow() {
  console.log("Starting VSCode Plugin Verification...");

  const tmpDir = "/tmp/floorp_vscode_test_" + Date.now();
  const filePath = tmpDir + "/test_file.txt";
  const testContent =
    "Hello from Floorp OS Automator! Timestamp: " + Date.now();

  try {
    // 1. Create directory (using Deno/standard fs not available? default_api unavailable?)
    // Standard Deno API is available in workflow runtime
    try {
      Deno.mkdirSync(tmpDir, { recursive: true });
      console.log("Created temp directory: " + tmpDir);
    } catch (e) {
      console.error(
        "Failed to create temp directory (might already exist or permission issue): " +
          e
      );
    }

    // 2. Test write_file
    console.log("Testing vscode.write_file...");
    const writeResult = vscode.write_file(filePath, testContent);
    console.log("write_file result: " + writeResult);

    // Wait a bit for VSCode to open and render
    console.log("Waiting for VSCode to open...");
    await new Promise((r) => setTimeout(r, 2000));

    // 3. Test open_file (Redundant if write_file opens it, but testing explicit call)
    console.log("Testing vscode.open_file...");
    const openResult = vscode.open_file(filePath);
    console.log("open_file result: " + openResult);
    await new Promise((r) => setTimeout(r, 1000));

    // 4. Test get_active_file_content
    console.log("Testing vscode.get_active_file_content...");
    // Ensure VSCode is focused for AppleScript to work
    const content = vscode.get_active_file_content();
    console.log("get_active_file_content result length: " + content.length);

    if (content.trim() === testContent.trim()) {
      console.log("SUCCESS: Content matches!");
    } else {
      console.error("FAILURE: Content mismatch!");
      console.error("Expected: " + testContent);
      console.error("Actual: " + content);
    }

    // 5. Test open_folder
    console.log("Testing vscode.open_folder...");
    const folderResult = vscode.open_folder(tmpDir);
    console.log("open_folder result: " + folderResult);
    await new Promise((r) => setTimeout(r, 2000));

    // 6. Test close_workspace
    console.log("Testing vscode.close_workspace...");
    const closeResult = vscode.close_workspace();
    console.log("close_workspace result: " + closeResult);

    console.log("Verification checks completed.");
  } catch (e) {
    console.error("Verification failed with error: " + e);
  }
}

workflow();
