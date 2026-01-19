#!/usr/bin/env node
const { execSync } = require("child_process");
const fs = require("fs");

globalThis.app = globalThis.app || {};
globalThis.app.sapphillon = globalThis.app.sapphillon || {};
globalThis.app.sapphillon.core = globalThis.app.sapphillon.core || {};

// exec wrapper using node child_process.execSync
globalThis.app.sapphillon.core.exec = globalThis.app.sapphillon.core.exec || {};
globalThis.app.sapphillon.core.exec.exec = function (cmd) {
  try {
    return execSync(cmd, { encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] });
  } catch (e) {
    if (e.stdout) return e.stdout.toString();
    // return stderr or empty string to mimic exec behavior
    return (e.stderr && e.stderr.toString()) || "";
  }
};

// filesystem polyfill
globalThis.app.sapphillon.core.filesystem =
  globalThis.app.sapphillon.core.filesystem || {};
globalThis.app.sapphillon.core.filesystem.read = function (path) {
  try {
    return fs.readFileSync(path, "utf8");
  } catch (e) {
    return "";
  }
};
globalThis.app.sapphillon.core.filesystem.write = function (path, content) {
  try {
    fs.writeFileSync(path, content, "utf8");
    return "ok";
  } catch (e) {
    throw e;
  }
};

// minimal search fallback
globalThis.app.sapphillon.core.search =
  globalThis.app.sapphillon.core.search || {};
globalThis.app.sapphillon.core.search.file = function (rootPath, query) {
  return JSON.stringify([]);
};

// Load Finder plugin and workflow script
require("../plugins/finder/src/00_finder.js");
require("../demo_workflows/workflow.js");

(async () => {
  try {
    if (typeof workflow !== "function") {
      console.error("workflow() not found");
      process.exit(2);
    }
    console.log("Invoking workflow()...");
    await workflow();
    console.log("workflow() completed.");

    const outPath = "/Users/user/Desktop/finder_test_results.json";
    if (fs.existsSync(outPath)) {
      console.log("Found output file:", outPath);
      const content = fs.readFileSync(outPath, "utf8");
      console.log("---- OUTPUT JSON START ----");
      console.log(content);
      console.log("---- OUTPUT JSON END ----");
    } else {
      console.log("Output file not found:", outPath);
    }
  } catch (e) {
    console.error("Error running workflow:", e);
    process.exit(1);
  }
})();
