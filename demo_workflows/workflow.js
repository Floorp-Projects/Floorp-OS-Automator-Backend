/**
 * Slack File Upload Workflow using Floorp Browser Automation
 *
 * このワークフローは Floorp ブラウザを使用して Slack Web アプリに
 * ファイルをアップロードする機能をテストします。
 *
 * 機能:
 * - Slack タブを検出
 * - ファイルアップロードのテスト
 *
 * DOM セレクタ（精査済み）:
 * - 添付ボタン: [data-qa="shortcuts_menu_trigger__Channel"]
 * - ファイル入力: input[data-qa="file_upload"]
 * - 送信ボタン: [data-qa="texty_send_button"]
 */

var SLACK_URL = "https://app.slack.com/client/T0A62PPRD7G/C0A68CVNZFE";
var TEST_FILE_PATH = "/Users/user/Desktop/test-upload.txt";

function log(message) {
  console.log(
    "[" + new Date().toISOString().substring(11, 19) + "] " + message
  );
}

function logDebug(label, value) {
  console.log("[DEBUG] " + label + ": " + String(value).substring(0, 200));
}

function sleep(ms) {
  var start = Date.now();
  while (Date.now() - start < ms) {}
}

function workflow() {
  log("=== Slack File Upload Test Workflow ===");
  log("");

  try {
    // Step 1: ブラウザタブから Slack を探す
    log("[Step 1] Searching for Slack tab...");
    var tabsResponse = floorp.browserTabs();
    logDebug("tabsResponse", tabsResponse);

    var tabsData = JSON.parse(tabsResponse);
    var tabs = tabsData.tabs || tabsData;
    logDebug("tabs count", tabs.length);

    var slackTab = null;
    for (var t = 0; t < tabs.length; t++) {
      var tab = tabs[t];
      var url = tab.url || "";
      if (
        url.indexOf("slack.com") !== -1 ||
        url.indexOf("app.slack.com") !== -1
      ) {
        slackTab = tab;
        log("Found Slack tab: " + tab.title);
        logDebug("Slack tab details", JSON.stringify(tab));
        break;
      }
    }

    if (!slackTab) {
      log("No Slack tab found. Opening Slack...");
      var newTabId = floorp.createTab(SLACK_URL, false);
      log("Created tab with instanceId: " + newTabId);

      log("Waiting for Slack to load...");
      floorp.tabWaitForNetworkIdle(newTabId, "15000");
      floorp.tabWaitForElement(newTabId, "[data-qa='channel_sidebar']", 10000);

      slackTab = {
        instance_id: newTabId,
        url: SLACK_URL,
        title: "Slack",
        status: "complete",
      };
    }

    // Step 2: Slack タブにアタッチ
    log("[Step 2] Attaching to Slack tab...");
    var activeTabId = String(slackTab.instance_id || slackTab.id);
    logDebug("activeTabId", activeTabId);

    var attachResult = floorp.attachToTab(activeTabId);
    log("Attached to tab: " + attachResult);

    // Step 3: DOM の確認
    log("[Step 3] Checking DOM elements...");

    // 添付ボタンの確認
    var attachBtnSelector = '[data-qa="shortcuts_menu_trigger__Channel"]';
    log("Checking attachment button: " + attachBtnSelector);
    try {
      floorp.tabWaitForElement(activeTabId, attachBtnSelector, 5000);
      log("✓ Attachment button found");
    } catch (e) {
      log("✗ Attachment button NOT found: " + e);
      // 代替セレクタを試す
      attachBtnSelector = '[aria-label="添付"]';
      log("Trying alternative selector: " + attachBtnSelector);
      try {
        floorp.tabWaitForElement(activeTabId, attachBtnSelector, 3000);
        log("✓ Attachment button found with alternative selector");
      } catch (e2) {
        log("✗ Alternative selector also failed: " + e2);
      }
    }

    // ファイル入力の確認
    var fileInputSelector = 'input[data-qa="file_upload"]';
    log("Checking file input: " + fileInputSelector);
    try {
      floorp.tabWaitForElement(activeTabId, fileInputSelector, 3000);
      log("✓ File input found");
    } catch (e) {
      log("✗ File input NOT found: " + e);
    }

    // 送信ボタンの確認
    var sendBtnSelector = '[data-qa="texty_send_button"]';
    log("Checking send button: " + sendBtnSelector);
    try {
      floorp.tabWaitForElement(activeTabId, sendBtnSelector, 3000);
      log("✓ Send button found");
    } catch (e) {
      log("✗ Send button NOT found: " + e);
    }

    // Step 4: ファイルアップロードのテスト
    log("[Step 4] Testing file upload...");
    log("File path: " + TEST_FILE_PATH);

    try {
      // 方法 1: 直接ファイル入力にアップロード
      log("Method 1: Direct file input upload");
      logDebug(
        "Calling tabUploadFile",
        fileInputSelector + " with " + TEST_FILE_PATH
      );

      var uploadResult = floorp.tabUploadFile(
        activeTabId,
        fileInputSelector,
        TEST_FILE_PATH
      );
      logDebug("uploadResult", uploadResult);
      log("✓ tabUploadFile executed");

      // アップロード後の待機
      log("Waiting for upload to process...");
      sleep(2000);

      // ファイルプレビューが表示されたか確認
      log("Checking for file preview...");
      try {
        // アップロードされたファイルのプレビュー要素を探す
        floorp.tabWaitForElement(activeTabId, ".p-file_upload_preview", 3000);
        log("✓ File preview appeared");
      } catch (e) {
        log("File preview not found (may be normal): " + e);
      }

      // 送信ボタンの状態を確認
      log("Checking send button state...");
      try {
        var sendBtnHtml = floorp.tabGetElement(activeTabId, sendBtnSelector);
        logDebug("sendBtnHtml", sendBtnHtml);

        if (sendBtnHtml.indexOf("disabled") === -1) {
          log("✓ Send button is enabled");

          // 実際に送信するか確認（コメントアウト可能）
          log("Clicking send button...");
          floorp.tabClick(activeTabId, sendBtnSelector);
          log("✓ Send button clicked");

          sleep(1000);
          log("✓ File upload and send completed!");
        } else {
          log("Send button is still disabled");
        }
      } catch (e) {
        log("Could not check send button: " + e);
      }
    } catch (e) {
      log("✗ File upload failed: " + e);
      logDebug("Error details", e.stack || e);

      // 方法 2: メニュー経由でアップロード（フォールバック）
      log("");
      log("Trying Method 2: Menu-based upload...");
      try {
        // 添付ボタンをクリック
        log("Clicking attachment button...");
        floorp.tabClick(activeTabId, attachBtnSelector);
        sleep(500);

        // メニューが表示されたか確認
        log("Checking for attachment menu...");
        floorp.tabWaitForElement(activeTabId, '[role="menu"]', 3000);
        log("✓ Attachment menu appeared");

        // 「コンピューターからアップロード」メニューアイテムを探す
        log("Looking for upload menu item...");
        // このセレクタは動的なので、テキストベースで探す必要があるかも
      } catch (e2) {
        log("Method 2 also failed: " + e2);
      }
    }

    log("");
    log("=== Workflow Complete ===");

    return {
      success: true,
      message: "File upload test completed",
    };
  } catch (error) {
    log("ERROR: Workflow failed: " + error);
    logDebug("Error stack", error.stack || error);
    return {
      success: false,
      error: String(error),
    };
  }
}

workflow();
