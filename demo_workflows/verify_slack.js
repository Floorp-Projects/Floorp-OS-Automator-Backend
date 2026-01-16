/**
 * Slack Workflow using Floorp Browser Automation
 *
 * このワークフローは Floorp ブラウザを使用して Slack Web アプリから情報を取得し、
 * メッセージを送信する機能を提供します。
 *
 * 機能:
 * - Slack タブを検出
 * - チャンネル/DM リストの取得
 * - メッセージの読み取り
 * - メッセージの送信
 */

const SLACK_URL = "https://app.slack.com/client/T0A62PPRD7G/C0A68CVNZFE";
const TEST_FILE_PATH = "/Users/user/Desktop/test-upload.txt";

function sleep(ms) {
  const start = Date.now();
  while (Date.now() - start < ms) {}
}

function workflow() {
  console.log("=== Slack Workflow using Floorp ===");
  console.log("");

  let createdTab = false;
  let tabId = null;

  try {
    // Step 1: ブラウザタブから Slack を探す
    console.log("[Step 1] Searching for Slack tab...");
    const tabsResponse = floorp.browserTabs();
    const tabsData = JSON.parse(tabsResponse);
    const tabs = tabsData.tabs || tabsData;

    let slackTab = null;
    for (const tab of tabs) {
      const url = tab.url || "";
      if (url.includes("slack.com") || url.includes("app.slack.com")) {
        slackTab = tab;
        console.log("Found Slack tab: " + tab.title);
        break;
      }
    }

    if (!slackTab) {
      console.log("No Slack tab found. Opening Slack...");
      // Slack を新しいタブで開く
      const createResult = floorp.createTab(SLACK_URL, false);
      try {
        const createData = JSON.parse(createResult);
        tabId = String(createData.instance_id || createData.id);
      } catch (e) {
        tabId = String(createResult);
      }
      createdTab = true;

      // ページ読み込みを待つ
      console.log("Waiting for Slack to load...");
      // ネットワークアイドルを待機（ページ読み込み完了を保証）
      floorp.tabWaitForNetworkIdle(tabId, "15000");
      floorp.tabWaitForElement(tabId, "[data-qa='channel_sidebar']", 10000);

      slackTab = {
        instance_id: tabId,
        url: SLACK_URL,
        title: "Slack",
        status: "complete",
      };
    }

    // Step 2: 既存の Slack タブにアタッチ
    console.log("[Step 2] Attaching to Slack tab...");
    tabId = String(slackTab.instance_id || slackTab.id);
    const attachResult = floorp.attachToTab(tabId);
    console.log("Attached to tab: " + attachResult);

    // Step 3: Slack の情報を取得
    console.log("[Step 3] Getting Slack information...");

    // ワークスペース名を取得
    let workspaceName = "Unknown";
    try {
      const wsResult = floorp.tabAttribute(
        tabId,
        ".p-client_workspace_wrapper",
        "aria-label"
      );
      workspaceName = wsResult || workspaceName;
    } catch (e) {
      console.log("Could not get workspace name: " + e);
    }
    console.log("Workspace: " + workspaceName);

    // 現在のチャンネル名を取得
    let currentChannel = "Unknown";
    try {
      const chResult = floorp.tabElementText(
        tabId,
        ".p-view_header__channel_title"
      );
      currentChannel = chResult || currentChannel;
    } catch (e) {
      try {
        // 別のセレクタを試す
        const chResult2 = floorp.tabElementText(
          tabId,
          ".p-channel_sidebar__channel--selected"
        );
        currentChannel = JSON.parse(chResult2).text || currentChannel;
      } catch (e2) {
        console.log("Could not get current channel: " + e2);
      }
    }
    console.log("Current channel: " + currentChannel);

    // Step 4: ファイルアップロード + メッセージ送信（同時送信）
    console.log("[Step 4] Uploading a file and sending a message...");

    const testMessage = "Hello from Floorp OS Automator! 🚀";
    const inputSelector = '[role="textbox"] p';
    const fileInputSelector = 'input[data-qa="file_upload"]';
    const sendButtonSelector = '[data-qa="texty_send_button"]';

    try {
      // ファイル入力欄を待つ
      floorp.tabWaitForElement(tabId, fileInputSelector, 5000);
      console.log("Found file input");

      // ファイルをアップロード
      floorp.tabUploadFile(tabId, fileInputSelector, TEST_FILE_PATH);
      console.log("Uploaded file: " + TEST_FILE_PATH);

      // アップロード後の待機
      sleep(1500);

      // メッセージ入力欄を待つ
      floorp.tabWaitForElement(tabId, inputSelector, 5000);
      console.log("Found message input");

      // メッセージを入力 (setInnerHTMLを使用 - 紫色ハイライト)
      floorp.tabSetInnerHTML(tabId, inputSelector, testMessage);
      console.log("Entered message using setInnerHTML: " + testMessage);

      // エディタ状態の反映待ち
      sleep(1000);

      // 同じタイミングで送信
      floorp.tabClick(tabId, sendButtonSelector);
      console.log("Message + file sent!");
    } catch (e) {
      console.log("Could not upload file or send message: " + e);
    }

    // Step 5: チャンネルリストを取得
    console.log("[Step 5] Getting channel list...");
    const channels = [];

    try {
      // サイドバーのチャンネル名を持つspan要素を取得
      // .p-channel_sidebar__channel_icon_prefix の隣にある span を取得する
      const selector = ".p-channel_sidebar__channel_icon_prefix + span";
      console.log(
        `[Debug] Executing tabGetElements with selector: "${selector}"`
      );

      const resultJson = floorp.tabGetElements(tabId, selector);
      console.log(`[Debug] Raw result JSON length: ${resultJson.length}`);

      const result = JSON.parse(resultJson); // { elements: string[] }

      const elementStrings = result.elements || [];
      console.log(`[Debug] Found ${elementStrings.length} matching elements.`);

      const attrRegex = /data-qa="channel_sidebar_name_([^"]+)"/;

      for (let i = 0; i < elementStrings.length; i++) {
        const html = elementStrings[i];
        const match = html.match(attrRegex);
        if (match) {
          channels.push(match[1]);
        } else {
          if (i < 3)
            console.log(
              `[Debug] No regex match for element: ${html.substring(0, 100)}...`
            );
        }
        if (channels.length >= 20) break;
      }
      console.log("Found " + channels.length + " channels");
    } catch (e) {
      console.log("Could not get channel list: " + e);
    }

    console.log("");
    console.log("=== Workflow Complete ===");

    return {
      success: true,
      workspace: workspaceName,
      currentChannel: currentChannel,
      channelsFound: channels.length,
      channels: channels.slice(0, 5), // 最初の5つだけ返す
      message: "Slack information retrieved successfully",
    };
  } catch (error) {
    console.error("Workflow failed: " + error);
    return {
      success: false,
      error: String(error),
    };
  } finally {
    // Step Last: インスタンス削除（必要ならタブも閉じる）
    if (tabId) {
      try {
        floorp.destroyTabInstance(tabId);
        console.log("Destroyed tab instance: " + tabId);
      } catch (e) {
        console.log("Could not destroy tab instance: " + e);
      }

      if (createdTab) {
        try {
          floorp.closeTab(tabId);
          console.log("Closed created tab: " + tabId);
        } catch (e) {
          console.log("Could not close tab: " + e);
        }
      }
    }
  }
}

workflow();
