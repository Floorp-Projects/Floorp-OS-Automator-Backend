use deno_core::op2;
use deno_core::serde;
use deno_error::JsErrorBox;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};
use sapphillon_core::proto::sapphillon::v1::{PluginFunction, PluginPackage};

const DEFAULT_BASE: &str = "http://localhost:58261";

fn cfg(token: Option<String>) -> openapi::apis::configuration::Configuration {
    let mut c = openapi::apis::configuration::Configuration::new();
    c.base_path = DEFAULT_BASE.to_string();
    if let Some(t) = token {
        c.bearer_access_token = Some(t);
    }
    c
}

// 同期版クライアントは内部で Tokio ランタイムを利用する可能性があるため、
// Deno/Tokio の非同期コンテキスト直下で drop するとパニックする。
// これを避けるため、すべてのブロッキング処理を専用スレッドで実行する。
fn run_blocking_json<T, F, E>(f: F) -> Result<String, JsErrorBox>
where
    F: Send + 'static + FnOnce() -> Result<T, E>,
    T: serde::Serialize + Send + 'static,
    E: std::fmt::Display,
{
    std::thread::spawn(move || {
        f().map(|r| serde_json::to_string(&r).unwrap())
            .map_err(|e| JsErrorBox::new("Error", e.to_string()))
    })
    .join()
    .map_err(|_| JsErrorBox::new("Error", "thread panicked".to_string()))?
}

pub fn core_floorp_plugin_package() -> CorePluginPackage {
    CorePluginPackage::new(
        "app.sapphillon.core.floorp".to_string(),
        "Floorp".to_string(),
        vec![
            floorp_health_plugin(),
            floorp_create_scraper_instance_plugin(),
            floorp_destroy_scraper_instance_plugin(),
            floorp_close_tab_plugin(),
            floorp_navigate_scraper_plugin(),
            floorp_scraper_html_plugin(),
            floorp_get_elements_plugin(),
            // tab-specific plugins
            floorp_tab_html_plugin(),
            floorp_tab_screenshot_plugin(),
            floorp_tab_get_elements_plugin(),
            floorp_tab_element_plugin(),
            floorp_tab_element_text_plugin(),
            floorp_tab_click_element_plugin(),
            floorp_tab_wait_for_element_plugin(),
            floorp_tab_element_screenshot_plugin(),
            floorp_tab_fullpage_screenshot_plugin(),
            floorp_tab_region_screenshot_plugin(),
            floorp_tab_fill_form_plugin(),
            floorp_tab_set_inner_html_plugin(),
            floorp_tab_set_text_content_plugin(),
            floorp_tab_dispatch_event_plugin(),
            floorp_tab_element_value_plugin(),
            floorp_tab_submit_form_plugin(),
            floorp_tab_clear_input_plugin(),
            floorp_scraper_uri_plugin(),
            floorp_wait_for_element_plugin(),
            floorp_click_element_plugin(),
            floorp_element_text_plugin(),
            floorp_element_value_plugin(),
            floorp_element_by_text_plugin(),
            floorp_element_text_content_plugin(),
            floorp_tab_element_by_text_plugin(),
            floorp_tab_element_text_content_plugin(),
            floorp_fill_form_plugin(),
            floorp_submit_form_plugin(),
            floorp_clear_input_plugin(),
            floorp_screenshot_plugin(),
            floorp_element_screenshot_plugin(),
            floorp_fullpage_screenshot_plugin(),
            floorp_region_screenshot_plugin(),
            floorp_create_tab_instance_plugin(),
            floorp_destroy_tab_instance_plugin(),
            floorp_navigate_tab_plugin(),
            floorp_tab_uri_plugin(),
            floorp_browser_tabs_plugin(),
            floorp_list_browser_tabs_plugin(),
            floorp_browser_history_plugin(),
            floorp_browser_downloads_plugin(),
            floorp_browser_context_plugin(),
            floorp_attach_to_tab_plugin(),
            floorp_check_tab_instance_exists_plugin(),
            floorp_check_scraper_instance_exists_plugin(),
            floorp_attribute_plugin(),
            floorp_is_visible_plugin(),
            floorp_is_enabled_plugin(),
            floorp_select_option_plugin(),
            floorp_set_checked_plugin(),
            floorp_hover_plugin(),
            floorp_scroll_to_plugin(),
            floorp_title_plugin(),
            floorp_double_click_plugin(),
            floorp_right_click_plugin(),
            floorp_focus_plugin(),
            floorp_drag_and_drop_plugin(),
            floorp_cookies_plugin(),
            floorp_set_cookie_plugin(),
            floorp_accept_alert_plugin(),
            floorp_dismiss_alert_plugin(),
            floorp_pdf_plugin(),
            floorp_wait_for_network_idle_plugin(),
            floorp_tab_attribute_plugin(),
            floorp_tab_is_visible_plugin(),
            floorp_tab_is_enabled_plugin(),
            floorp_tab_select_option_plugin(),
            floorp_tab_set_checked_plugin(),
            floorp_tab_hover_plugin(),
            floorp_tab_scroll_to_plugin(),
            floorp_tab_title_plugin(),
            floorp_tab_double_click_plugin(),
            floorp_tab_right_click_plugin(),
            floorp_tab_focus_plugin(),
            floorp_tab_drag_and_drop_plugin(),
            floorp_tab_cookies_plugin(),
            floorp_tab_set_cookie_plugin(),
            floorp_tab_accept_alert_plugin(),
            floorp_tab_dismiss_alert_plugin(),
            floorp_tab_pdf_plugin(),
            floorp_tab_wait_for_network_idle_plugin(),
            // workspace plugins
            floorp_list_workspaces_plugin(),
            floorp_get_current_workspace_plugin(),
            floorp_switch_to_next_workspace_plugin(),
            floorp_switch_to_previous_workspace_plugin(),
            floorp_switch_to_workspace_plugin(),
            // Input / Press Key / Upload File plugins
            floorp_input_plugin(),
            floorp_tab_input_plugin(),
            floorp_press_key_plugin(),
            floorp_tab_press_key_plugin(),
            floorp_upload_file_plugin(),
            floorp_tab_upload_file_plugin(),
        ],
    )
}

pub fn floorp_plugin_package() -> PluginPackage {
    PluginPackage {
        package_id: "app.sapphillon.core.floorp".to_string(),
        package_name: "Floorp".to_string(),
        provider_id: "".to_string(),
        description: "Floorp browser automation primitives.".to_string(),
        functions: floorp_plugin_functions(),
        package_version: env!("CARGO_PKG_VERSION").to_string(),
        deprecated: None,
        plugin_store_url: "BUILTIN".to_string(),
        internal_plugin: Some(true),
        installed_at: None,
        updated_at: None,
        verified: Some(true),
    }
}

fn floorp_plugin_functions() -> Vec<PluginFunction> {
    let mut functions = vec![floorp_plugin_function(
        "health",
        "Health",
        "Floorp OS API health endpoint",
    )];

    functions.extend(
        [
            (
                "createScraperInstance",
                "Create Scraper Instance",
                "Creates a new scraper instance.",
            ),
            (
                "createTabInstance",
                "Create Tab Instance",
                "Creates a new tab instance.",
            ),
            (
                "navigateScraper",
                "Navigate Scraper",
                "Navigate a scraper instance to a URL.",
            ),
            (
                "navigateTab",
                "Navigate Tab",
                "Navigate a tab instance to a URL.",
            ),
            (
                "scraperHtml",
                "Scraper HTML",
                "Get current page HTML of scraper instance.",
            ),
            (
                "getElements",
                "Get Elements",
                "Get matching elements from scraper.",
            ),
            (
                "tabHtml",
                "Tab HTML",
                "Get current page HTML of tab instance.",
            ),
            (
                "tabGetElements",
                "Tab Get Elements",
                "Get matching elements from tab.",
            ),
            (
                "tabScreenshot",
                "Tab Screenshot",
                "Take a screenshot of the tab page (PNG base64).",
            ),
            (
                "tabElement",
                "Tab Element",
                "Get element information from tab by selector.",
            ),
            (
                "tabElementText",
                "Tab Element Text",
                "Get text content of element in tab by selector.",
            ),
            (
                "tabClickElement",
                "Tab Click Element",
                "Click an element in tab by selector.",
            ),
            (
                "tabWaitForElement",
                "Tab Wait For Element",
                "Wait for an element in tab by selector.",
            ),
            (
                "tabElementScreenshot",
                "Tab Element Screenshot",
                "Take a screenshot of an element in tab (PNG base64).",
            ),
            (
                "tabFullPageScreenshot",
                "Tab Full Page Screenshot",
                "Take a full page screenshot of tab (PNG base64).",
            ),
            (
                "tabRegionScreenshot",
                "Tab Region Screenshot",
                "Take a region screenshot of tab (PNG base64).",
            ),
            ("tabFillForm", "Tab Fill Form", "Fill a form in tab."),
            (
                "tabElementValue",
                "Tab Element Value",
                "Get element value in tab by selector.",
            ),
            (
                "tabSubmitForm",
                "Tab Submit Form",
                "Submit a form element in tab.",
            ),
            (
                "scraperUri",
                "Scraper URI",
                "Get current URI of scraper instance.",
            ),
            ("tabUri", "Tab URI", "Get current URI of tab instance."),
            (
                "waitForElement",
                "Wait For Element",
                "Wait for an element by selector.",
            ),
            (
                "clickElement",
                "Click Element",
                "Click an element by selector.",
            ),
            (
                "elementText",
                "Element Text",
                "Get text content of element by selector.",
            ),
            (
                "elementValue",
                "Element Value",
                "Get value of element by selector.",
            ),
            (
                "elementByText",
                "Element By Text",
                "Find first element containing text.",
            ),
            (
                "elementTextContent",
                "Element Text Content",
                "Get trimmed text content for selector.",
            ),
            (
                "tabElementByText",
                "Tab Element By Text",
                "Find first element containing text in tab.",
            ),
            (
                "tabElementTextContent",
                "Tab Element Text Content",
                "Get trimmed text content for selector in tab.",
            ),
            ("fillForm", "Fill Form", "Fill a form element."),
            ("submitForm", "Submit Form", "Submit a form element."),
            (
                "screenshot",
                "Screenshot",
                "Take a screenshot of the page (PNG base64).",
            ),
            (
                "elementScreenshot",
                "Element Screenshot",
                "Take a screenshot of an element (PNG base64).",
            ),
            (
                "fullPageScreenshot",
                "Full Page Screenshot",
                "Take a full page screenshot (PNG base64).",
            ),
            (
                "regionScreenshot",
                "Region Screenshot",
                "Take a region screenshot (PNG base64).",
            ),
            (
                "listBrowserTabs",
                "List Browser Tabs",
                "List browser tabs (lightweight).",
            ),
            (
                "browserTabs",
                "Browser Tabs",
                "Get browser tabs (detailed).",
            ),
            (
                "browserHistory",
                "Browser History",
                "Get browser history list.",
            ),
            (
                "browserDownloads",
                "Browser Downloads",
                "Get browser downloads list.",
            ),
            (
                "browserContext",
                "Browser Context",
                "Get browser context (history/tabs/downloads).",
            ),
            (
                "attachToTab",
                "Attach To Tab",
                "Attach to an existing tab instance.",
            ),
            (
                "destroyTabInstance",
                "Destroy Tab Instance",
                "Destroy a tab instance.",
            ),
            (
                "destroyScraperInstance",
                "Destroy Scraper Instance",
                "Destroy a scraper instance.",
            ),
            (
                "checkTabInstanceExists",
                "Check Tab Instance Exists",
                "Check if tab instance exists.",
            ),
            (
                "checkScraperInstanceExists",
                "Check Scraper Instance Exists",
                "Check if scraper instance exists.",
            ),
            ("attribute", "Get Attribute", "Get element attribute."),
            ("isVisible", "Is Visible", "Check if element is visible."),
            ("isEnabled", "Is Enabled", "Check if element is enabled."),
            (
                "selectOption",
                "Select Option",
                "Select option in dropdown.",
            ),
            (
                "setChecked",
                "Set Checked",
                "Set checkbox/radio checked state.",
            ),
            ("hover", "Hover", "Hover over element."),
            ("scrollTo", "Scroll To", "Scroll to element."),
            ("title", "Get Title", "Get page title."),
            ("doubleClick", "Double Click", "Double click element."),
            ("rightClick", "Right Click", "Right click element."),
            ("focus", "Focus", "Focus element."),
            ("dragAndDrop", "Drag and Drop", "Drag and drop element."),
            ("cookies", "Get Cookies", "Get cookies."),
            ("setCookie", "Set Cookie", "Set cookie."),
            ("acceptAlert", "Accept Alert", "Accept alert."),
            ("dismissAlert", "Dismiss Alert", "Dismiss alert."),
            ("pdf", "Save as PDF", "Save page as PDF."),
            (
                "waitForNetworkIdle",
                "Wait for Network Idle",
                "Wait for network idle.",
            ),
            (
                "tabAttribute",
                "Tab Get Attribute",
                "Get element attribute in tab.",
            ),
            (
                "tabIsVisible",
                "Tab Is Visible",
                "Check if element is visible in tab.",
            ),
            (
                "tabIsEnabled",
                "Tab Is Enabled",
                "Check if element is enabled in tab.",
            ),
            (
                "tabSelectOption",
                "Tab Select Option",
                "Select option in dropdown in tab.",
            ),
            (
                "tabSetChecked",
                "Tab Set Checked",
                "Set checkbox/radio checked state in tab.",
            ),
            ("tabHover", "Tab Hover", "Hover over element in tab."),
            ("tabScrollTo", "Tab Scroll To", "Scroll to element in tab."),
            ("tabTitle", "Tab Get Title", "Get page title in tab."),
            (
                "tabDoubleClick",
                "Tab Double Click",
                "Double click element in tab.",
            ),
            (
                "tabRightClick",
                "Tab Right Click",
                "Right click element in tab.",
            ),
            ("tabFocus", "Tab Focus", "Focus element in tab."),
            (
                "tabDragAndDrop",
                "Tab Drag and Drop",
                "Drag and drop element in tab.",
            ),
            ("tabCookies", "Tab Get Cookies", "Get cookies in tab."),
            ("tabSetCookie", "Tab Set Cookie", "Set cookie in tab."),
            ("tabAcceptAlert", "Tab Accept Alert", "Accept alert in tab."),
            (
                "tabDismissAlert",
                "Tab Dismiss Alert",
                "Dismiss alert in tab.",
            ),
            ("tabPdf", "Tab Save as PDF", "Save page as PDF in tab."),
            (
                "tabWaitForNetworkIdle",
                "Tab Wait for Network Idle",
                "Wait for network idle in tab.",
            ),
            // Advanced input functions (scraper)
            (
                "input",
                "Input",
                "Type into an element.",
            ),
            (
                "pressKey",
                "Press Key",
                "Press key or key combination.",
            ),
            (
                "uploadFile",
                "Upload File",
                "Upload file via input[type=file].",
            ),
            // Advanced input functions (tab)
            (
                "tabInput",
                "Tab Input",
                "Type into an element in tab.",
            ),
            (
                "tabPressKey",
                "Tab Press Key",
                "Press key or key combination in tab.",
            ),
            (
                "tabUploadFile",
                "Tab Upload File",
                "Upload file via input[type=file] in tab.",
            ),
            // Workspace functions
            (
                "listWorkspaces",
                "List Workspaces",
                "List all workspaces with metadata.",
            ),
            (
                "getCurrentWorkspace",
                "Get Current Workspace",
                "Get the current workspace ID.",
            ),
            (
                "switchToNextWorkspace",
                "Switch To Next Workspace",
                "Switch to the next workspace.",
            ),
            (
                "switchToPreviousWorkspace",
                "Switch To Previous Workspace",
                "Switch to the previous workspace.",
            ),
            (
                "switchToWorkspace",
                "Switch To Workspace",
                "Switch to a specific workspace by ID.",
            ),
        ]
        .into_iter()
        .map(|(suffix, name, desc)| floorp_plugin_function(suffix, name, desc)),
    );

    functions
}

fn floorp_plugin_function(suffix: &str, name: &str, description: &str) -> PluginFunction {
    PluginFunction {
        function_id: format!("app.sapphillon.core.floorp.{}", suffix),
        function_name: name.to_string(),
        description: description.to_string(),
        permissions: vec![],
        function_define: None,
        version: "".to_string(),
    }
}

macro_rules! make_plugin {
    ($func:ident, $op:ident, $name:literal, $title:literal, $desc:literal) => {
        pub fn $func() -> CorePluginFunction {
            CorePluginFunction::new(
                format!("app.sapphillon.core.floorp.{}", $name),
                $title.to_string(),
                $desc.to_string(),
                $op(),
                None,
            )
        }
    };
}

// --- op 実装 ---
#[op2]
#[string]
fn op_floorp_health() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_health(&c)
    })
}

#[op2]
#[string]
fn op_floorp_create_scraper_instance() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::create_scraper_instance(&c).map(|r| {
            serde_json::json!({
                "instanceId": r.instance_id,
                "id": r.instance_id,
            })
        })
    })
}

#[op2]
#[string]
fn op_floorp_create_tab_instance(
    #[string] url: String,
    #[string] in_background: Option<String>,
) -> Result<String, JsErrorBox> {
    let mut body = openapi::models::CreateTabInstanceRequest {
        url,
        in_background: None,
    };
    if let Some(b) = in_background {
        body.in_background = b.parse::<bool>().ok();
    }
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::create_tab_instance(&c, body).map(|r| {
            serde_json::json!({
                "instanceId": r.instance_id,
                "id": r.instance_id,
            })
        })
    })
}

#[op2]
#[string]
fn op_floorp_navigate_scraper(
    #[string] id: String,
    #[string] url: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::NavigateRequest { url };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::navigate_scraper_instance(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_navigate_tab(
    #[string] id: String,
    #[string] url: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::NavigateRequest { url };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::navigate_tab_instance(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_scraper_html(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_instance_html(&c, &id).map(|r| {
            let html = r.html.and_then(|h| h).unwrap_or_default();
            serde_json::json!({ "html": html })
        })
    })
}

#[op2]
#[string]
fn op_floorp_scraper_uri(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_instance_uri(&c, &id).map(|r| {
            let uri = r.uri.and_then(|u| u).unwrap_or_default();
            serde_json::json!({ "uri": uri })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_uri(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_instance_uri(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_wait_for_element(
    #[string] id: String,
    #[string] selector: String,
    #[string] timeout_ms: Option<String>,
) -> Result<String, JsErrorBox> {
    let timeout = timeout_ms.and_then(|s| s.parse::<i32>().ok());
    let body = openapi::models::WaitForElementRequest {
        selector: selector.clone(),
        timeout,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::wait_for_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_click_element(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::click_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_element_text(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_element_text(&c, &id, &selector).map(|r| {
            let text = r.text.and_then(|t| t).unwrap_or_default();
            serde_json::json!({ "text": text })
        })
    })
}

#[op2]
#[string]
fn op_floorp_element_value(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_element_value(&c, &id, &selector).map(|r| {
            let value = r.value.and_then(|v| v).unwrap_or_default();
            serde_json::json!({ "value": value })
        })
    })
}

#[op2]
#[string]
fn op_floorp_element_by_text(
    #[string] id: String,
    #[string] text: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_element_by_text(&c, &id, &text).map(|r| {
            let element = r.element.and_then(|e| e).unwrap_or_default();
            serde_json::json!({ "element": element })
        })
    })
}

#[op2]
#[string]
fn op_floorp_element_text_content(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_element_text_content(&c, &id, &selector).map(|r| {
            let text = r.text.and_then(|t| t).unwrap_or_default();
            serde_json::json!({ "text": text })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_element_by_text(
    #[string] id: String,
    #[string] text: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_element_by_text(&c, &id, &text).map(|r| {
            let element = r.element.and_then(|e| e).unwrap_or_default();
            serde_json::json!({ "element": element })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_element_text_content(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_element_text_content(&c, &id, &selector).map(|r| {
            let text = r.text.and_then(|t| t).unwrap_or_default();
            serde_json::json!({ "text": text })
        })
    })
}

#[op2]
#[string]
fn op_floorp_fill_form(
    #[string] id: String,
    #[string] selector: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    let mut map = std::collections::HashMap::new();
    map.insert(selector, value);
    let body = openapi::models::FillFormRequest { form_data: map };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::fill_scraper_form(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_submit_form(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::submit_scraper_form(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_clear_input(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::clear_scraper_input(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_screenshot(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_scraper_screenshot(&c, &id).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_element_screenshot(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_scraper_element_screenshot(&c, &id, &selector).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_fullpage_screenshot(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_scraper_full_page_screenshot(&c, &id).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_region_screenshot(
    #[string] id: String,
    #[string] x: Option<String>,
    #[string] y: Option<String>,
    #[string] w: Option<String>,
    #[string] h: Option<String>,
) -> Result<String, JsErrorBox> {
    let mut rect = openapi::models::Rect::new();
    rect.x = x.and_then(|v| v.parse::<i32>().ok());
    rect.y = y.and_then(|v| v.parse::<i32>().ok());
    rect.width = w.and_then(|v| v.parse::<i32>().ok());
    rect.height = h.and_then(|v| v.parse::<i32>().ok());
    let body = openapi::models::RegionScreenshotRequest {
        rect: Some(Box::new(rect)),
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_scraper_region_screenshot(&c, &id, body).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

// --- Tab ops implementations ---
#[op2]
#[string]
fn op_floorp_tab_html(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_instance_html(&c, &id).map(|r| {
            let html = r.html.and_then(|h| h).unwrap_or_default();
            serde_json::json!({ "html": html })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_screenshot(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_tab_screenshot(&c, &id).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_element(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_element(&c, &id, &selector)
    })
}

#[op2]
#[string]
fn op_floorp_tab_element_text(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_element_text(&c, &id, &selector).map(|r| {
            let text = r.text.and_then(|t| t).unwrap_or_default();
            serde_json::json!({ "text": text })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_click_element(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::click_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_wait_for_element(
    #[string] id: String,
    #[string] selector: String,
    #[string] timeout_ms: Option<String>,
) -> Result<String, JsErrorBox> {
    let timeout = timeout_ms.and_then(|s| s.parse::<i32>().ok());
    let body = openapi::models::WaitForElementRequest {
        selector: selector.clone(),
        timeout,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::wait_for_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_element_screenshot(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_tab_element_screenshot(&c, &id, &selector).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_fullpage_screenshot(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_tab_full_page_screenshot(&c, &id).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_region_screenshot(
    #[string] id: String,
    #[string] x: Option<String>,
    #[string] y: Option<String>,
    #[string] w: Option<String>,
    #[string] h: Option<String>,
) -> Result<String, JsErrorBox> {
    let mut rect = openapi::models::Rect::new();
    rect.x = x.and_then(|v| v.parse::<i32>().ok());
    rect.y = y.and_then(|v| v.parse::<i32>().ok());
    rect.width = w.and_then(|v| v.parse::<i32>().ok());
    rect.height = h.and_then(|v| v.parse::<i32>().ok());
    let body = openapi::models::RegionScreenshotRequest {
        rect: Some(Box::new(rect)),
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::take_tab_region_screenshot(&c, &id, body).map(|r| {
            let image = r.image.and_then(|i| i).unwrap_or_default();
            serde_json::json!({ "image": image })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_fill_form(
    #[string] id: String,
    #[string] selector: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    let mut map = std::collections::HashMap::new();
    map.insert(selector, value);
    let body = openapi::models::FillFormRequest { form_data: map };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::fill_tab_form(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_set_inner_html(
    #[string] id: String,
    #[string] selector: String,
    #[string] html: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SetInnerHtmlRequest { selector, html };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::set_tab_inner_html(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_set_text_content(
    #[string] id: String,
    #[string] selector: String,
    #[string] text: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SetTextContentRequest { selector, text };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::set_tab_text_content(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_dispatch_event(
    #[string] id: String,
    #[string] selector: String,
    #[string] event_type: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::DispatchEventRequest {
        selector,
        event_type,
        options: None,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::dispatch_tab_event(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_element_value(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_element_value(&c, &id, &selector).map(|r| {
            let value = r.value.and_then(|v| v).unwrap_or_default();
            serde_json::json!({ "value": value })
        })
    })
}

#[op2]
#[string]
fn op_floorp_tab_submit_form(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::submit_tab_form(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_clear_input(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::clear_tab_input(&c, &id, body)
    })
}

// ---- Browser / Tab listing & context ----
#[op2]
#[string]
fn op_floorp_list_browser_tabs() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::list_browser_tabs(&c)
    })
}

#[op2]
#[string]
fn op_floorp_browser_tabs() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_browser_tabs(&c)
    })
}

#[op2]
#[string]
fn op_floorp_browser_history(#[string] limit: Option<String>) -> Result<String, JsErrorBox> {
    let lim = limit.and_then(|v| v.parse::<i32>().ok());
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_browser_history(&c, lim)
    })
}

#[op2]
#[string]
fn op_floorp_browser_downloads(#[string] limit: Option<String>) -> Result<String, JsErrorBox> {
    let lim = limit.and_then(|v| v.parse::<i32>().ok());
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_browser_downloads(&c, lim)
    })
}

#[op2]
#[string]
fn op_floorp_browser_context(
    #[string] history_limit: Option<String>,
    #[string] download_limit: Option<String>,
) -> Result<String, JsErrorBox> {
    let h = history_limit.and_then(|v| v.parse::<i32>().ok());
    let d = download_limit.and_then(|v| v.parse::<i32>().ok());
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_browser_context(&c, h, d)
    })
}

// ---- Attach / Destroy / Exists ----
#[op2]
#[string]
fn op_floorp_attach_to_tab(#[string] browser_id: String) -> Result<String, JsErrorBox> {
    let body = openapi::models::AttachRequest { browser_id };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::attach_to_tab(&c, body)
    })
}

#[op2]
#[string]
fn op_floorp_destroy_tab_instance(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::destroy_tab_instance(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_destroy_scraper_instance(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::destroy_scraper_instance(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_close_tab(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::close_tab(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_check_tab_instance_exists(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::check_tab_instance_exists(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_check_scraper_instance_exists(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::check_scraper_instance_exists(&c, &id)
    })
}

// --- PluginFunction ラッパ ---
pub fn floorp_health_plugin() -> CorePluginFunction {
    // JS グルー (globalThis.floorp ...) を最初のプラグイン関数経由で注入
    CorePluginFunction::new(
        "app.sapphillon.core.floorp.health".to_string(),
        "Health".to_string(),
        "Floorp OS API health endpoint".to_string(),
        op_floorp_health(),
        Some(include_str!("00_floorp.js").to_string()),
    )
}
make_plugin!(
    floorp_create_scraper_instance_plugin,
    op_floorp_create_scraper_instance,
    "createScraperInstance",
    "Create Scraper Instance",
    "Creates a new scraper instance."
);
make_plugin!(
    floorp_create_tab_instance_plugin,
    op_floorp_create_tab_instance,
    "createTabInstance",
    "Create Tab Instance",
    "Creates a new tab instance."
);
make_plugin!(
    floorp_navigate_scraper_plugin,
    op_floorp_navigate_scraper,
    "navigateScraper",
    "Navigate Scraper",
    "Navigate a scraper instance to a URL."
);
make_plugin!(
    floorp_navigate_tab_plugin,
    op_floorp_navigate_tab,
    "navigateTab",
    "Navigate Tab",
    "Navigate a tab instance to a URL."
);
make_plugin!(
    floorp_scraper_html_plugin,
    op_floorp_scraper_html,
    "scraperHtml",
    "Scraper HTML",
    "Get current page HTML of scraper instance."
);
make_plugin!(
    floorp_tab_html_plugin,
    op_floorp_tab_html,
    "tabHtml",
    "Tab HTML",
    "Get current page HTML of tab instance."
);
make_plugin!(
    floorp_tab_screenshot_plugin,
    op_floorp_tab_screenshot,
    "tabScreenshot",
    "Tab Screenshot",
    "Take a screenshot of the tab page (PNG base64)."
);
make_plugin!(
    floorp_tab_element_plugin,
    op_floorp_tab_element,
    "tabElement",
    "Tab Element",
    "Get element information from tab by selector."
);
make_plugin!(
    floorp_tab_element_text_plugin,
    op_floorp_tab_element_text,
    "tabElementText",
    "Tab Element Text",
    "Get text content of element in tab by selector."
);
make_plugin!(
    floorp_tab_click_element_plugin,
    op_floorp_tab_click_element,
    "tabClickElement",
    "Tab Click Element",
    "Click an element in tab by selector."
);
make_plugin!(
    floorp_tab_wait_for_element_plugin,
    op_floorp_tab_wait_for_element,
    "tabWaitForElement",
    "Tab Wait For Element",
    "Wait for an element in tab by selector."
);
make_plugin!(
    floorp_tab_element_screenshot_plugin,
    op_floorp_tab_element_screenshot,
    "tabElementScreenshot",
    "Tab Element Screenshot",
    "Take a screenshot of an element in tab (PNG base64)."
);
make_plugin!(
    floorp_tab_fullpage_screenshot_plugin,
    op_floorp_tab_fullpage_screenshot,
    "tabFullPageScreenshot",
    "Tab Full Page Screenshot",
    "Take a full page screenshot of tab (PNG base64)."
);
make_plugin!(
    floorp_tab_region_screenshot_plugin,
    op_floorp_tab_region_screenshot,
    "tabRegionScreenshot",
    "Tab Region Screenshot",
    "Take a region screenshot of tab (PNG base64)."
);
make_plugin!(
    floorp_tab_fill_form_plugin,
    op_floorp_tab_fill_form,
    "tabFillForm",
    "Tab Fill Form",
    "Fill a form in tab."
);
make_plugin!(
    floorp_tab_set_inner_html_plugin,
    op_floorp_tab_set_inner_html,
    "tabSetInnerHTML",
    "Tab Set innerHTML",
    "Set innerHTML of an element in tab (for contenteditable elements like Slack)."
);
make_plugin!(
    floorp_tab_set_text_content_plugin,
    op_floorp_tab_set_text_content,
    "tabSetTextContent",
    "Tab Set textContent",
    "Set textContent of an element in tab (for contenteditable elements like Slack)."
);
make_plugin!(
    floorp_tab_dispatch_event_plugin,
    op_floorp_tab_dispatch_event,
    "tabDispatchEvent",
    "Tab Dispatch Event",
    "Dispatch an event on an element in tab (for contenteditable elements)."
);
make_plugin!(
    floorp_tab_element_value_plugin,
    op_floorp_tab_element_value,
    "tabElementValue",
    "Tab Element Value",
    "Get element value in tab by selector."
);
make_plugin!(
    floorp_tab_submit_form_plugin,
    op_floorp_tab_submit_form,
    "tabSubmitForm",
    "Tab Submit Form",
    "Submit a form element in tab."
);
make_plugin!(
    floorp_tab_clear_input_plugin,
    op_floorp_tab_clear_input,
    "tabClearInput",
    "Tab Clear Input",
    "Clear an input field in tab."
);
make_plugin!(
    floorp_scraper_uri_plugin,
    op_floorp_scraper_uri,
    "scraperUri",
    "Scraper URI",
    "Get current URI of scraper instance."
);
make_plugin!(
    floorp_tab_uri_plugin,
    op_floorp_tab_uri,
    "tabUri",
    "Tab URI",
    "Get current URI of tab instance."
);
make_plugin!(
    floorp_wait_for_element_plugin,
    op_floorp_wait_for_element,
    "waitForElement",
    "Wait For Element",
    "Wait for an element by selector."
);
make_plugin!(
    floorp_click_element_plugin,
    op_floorp_click_element,
    "clickElement",
    "Click Element",
    "Click an element by selector."
);
make_plugin!(
    floorp_element_text_plugin,
    op_floorp_element_text,
    "elementText",
    "Element Text",
    "Get text content of element by selector."
);
make_plugin!(
    floorp_element_value_plugin,
    op_floorp_element_value,
    "elementValue",
    "Element Value",
    "Get value of element by selector."
);
make_plugin!(
    floorp_element_by_text_plugin,
    op_floorp_element_by_text,
    "elementByText",
    "Element By Text",
    "Find first element containing text."
);
make_plugin!(
    floorp_element_text_content_plugin,
    op_floorp_element_text_content,
    "elementTextContent",
    "Element Text Content",
    "Get trimmed text content for selector."
);
make_plugin!(
    floorp_tab_element_by_text_plugin,
    op_floorp_tab_element_by_text,
    "tabElementByText",
    "Tab Element By Text",
    "Find first element containing text in tab."
);
make_plugin!(
    floorp_tab_element_text_content_plugin,
    op_floorp_tab_element_text_content,
    "tabElementTextContent",
    "Tab Element Text Content",
    "Get trimmed text content for selector in tab."
);
make_plugin!(
    floorp_fill_form_plugin,
    op_floorp_fill_form,
    "fillForm",
    "Fill Form",
    "Fill a form element."
);
make_plugin!(
    floorp_submit_form_plugin,
    op_floorp_submit_form,
    "submitForm",
    "Submit Form",
    "Submit a form element."
);
make_plugin!(
    floorp_clear_input_plugin,
    op_floorp_clear_input,
    "clearInput",
    "Clear Input",
    "Clear an input field."
);
make_plugin!(
    floorp_screenshot_plugin,
    op_floorp_screenshot,
    "screenshot",
    "Screenshot",
    "Take a screenshot of the page (PNG base64)."
);
make_plugin!(
    floorp_element_screenshot_plugin,
    op_floorp_element_screenshot,
    "elementScreenshot",
    "Element Screenshot",
    "Take a screenshot of an element (PNG base64)."
);
make_plugin!(
    floorp_fullpage_screenshot_plugin,
    op_floorp_fullpage_screenshot,
    "fullPageScreenshot",
    "Full Page Screenshot",
    "Take a full page screenshot (PNG base64)."
);
make_plugin!(
    floorp_region_screenshot_plugin,
    op_floorp_region_screenshot,
    "regionScreenshot",
    "Region Screenshot",
    "Take a region screenshot (PNG base64)."
);
make_plugin!(
    floorp_list_browser_tabs_plugin,
    op_floorp_list_browser_tabs,
    "listBrowserTabs",
    "List Browser Tabs",
    "List browser tabs (lightweight)."
);
make_plugin!(
    floorp_browser_tabs_plugin,
    op_floorp_browser_tabs,
    "browserTabs",
    "Browser Tabs",
    "Get browser tabs (detailed)."
);
make_plugin!(
    floorp_browser_history_plugin,
    op_floorp_browser_history,
    "browserHistory",
    "Browser History",
    "Get browser history list."
);
make_plugin!(
    floorp_browser_downloads_plugin,
    op_floorp_browser_downloads,
    "browserDownloads",
    "Browser Downloads",
    "Get browser downloads list."
);
make_plugin!(
    floorp_browser_context_plugin,
    op_floorp_browser_context,
    "browserContext",
    "Browser Context",
    "Get browser context (history/tabs/downloads)."
);
make_plugin!(
    floorp_attach_to_tab_plugin,
    op_floorp_attach_to_tab,
    "attachToTab",
    "Attach To Tab",
    "Attach to an existing tab instance."
);
make_plugin!(
    floorp_destroy_tab_instance_plugin,
    op_floorp_destroy_tab_instance,
    "destroyTabInstance",
    "Destroy Tab Instance",
    "Destroy a tab instance."
);
make_plugin!(
    floorp_destroy_scraper_instance_plugin,
    op_floorp_destroy_scraper_instance,
    "destroyScraperInstance",
    "Destroy Scraper Instance",
    "Destroy a scraper instance."
);
make_plugin!(
    floorp_close_tab_plugin,
    op_floorp_close_tab,
    "closeTab",
    "Close Tab",
    "Close a tab instance."
);
make_plugin!(
    floorp_check_tab_instance_exists_plugin,
    op_floorp_check_tab_instance_exists,
    "checkTabInstanceExists",
    "Check Tab Instance Exists",
    "Check if tab instance exists."
);
make_plugin!(
    floorp_check_scraper_instance_exists_plugin,
    op_floorp_check_scraper_instance_exists,
    "checkScraperInstanceExists",
    "Check Scraper Instance Exists",
    "Check if scraper instance exists."
);

// --- New Scraper Ops ---

#[op2]
#[string]
fn op_floorp_get_elements(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_elements(&c, &id, &selector)
            .map(|r| serde_json::json!({ "elements": r.elements }))
    })
}

#[op2]
#[string]
fn op_floorp_tab_get_elements(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_elements(&c, &id, &selector)
            .map(|r| serde_json::json!({ "elements": r.elements }))
    })
}

#[op2]
#[string]
fn op_floorp_attribute(
    #[string] id: String,
    #[string] selector: String,
    #[string] name: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_attribute(&c, &id, &selector, &name)
            .map(|r| serde_json::json!({ "value": r.value }))
    })
}

#[op2]
#[string]
fn op_floorp_is_visible(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::is_scraper_visible(&c, &id, &selector)
            .map(|r| serde_json::json!({ "visible": r.visible }))
    })
}

#[op2]
#[string]
fn op_floorp_is_enabled(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::is_scraper_enabled(&c, &id, &selector)
            .map(|r| serde_json::json!({ "enabled": r.enabled }))
    })
}

#[op2]
#[string]
fn op_floorp_select_option(
    #[string] id: String,
    #[string] selector: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectOptionRequest { selector, value };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::select_scraper_option(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_set_checked(
    #[string] id: String,
    #[string] selector: String,
    #[string] checked: String, // "true" or "false"
) -> Result<String, JsErrorBox> {
    let checked_bool = checked.parse::<bool>().unwrap_or(false);
    let body = openapi::models::SetCheckedRequest {
        selector,
        checked: checked_bool,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::set_scraper_checked(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_hover(#[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::hover_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_scroll_to(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::scroll_to_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_title(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_title(&c, &id)
            .map(|r| serde_json::json!({ "title": r.title }))
    })
}

#[op2]
#[string]
fn op_floorp_double_click(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::double_click_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_right_click(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::right_click_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_focus(#[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::focus_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_drag_and_drop(
    #[string] id: String,
    #[string] source: String,
    #[string] target: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::DragAndDropRequest {
        source_selector: source,
        target_selector: target,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::drag_and_drop_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_cookies(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_scraper_cookies(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_set_cookie(
    #[string] id: String,
    #[string] cookie_json: String,
) -> Result<String, JsErrorBox> {
    let body: openapi::models::CookieData =
        serde_json::from_str(&cookie_json).map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::set_scraper_cookie(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_accept_alert(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::accept_scraper_alert(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_dismiss_alert(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::dismiss_scraper_alert(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_pdf(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::save_scraper_pdf(&c, &id)
            .map(|r| serde_json::json!({ "pdf": r.pdf }))
    })
}

#[op2]
#[string]
fn op_floorp_wait_for_network_idle(
    #[string] id: String,
    #[string] timeout_ms: Option<String>,
) -> Result<String, JsErrorBox> {
    let mut body = openapi::models::WaitForNetworkIdleRequest { timeout: None };
    if let Some(t) = timeout_ms {
        body.timeout = t.parse::<i32>().ok();
    }
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::wait_for_scraper_network_idle(&c, &id, body)
    })
}

// --- New Tab Ops ---

#[op2]
#[string]
fn op_floorp_tab_attribute(
    #[string] id: String,
    #[string] selector: String,
    #[string] name: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_attribute(&c, &id, &selector, &name)
            .map(|r| serde_json::json!({ "value": r.value }))
    })
}

#[op2]
#[string]
fn op_floorp_tab_is_visible(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::is_tab_visible(&c, &id, &selector)
            .map(|r| serde_json::json!({ "visible": r.visible }))
    })
}

#[op2]
#[string]
fn op_floorp_tab_is_enabled(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::is_tab_enabled(&c, &id, &selector)
            .map(|r| serde_json::json!({ "enabled": r.enabled }))
    })
}

#[op2]
#[string]
fn op_floorp_tab_select_option(
    #[string] id: String,
    #[string] selector: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectOptionRequest { selector, value };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::select_tab_option(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_set_checked(
    #[string] id: String,
    #[string] selector: String,
    #[string] checked: String,
) -> Result<String, JsErrorBox> {
    let checked_bool = checked.parse::<bool>().unwrap_or(false);
    let body = openapi::models::SetCheckedRequest {
        selector,
        checked: checked_bool,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::set_tab_checked(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_hover(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::hover_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_scroll_to(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::scroll_to_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_title(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_title(&c, &id)
            .map(|r| serde_json::json!({ "title": r.title }))
    })
}

#[op2]
#[string]
fn op_floorp_tab_double_click(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::double_click_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_right_click(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::right_click_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_focus(
    #[string] id: String,
    #[string] selector: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::SelectorRequest { selector };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::focus_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_drag_and_drop(
    #[string] id: String,
    #[string] source: String,
    #[string] target: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::DragAndDropRequest {
        source_selector: source,
        target_selector: target,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::drag_and_drop_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_cookies(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_tab_cookies(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_tab_set_cookie(
    #[string] id: String,
    #[string] cookie_json: String,
) -> Result<String, JsErrorBox> {
    let body: openapi::models::CookieData =
        serde_json::from_str(&cookie_json).map_err(|e| JsErrorBox::new("Error", e.to_string()))?;
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::set_tab_cookie(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_accept_alert(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::accept_tab_alert(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_tab_dismiss_alert(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::dismiss_tab_alert(&c, &id)
    })
}

#[op2]
#[string]
fn op_floorp_tab_pdf(#[string] id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::save_tab_pdf(&c, &id)
            .map(|r| serde_json::json!({ "pdf": r.pdf }))
    })
}

#[op2]
#[string]
fn op_floorp_tab_wait_for_network_idle(
    #[string] id: String,
    #[string] timeout_ms: Option<String>,
) -> Result<String, JsErrorBox> {
    let mut body = openapi::models::WaitForNetworkIdleRequest { timeout: None };
    if let Some(t) = timeout_ms {
        body.timeout = t.parse::<i32>().ok();
    }
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::wait_for_tab_network_idle(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_list_workspaces() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::list_workspaces(&c)
    })
}

#[op2]
#[string]
fn op_floorp_get_current_workspace() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::get_current_workspace(&c)
    })
}

#[op2]
#[string]
fn op_floorp_switch_to_next_workspace() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::switch_to_next_workspace(&c)
    })
}

#[op2]
#[string]
fn op_floorp_switch_to_previous_workspace() -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::switch_to_previous_workspace(&c)
    })
}

#[op2]
#[string]
fn op_floorp_switch_to_workspace(#[string] workspace_id: String) -> Result<String, JsErrorBox> {
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::switch_to_workspace(&c, &workspace_id)
    })
}

make_plugin!(
    floorp_attribute_plugin,
    op_floorp_attribute,
    "attribute",
    "Get Attribute",
    "Get element attribute."
);
make_plugin!(
    floorp_is_visible_plugin,
    op_floorp_is_visible,
    "isVisible",
    "Is Visible",
    "Check if element is visible."
);
make_plugin!(
    floorp_is_enabled_plugin,
    op_floorp_is_enabled,
    "isEnabled",
    "Is Enabled",
    "Check if element is enabled."
);
make_plugin!(
    floorp_select_option_plugin,
    op_floorp_select_option,
    "selectOption",
    "Select Option",
    "Select option in dropdown."
);
make_plugin!(
    floorp_set_checked_plugin,
    op_floorp_set_checked,
    "setChecked",
    "Set Checked",
    "Set checkbox/radio checked state."
);
make_plugin!(
    floorp_hover_plugin,
    op_floorp_hover,
    "hover",
    "Hover",
    "Hover over element."
);
make_plugin!(
    floorp_scroll_to_plugin,
    op_floorp_scroll_to,
    "scrollTo",
    "Scroll To",
    "Scroll to element."
);
make_plugin!(
    floorp_title_plugin,
    op_floorp_title,
    "title",
    "Get Title",
    "Get page title."
);
make_plugin!(
    floorp_double_click_plugin,
    op_floorp_double_click,
    "doubleClick",
    "Double Click",
    "Double click element."
);
make_plugin!(
    floorp_right_click_plugin,
    op_floorp_right_click,
    "rightClick",
    "Right Click",
    "Right click element."
);
make_plugin!(
    floorp_focus_plugin,
    op_floorp_focus,
    "focus",
    "Focus",
    "Focus element."
);
make_plugin!(
    floorp_drag_and_drop_plugin,
    op_floorp_drag_and_drop,
    "dragAndDrop",
    "Drag and Drop",
    "Drag and drop element."
);
make_plugin!(
    floorp_cookies_plugin,
    op_floorp_cookies,
    "cookies",
    "Get Cookies",
    "Get cookies."
);
make_plugin!(
    floorp_set_cookie_plugin,
    op_floorp_set_cookie,
    "setCookie",
    "Set Cookie",
    "Set cookie."
);
make_plugin!(
    floorp_accept_alert_plugin,
    op_floorp_accept_alert,
    "acceptAlert",
    "Accept Alert",
    "Accept alert."
);
make_plugin!(
    floorp_dismiss_alert_plugin,
    op_floorp_dismiss_alert,
    "dismissAlert",
    "Dismiss Alert",
    "Dismiss alert."
);
make_plugin!(
    floorp_pdf_plugin,
    op_floorp_pdf,
    "pdf",
    "Save as PDF",
    "Save page as PDF."
);
make_plugin!(
    floorp_wait_for_network_idle_plugin,
    op_floorp_wait_for_network_idle,
    "waitForNetworkIdle",
    "Wait for Network Idle",
    "Wait for network idle."
);
make_plugin!(
    floorp_tab_attribute_plugin,
    op_floorp_tab_attribute,
    "tabAttribute",
    "Tab Get Attribute",
    "Get element attribute in tab."
);
make_plugin!(
    floorp_tab_is_visible_plugin,
    op_floorp_tab_is_visible,
    "tabIsVisible",
    "Tab Is Visible",
    "Check if element is visible in tab."
);
make_plugin!(
    floorp_tab_is_enabled_plugin,
    op_floorp_tab_is_enabled,
    "tabIsEnabled",
    "Tab Is Enabled",
    "Check if element is enabled in tab."
);
make_plugin!(
    floorp_tab_select_option_plugin,
    op_floorp_tab_select_option,
    "tabSelectOption",
    "Tab Select Option",
    "Select option in dropdown in tab."
);
make_plugin!(
    floorp_tab_set_checked_plugin,
    op_floorp_tab_set_checked,
    "tabSetChecked",
    "Tab Set Checked",
    "Set checkbox/radio checked state in tab."
);
make_plugin!(
    floorp_tab_hover_plugin,
    op_floorp_tab_hover,
    "tabHover",
    "Tab Hover",
    "Hover over element in tab."
);
make_plugin!(
    floorp_tab_scroll_to_plugin,
    op_floorp_tab_scroll_to,
    "tabScrollTo",
    "Tab Scroll To",
    "Scroll to element in tab."
);
make_plugin!(
    floorp_tab_title_plugin,
    op_floorp_tab_title,
    "tabTitle",
    "Tab Get Title",
    "Get page title in tab."
);
make_plugin!(
    floorp_tab_double_click_plugin,
    op_floorp_tab_double_click,
    "tabDoubleClick",
    "Tab Double Click",
    "Double click element in tab."
);
make_plugin!(
    floorp_tab_right_click_plugin,
    op_floorp_tab_right_click,
    "tabRightClick",
    "Tab Right Click",
    "Right click element in tab."
);
make_plugin!(
    floorp_tab_focus_plugin,
    op_floorp_tab_focus,
    "tabFocus",
    "Tab Focus",
    "Focus element in tab."
);
make_plugin!(
    floorp_tab_drag_and_drop_plugin,
    op_floorp_tab_drag_and_drop,
    "tabDragAndDrop",
    "Tab Drag and Drop",
    "Drag and drop element in tab."
);
make_plugin!(
    floorp_tab_cookies_plugin,
    op_floorp_tab_cookies,
    "tabCookies",
    "Tab Get Cookies",
    "Get cookies in tab."
);
make_plugin!(
    floorp_tab_set_cookie_plugin,
    op_floorp_tab_set_cookie,
    "tabSetCookie",
    "Tab Set Cookie",
    "Set cookie in tab."
);
make_plugin!(
    floorp_tab_accept_alert_plugin,
    op_floorp_tab_accept_alert,
    "tabAcceptAlert",
    "Tab Accept Alert",
    "Accept alert in tab."
);
make_plugin!(
    floorp_tab_dismiss_alert_plugin,
    op_floorp_tab_dismiss_alert,
    "tabDismissAlert",
    "Tab Dismiss Alert",
    "Dismiss alert in tab."
);
make_plugin!(
    floorp_tab_pdf_plugin,
    op_floorp_tab_pdf,
    "tabPdf",
    "Tab Save as PDF",
    "Save page as PDF in tab."
);
make_plugin!(
    floorp_tab_wait_for_network_idle_plugin,
    op_floorp_tab_wait_for_network_idle,
    "tabWaitForNetworkIdle",
    "Tab Wait for Network Idle",
    "Wait for network idle in tab."
);
make_plugin!(
    floorp_list_workspaces_plugin,
    op_floorp_list_workspaces,
    "listWorkspaces",
    "List Workspaces",
    "List all workspaces with metadata."
);
make_plugin!(
    floorp_get_current_workspace_plugin,
    op_floorp_get_current_workspace,
    "getCurrentWorkspace",
    "Get Current Workspace",
    "Get the current workspace ID."
);
make_plugin!(
    floorp_switch_to_next_workspace_plugin,
    op_floorp_switch_to_next_workspace,
    "switchToNextWorkspace",
    "Switch To Next Workspace",
    "Switch to the next workspace."
);
make_plugin!(
    floorp_switch_to_previous_workspace_plugin,
    op_floorp_switch_to_previous_workspace,
    "switchToPreviousWorkspace",
    "Switch To Previous Workspace",
    "Switch to the previous workspace."
);
make_plugin!(
    floorp_switch_to_workspace_plugin,
    op_floorp_switch_to_workspace,
    "switchToWorkspace",
    "Switch To Workspace",
    "Switch to a specific workspace by ID."
);

make_plugin!(
    floorp_get_elements_plugin,
    op_floorp_get_elements,
    "getElements",
    "Get Elements",
    "Get all matching elements (outerHTML)."
);

make_plugin!(
    floorp_tab_get_elements_plugin,
    op_floorp_tab_get_elements,
    "tabGetElements",
    "Tab Get Elements",
    "Get all matching elements in tab (outerHTML)."
);

// --- Input / Press Key / Upload File operations ---

#[op2]
#[string]
fn op_floorp_input(
    #[string] id: String,
    #[string] selector: String,
    #[string] value: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::InputRequest {
        selector,
        value,
        typing_mode: None,
        typing_delay_ms: None,
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::input_scraper_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_input(
    #[string] id: String,
    #[string] selector: String,
    #[string] value: String,
    typing_mode: bool,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::InputRequest {
        selector,
        value,
        typing_mode: Some(typing_mode),
        typing_delay_ms: if typing_mode { Some(10) } else { None },
    };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::input_tab_element(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_press_key(
    #[string] id: String,
    #[string] key: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::PressKeyRequest { key };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::press_scraper_key(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_press_key(
    #[string] id: String,
    #[string] key: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::PressKeyRequest { key };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::press_tab_key(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_upload_file(
    #[string] id: String,
    #[string] selector: String,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::UploadFileRequest { selector, file_path };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::upload_scraper_file(&c, &id, body)
    })
}

#[op2]
#[string]
fn op_floorp_tab_upload_file(
    #[string] id: String,
    #[string] selector: String,
    #[string] file_path: String,
) -> Result<String, JsErrorBox> {
    let body = openapi::models::UploadFileRequest { selector, file_path };
    run_blocking_json(move || {
        let c = cfg(None);
        openapi::apis::default_api::upload_tab_file(&c, &id, body)
    })
}

make_plugin!(
    floorp_input_plugin,
    op_floorp_input,
    "input",
    "Input",
    "Type into an element."
);

make_plugin!(
    floorp_tab_input_plugin,
    op_floorp_tab_input,
    "tabInput",
    "Tab Input",
    "Type into an element in tab."
);

make_plugin!(
    floorp_press_key_plugin,
    op_floorp_press_key,
    "pressKey",
    "Press Key",
    "Press key or key combination."
);

make_plugin!(
    floorp_tab_press_key_plugin,
    op_floorp_tab_press_key,
    "tabPressKey",
    "Tab Press Key",
    "Press key or key combination in tab."
);

make_plugin!(
    floorp_upload_file_plugin,
    op_floorp_upload_file,
    "uploadFile",
    "Upload File",
    "Upload file via input[type=file]."
);

make_plugin!(
    floorp_tab_upload_file_plugin,
    op_floorp_tab_upload_file,
    "tabUploadFile",
    "Tab Upload File",
    "Upload file via input[type=file] in tab."
);
