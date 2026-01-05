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
      floorp.tabWaitForElement(tabId, "[data-qa='channel_sidebar']", 10000);

      return {
        success: true,
        action: "opened_slack",
        message: "Opened Slack in new tab. Please log in if needed.",
        tabId: tabId,
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
      const wsResult = floorp.tabElementText(tabId, "[data-qa='team-name']");
      workspaceName = JSON.parse(wsResult).text || workspaceName;
    } catch (e) {
      console.log("Could not get workspace name: " + e);
    }
    console.log("Workspace: " + workspaceName);

    // 現在のチャンネル名を取得
    let currentChannel = "Unknown";
    try {
      const chResult = floorp.tabElementText(
        tabId,
        "[data-qa='channel_header_info']"
      );
      currentChannel = JSON.parse(chResult).text || currentChannel;
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

      // メッセージを入力 (contenteditable 対応)
      floorp.tabSetTextContent(tabId, inputSelector, testMessage);
      floorp.tabDispatchEvent(tabId, inputSelector, "input");
      console.log(
        "Entered message using setTextContent/dispatchEvent: " + testMessage
      );

      // 少し待つ
      // (Note: 実際に送信する場合は以下のコメントを外す)
      // floorp.tabClick(tabId, '[data-qa="texty_send_button"]');
      // console.log("Message sent!");

      console.log("NOTE: Send button click is commented out for safety.");
      console.log("Uncomment the tabClick line to actually send messages.");
    } catch (e) {
      console.log("Could not interact with message input: " + e);
    }

    // Step 5: チャンネルリストを取得
    console.log("[Step 5] Getting channel list...");
    const channels = [];

    try {
      // サイドバーのチャンネル要素を取得
      const html = floorp.tabHtml(tabId);
      // HTMLからチャンネル名を抽出（簡易的なパース）
      const channelMatches = html.match(
        /data-qa-channel-sidebar-channel-id="[^"]*"[^>]*>([^<]+)</g
      );
      if (channelMatches) {
        for (let i = 0; i < Math.min(channelMatches.length, 10); i++) {
          const match = channelMatches[i].match(/>([^<]+)</);
          if (match) {
            channels.push(match[1]);
          }
        }
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
