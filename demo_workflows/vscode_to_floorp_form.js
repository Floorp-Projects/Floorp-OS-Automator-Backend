/**
 * Demo Workflow 1: VSCode to Floorp Form
 * 
 * このワークフローは以下の処理を行います:
 * 1. VSCodeでアクティブに表示されているファイルの内容を取得
 * 2. Floorpでアクティブなタブのフォームに内容を入力
 * 
 * 使用する前提:
 * - VSCodeでファイルが開かれている状態
 * - Floorp OSサーバーが起動している
 * - Floorpで入力先のフォームがあるページが開かれている
 */

function workflow() {
    try {
        // Step 1: VSCodeからアクティブファイルの内容を取得
        console.log("Step 1: Getting content from VSCode active file...");
        const content = vscode.get_active_file_content();
        console.log("Got content from VSCode (first 100 chars): " + content.substring(0, 100));
        
        // Step 2: Floorpのブラウザタブを取得
        console.log("Step 2: Getting Floorp browser tabs...");
        const tabsResponse = floorp.listBrowserTabs();
        const tabs = JSON.parse(tabsResponse);
        
        if (!tabs || tabs.length === 0) {
            console.log(JSON.stringify({ ok: false, reason: "No browser tabs found" }));
            return;
        }
        
        // Step 3: 最初のタブにアタッチ
        console.log("Step 3: Attaching to first tab...");
        const firstTab = tabs[0];
        const tabId = firstTab.id || firstTab.tabId;
        const attachResult = floorp.attachToTab(tabId.toString());
        console.log("Attached to tab: " + attachResult);
        
        // Step 4: textarea または input[type="text"] を探してフォームに入力
        // ユーザーはセレクタをカスタマイズ可能
        const formSelector = "textarea, input[type='text'], .form-input";
        console.log("Step 4: Filling form with selector: " + formSelector);
        
        try {
            // フォームにVSCodeの内容を入力
            floorp.tabFillForm(tabId.toString(), formSelector, content);
            console.log(JSON.stringify({ 
                ok: true, 
                message: "Successfully filled form with VSCode content",
                contentLength: content.length
            }));
        } catch (fillError) {
            // textareaが見つからない場合、具体的なセレクタを試す
            console.log("Could not find generic form element, trying specific selectors...");
            
            // よくある入力フィールドのセレクタを試す
            const selectors = ["#content", "#text", "#input", ".input-field", "textarea"];
            let filled = false;
            
            for (const selector of selectors) {
                try {
                    floorp.tabFillForm(tabId.toString(), selector, content);
                    console.log(JSON.stringify({ 
                        ok: true, 
                        message: "Successfully filled form with selector: " + selector,
                        contentLength: content.length
                    }));
                    filled = true;
                    break;
                } catch (e) {
                    // 次のセレクタを試す
                }
            }
            
            if (!filled) {
                console.log(JSON.stringify({ 
                    ok: false, 
                    reason: "Could not find suitable form element",
                    tried_selectors: selectors
                }));
            }
        }
        
    } catch (e) {
        console.log(JSON.stringify({ 
            ok: false, 
            reason: "Workflow failed", 
            error: String(e) 
        }));
    }
}

workflow();
