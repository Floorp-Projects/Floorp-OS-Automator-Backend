// Sapphillon
// Copyright 2025 Yuta Takahashi
//
// This file is part of Sapphillon
//
// Sapphillon is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::env;
use std::error::Error;

use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs},
};

#[allow(dead_code)]
pub fn generate_workflow(_user_query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let workflow = r#"
function workflow() {
    const base = "http://localhost:58261";
    let scraperId;
    try {
        console.log("Creating scraper instance...");
    const scraperResponse = floorp.createScraper(base);
    const scraper = JSON.parse(scraperResponse);
    scraperId = scraper.instanceId ?? scraper.id;
        console.log(`Scraper instance created with ID: ${scraperId}`);

        console.log("Navigating to https://www.google.com...");
        floorp.navigate(base, scraperId, "https://www.google.com");
        console.log("Navigation complete.");

        // Poll URI until it becomes non-empty (up to ~5s)
        console.log("Getting page URI...");
        let currentUri = "";
        for (let i = 0; i < 50; i++) {
            const uriResponse = floorp.uri(base, scraperId);
            const uri = JSON.parse(uriResponse);
            currentUri = uri.uri || "";
            if (currentUri) break;
        }
        console.log(`Page URI: ${currentUri}`);

        // Get HTML
        console.log("Getting page HTML...");
        const htmlResponse = floorp.html(base, scraperId);
        const htmlContent = JSON.parse(htmlResponse);
        console.log("HTML content received.");

        // Take screenshot
        console.log("Taking screenshot...");
        const screenshotResponse = floorp.screenshot(base, scraperId);
        const screenshotContent = JSON.parse(screenshotResponse);
        console.log("Screenshot taken.");

        return {
            uri: currentUri,
            html: htmlContent.html.substring(0, 200), // Return first 200 chars of HTML
            screenshot: screenshotContent.image.substring(0, 100) + "..." // Truncate for display
        };
    } catch (e) {
        console.error(`An error occurred: ${e.toString()}`);
        return { error: e.toString() };
    } finally {
        if (scraperId) {
             console.log(`Destroying scraper instance ${scraperId}...`);
             floorp.destroyScraperInstance(base, scraperId);
             console.log("Scraper instance destroyed.");
        }
    }
}
"#
    .to_string();
    Ok(workflow)
}

pub async fn generate_workflow_async(
    _user_query: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let workflow = r#"
function workflow() {
    const base = "http://localhost:58261";
    let scraperId;
    try {
        console.log("Creating scraper instance...");
    const scraperResponse = floorp.createScraper(base);
    const scraper = JSON.parse(scraperResponse);
    scraperId = scraper.instanceId ?? scraper.id;
        console.log(`Scraper instance created with ID: ${scraperId}`);

        console.log("Navigating to https://www.google.com...");
        floorp.navigate(base, scraperId, "https://www.google.com");
        console.log("Navigation complete.");

        // Poll URI until it becomes non-empty (up to ~5s)
        console.log("Getting page URI...");
        let currentUri = "";
        for (let i = 0; i < 50; i++) {
            const uriResponse = floorp.uri(base, scraperId);
            const uri = JSON.parse(uriResponse);
            currentUri = uri.uri || "";
            if (currentUri) break;
        }
        console.log(`Page URI: ${currentUri}`);

        // Get HTML
        console.log("Getting page HTML...");
        const htmlResponse = floorp.html(base, scraperId);
        const htmlContent = JSON.parse(htmlResponse);
        console.log("HTML content received.");
        console.log(`HTML: ${htmlContent.html}`);

        // Take screenshot
        console.log("Taking screenshot...");
        const screenshotResponse = floorp.screenshot(base, scraperId);
        const screenshotContent = JSON.parse(screenshotResponse);
        console.log("Screenshot taken.");
        console.log(`Screenshot (base64): ${screenshotContent.image}`);

        return {
            uri: currentUri,
            html: htmlContent.html.substring(0, 200), // Return first 200 chars of HTML
            screenshot: screenshotContent.image.substring(0, 100) + "..." // Truncate for display
        };
    } catch (e) {
        console.error(`An error occurred: ${e.toString()}`);
        return { error: e.toString() };
    } finally {
        if (scraperId) {
             console.log(`Destroying scraper instance ${scraperId}...`);
             floorp.destroyScraperInstance(base, scraperId);
             console.log("Scraper instance destroyed.");
        }
    }
}
"#
    .to_string();
    Ok(workflow)
}

#[allow(dead_code)]
fn generate_prompt(user_query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let template = format!(
        r#"
    ## System

    あなたは「Workflow Planner and Generator」です。
    あなたの役割は、与えられたタスクを達成するための **実行可能で明確なワークフロー** を設計し、
    その手順を **実際に動作するPythonファイル `workflow.js`** として出力することです。

    ### 目的
    - ユーザーの質問や依頼を達成するための、再現性・信頼性の高い処理手順を定義する。
    - `workflow.js` 内には **必ず `workflow()` 関数** を含めること。
    - ワークフローは、与えられたToolだけを使って完結すること。

    ### 出力ルール
    - 出力は必ず `<code>` タグで囲まれた **Json Codeのみ**とする。
    - `workflow()` 関数は **定義のみ** を行い、その中に全てのロジックを記述する（関数の外に処理を記述しない）。
    - 実際の関数呼び出しや実行結果の表示は行わない。
    - 各ステップにおいて **コメントで意図や処理内容を説明** すること。
    - 必要に応じて例外処理を入れることで、失敗時の理由を明確にする。
    - 実行結果の出力はすべてconsole.log()を使用すること。

    ### ワークフロー設計ガイドライン
    1. **目的の正確な理解**  
    ユーザーの要求や質問を正確に理解し、達成するべきゴールを明確化する。
    2. **情報不足の補完**  
    - 必要情報が不足している場合は `search()` や `web()` を用いて補完する。  
    - 不確実な情報は `llm()` を使い抽出・要約・推論する。
    3. **処理順序の最適化**  
    - 不要な検索や推論は避け、最小限の手順で目的を達成する。  
    - ただし精度を確保するため、信頼性確認ステップを含める。
    4. **Tool使用の優先度**  
    - 直接的な情報取得が可能な場合： `search()` → `web()`  
    - 推論や抽出が必要な場合： `llm()`
    - 上記の標準ツール以外も状況によって与えられている場合があるので適切に目的やゴールを理解して使用すること
    - 許可されていない外部関数は絶対に使用しない。
    5. **エラー時の対処**  
    - 検索結果が空・Webページが取得不可・抽出失敗などのケースに対応する。
    6. **汎用性**  
    - 歴史、科学、技術、エンタメ、ビジネス、日常的な質問など、あらゆるジャンルに対応できるよう設計する。

    ---

    ### 利用可能なTool
    - `fetch(url: str) -> {{body: str}}`

    ---

    ### 出力例
    <code>
    function workflow() {{
        /**
         * タスクの概要:
         * 1. ユーザー要求の核心を特定
         * 2. 必要な情報を検索または推論で補完
         * 3. 信頼性の高い情報源から必要なデータを抽出
         * 4. 目的に沿った形で加工して返す
        */
        try {{
            // 1. 必要な検索クエリを組み立てて検索を実行します。
            const searchResults = search("");
            if (!searchResults?.results?.length) {{
                throw new Error("検索結果が取得できませんでした。");
            }}

            // 2. 最適な情報源を選択し、Webページの内容を取得します。
            const topUrl = searchResults.results[0].url;
            const page = web(topUrl);
            const markdown = page?.markdown;

            if (!markdown) {{
                throw new Error("Webページの内容が取得できませんでした。");
            }}  

            // 3. LLMを使用して情報を抽出
            const extractPrompt = `以下の情報から必要な項目を抽出してください:\n${{markdown}}`;
            const extracted = llm(extractPrompt);

            return extracted.text;
        }}
    }}
    </code>

    ## User
    User Query(Task):
    - {user_query}
    - 使用言語: ja-JP
    "#
    );
    let prompt = template.replace("{user_query}", user_query);
    Ok(prompt)
}

#[allow(dead_code)]
fn extract_first_code(xml: &str) -> Option<String> {
    let open = "<code>";
    let close = "</code>";

    if let Some(s) = xml.find(open) {
        let s_idx = s + open.len();
        if let Some(e_rel) = xml[s_idx..].find(close) {
            let e_idx = s_idx + e_rel;
            return Some(xml[s_idx..e_idx].to_string());
        }
    }
    None
}

#[allow(dead_code)]
pub fn llm_call(user_query: &str) -> Result<String, Box<dyn Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(_llm_call_async(user_query))
}

pub async fn _llm_call_async(user_query: &str) -> Result<String, Box<dyn Error>> {
    // OpenRouter のエンドポイント
    let api_base = "https://openrouter.ai/api/v1".to_string();
    let api_key = env::var("OPENROUTER_API_KEY")?;

    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base),
    );

    let model: &str = "openai/gpt-oss-20b:free";

    // ユーザー入力をメッセージに反映
    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages([ChatCompletionRequestUserMessageArgs::default()
            .content(user_query)
            .build()?
            .into()])
        .build()?;

    let response = client.chat().create(request).await?;

    // 最初のレスポンスの message.content を取得
    let content = response
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_else(|| "".to_string());

    Ok(content)
}

// .envがない状態ではテストを通過しないため、コメントアウト
// #[test]
// fn test_llm_call() -> Result<(), Box<dyn Error>> {
//     // async fn ではなく sync ラッパーを使う
//     let result: String = llm_call("ロシア語でこんにちはってなんていいいますか？")?;
//     println!("LLM result: {}", result);
//     Ok(())
// }

// #[test]
// fn test_generate_prompt() -> Result<(), Box<dyn Error>> {
//     let prompt = generate_prompt("今日の天気はなんですか?")?;
//     println!("{}", prompt);
//     Ok(())
// }

// #[test]
// fn test_extract_first_code() -> Result<(), Box<dyn Error>> {
//     let result = extract_first_code("<code>Hello World</code>");
//     assert_eq!(result, Some("Hello World".to_string()));
//     println!("Extracted code: {:?}", result);
//     Ok(())
// }

// #[test]
// fn test_generate_workflow() -> Result<(), Box<dyn Error>> {
//     let workflow = generate_workflow("今日の天気はなんですか?")?;
//     println!("{}", workflow);
//     Ok(())
// }
