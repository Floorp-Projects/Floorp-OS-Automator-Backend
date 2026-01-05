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

function workflow() {
  console.log("=== Slack Workflow using Floorp ===");
  console.log("");

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
      const createData = JSON.parse(createResult);
      const tabId = createData.instance_id || createData.id;

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
    const tabId = String(slackTab.instance_id || slackTab.id);
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

    // Step 4: メッセージを入力して送信
    console.log("[Step 4] Sending a test message...");

    const testMessage = "Hello from Floorp OS Automator! 🚀";
    const inputSelector = '[role="textbox"] p';

    try {
      // メッセージ入力欄を待つ
      floorp.tabWaitForElement(tabId, inputSelector, 5000);
      console.log("Found message input");

      // メッセージを入力 (setInnerHTMLを使用 - 紫色ハイライト)
      floorp.tabSetInnerHTML(tabId, inputSelector, testMessage);
      console.log("Entered message using setInnerHTML: " + testMessage);

      // Wait for 1 second to ensure editor state update
      const start = Date.now();
      while (Date.now() - start < 1000) {}

      // 少し待つ
      floorp.tabClick(tabId, '[data-qa="texty_send_button"]');
      console.log("Message sent!");
    } catch (e) {
      console.log("Could not interact with message input: " + e);
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
  }
}

workflow();
