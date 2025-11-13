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
pub fn generate_workflow(user_query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = generate_prompt(user_query)?;
    let workflow_raw = llm_call(&prompt)?;
    // TODO: insert javascript tool code.
    let workflow_code = extract_first_code(&workflow_raw)
        .ok_or_else(|| "No code section found in the response")?;

    let prefix_code = r"
function get_github_commits(repo, user) {
const github_search_url = `https://github.com/search?q=author%3A${user}+repo%3A${repo}&type=commits&s=committer-date&o=desc`;
const tabId = JSON.parse(floorpCreateTab(github_search_url, false)).id;
let commits_html = floorpTabElement(tabId, 'div.Box-sc-62in7e-0.gwXVXe', 1000000);
return commits_html;
}

function get_google_maps_location() {
const tabId = JSON.parse(floorpCreateTab('https://myactivity.google.com/product/maps', false)).id;
floorpTabWaitForElement(tabId, 'span.hFYxqd', 1000000);
let location_html = floorpTabElement(tabId, 'div.vwWeec', 1000000);
return location_html;
}
";
    Ok(format!("{prefix_code}{workflow_code}"))
}

pub async fn generate_workflow_async(
    user_query: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = generate_prompt(user_query)?;
    let workflow_raw = _llm_call_async(&prompt).await?;
    let workflow_code = extract_first_code(&workflow_raw)
    .ok_or_else(|| "No code section found in the response")?;

    let prefix_code = r"
function get_github_commits(repo, user) {
const github_search_url = `https://github.com/search?q=author%3A${user}+repo%3A${repo}&type=commits&s=committer-date&o=desc`;
const tabId = JSON.parse(floorpCreateTab(github_search_url, false)).id;
let commits_html = floorpTabElement(tabId, 'div.Box-sc-62in7e-0.gwXVXe', 5000);
return commits_html;
}

function get_google_maps_location() {
const tabId = JSON.parse(floorpCreateTab('https://myactivity.google.com/product/maps', false)).id;
floorpTabWaitForElement(tabId, 'span.hFYxqd', 5000);
let location_html = floorpTabElement(tabId, 'div.vwWeec', 1000000);
return location_html;
}";
    Ok(format!("{prefix_code}{workflow_code}"))
}

#[allow(dead_code)]
fn generate_prompt(user_query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let today_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let prompt = format!(
        r#"
    ## System

    あなたは「Workflow Planner and Generator」です。
    あなたの役割は、与えられたタスクを達成するための **実行可能で明確なワークフロー** を設計し、
    その手順を **実際に動作するJavascript code `workflow.js`** として出力することです。
    現在時刻: {today_date}

    ### 目的
    - ユーザーの質問や依頼を達成するための、再現性・信頼性の高い処理手順を定義する。
    - `workflow.js` 内には **必ず `workflow()` 関数** を含めること。
    - ワークフローは、与えられたToolだけを使って完結すること。

    ### 出力ルール
    - 出力は必ず ```javascript``` タグで囲まれた **Javascript Codeのみ**とする。
    - `workflow()` 関数は **定義のみ** を行い、その中に全てのロジックを記述する（関数の外に処理を記述しない）。
    - **実際の関数呼び出しや実行結果の表示は行わない。**
    - 各ステップにおいて **コメントで意図や処理内容を説明** すること。
    - 必要に応じて例外処理を入れることで、失敗時の理由を明確にする。
    - 実行結果の出力はすべてconsole.log()を使用すること。

    ### ワークフロー設計ガイドライン
    1. **目的の正確な理解**  
    ユーザーの要求や質問を正確に理解し、達成するべきゴールを明確化する。成功条件・制約・出力形式を明示する。
    2. **情報不足の補完**  
    - 既知の知識で不足する場合は、許可された情報源や資料を適切に参照する。  
    - 不確実な情報は根拠を明示し、仮説と確度を区別する。
    3. **処理順序の最適化**  
    - 直接的で信頼性の高い取得手段を優先し、不要な探索や過度な推論を避ける。  
    - 正確性を確保するため、検証・クロスチェックのステップを含める。
    4. **手段選択の原則**  
    - 事実の取得が必要な場合：一次情報や公式資料などの直接ソースを優先する。  
    - 解釈・要約・推論が必要な場合：構造化・根拠提示を行い、結論の妥当性を示す。  
    - 利用可能な機能は、目的・制約・権限に照らして最小限かつ適切に選ぶ。  
    - 許可されていない機能・外部リソースは使用しない。
    5. **エラー時の対処**  
    - 情報未取得・アクセス不可・抽出失敗などに対し、代替手段、再試行、スコープ調整を検討する。
    6. **難しい問題に対する対処**
    - 難しい問題、複雑問題に対しては、問題を分解し、各要素を個別に検討するアプローチを取る。
    - 難しい問題を単純化して考えるのではなく、問題の本質の本質を捉えて、長くても正確に解決する方法を優先する。
    ---

    ### 利用可能なTool
    - `fetch(url: str) -> str`
    - `console.log(str) -> stdout`
    - `writeFile(path: str, content: str)`
    - `get_github_commits(repo: str, user: str) -> str`
    - `get_google_maps_location() -> str`
    ---

    ### 出力例
    #### 例1: 天気情報の取得
    ```javascript
    function workflow() {{
        const url = "https://api.example.com/data";

        try {{
            // fetch は文字列を返す（ツール仕様）のでそのまま受け取る
            const body = fetch(url);

            // 受け取った文字列を JSON.parse でパースする（失敗検出）
            let data;
            try {{
            data = JSON.parse(body);
            }} catch (e) {{
            console.log(JSON.stringify({{
                ok: false,
                reason: "JSON parse error",
                error: String(e)
            }}));
            return;
        }}

            // 成功時はパースしたオブジェクトを出力する
            console.log(JSON.stringify({{
            ok: true,
            data: data
        }}));
    }} catch (e) {{
            // fetch が例外を投げた場合（ネットワークエラー等）
            console.log(JSON.stringify({{
                ok: false,
                reason: "fetch failed",
                error: String(e)
            }}));
        }}
    }}
    ```
    #### 例2: GitHub コミット履歴の取得
    ```javascript
    function workflow() {{
        const repo = 'Walkmana-25/Sapphillon';
        const user = 'Walkmana-25';
        let commits_html = get_github_commits(repo, user);
        let location_html = get_google_maps_location();
        let url = 'http://host.docker.internal:5050/generate?q=' +  encodeURIComponent(`${{commits_html}}\n\n${{location_html}}\n\n上記のGitHubのコミット履歴とGoogleマップの行動履歴から、その人の作業日報を作成してください。`);
        let llm_res = fetch(url);
        writeFile('./example/nippou_result.txt', llm_res);
        console.log(llm_res);
    }}
    ```
    ## User
    User Query(Task):
    - {user_query}
    - 使用言語: ja-JP
    "#
    );
    Ok(prompt)
}

#[allow(dead_code)]
fn extract_first_code(xml: &str) -> Option<String> {
    let open = "```javascript\n";
    let close = "\n```";

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
    let api_base = &env::var("OPENAI_API_BASE")?;
    let api_key = &env::var("OPENAI_API_KEY")?;
    let model = &env::var("OPENAI_MODEL")?;

    let client = Client::with_config(
        OpenAIConfig::new()
            .with_api_key(api_key)
            .with_api_base(api_base),
    );

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

#[test]
fn test_extract_first_code() -> Result<(), Box<dyn Error>> {
    let result = extract_first_code("```javascript\nHello World\n```");
    assert_eq!(result, Some("Hello World".to_string()));
    println!("Extracted code: {result:?}");
    Ok(())
}

//.envがない状態ではテストを通過しないため、コメントアウト
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

#[test]
fn test_generate_workflow() -> Result<(), Box<dyn Error>> {
    let workflow = generate_workflow("repo: Walkmana-25/Sapphillon, user: Walkmana-25の日報を位置情報とコミット履歴を用いて作成してください。")?;
    println!("{}", workflow);
    Ok(())
}
