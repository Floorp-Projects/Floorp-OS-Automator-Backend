use deno_core::op2;
use deno_error::JsErrorBox;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};

const DEFAULT_BASE: &str = "http://127.0.0.1:58261";

fn cfg(base: Option<String>, token: Option<String>) -> openapi::apis::configuration::Configuration {
	let mut c = openapi::apis::configuration::Configuration::new();
	c.base_path = base.unwrap_or_else(|| DEFAULT_BASE.to_string());
	if let Some(t) = token { c.bearer_access_token = Some(t); }
	c
}

pub fn floorp_plugin_package() -> CorePluginPackage {
	CorePluginPackage::new(
		"app.sapphillon.core.floorp".to_string(),
		"Floorp".to_string(),
		vec![
			floorp_health_plugin(),
			floorp_create_scraper_instance_plugin(),
			floorp_destroy_scraper_instance_plugin(),
			floorp_navigate_scraper_plugin(),
			floorp_scraper_html_plugin(),
			floorp_scraper_uri_plugin(),
			floorp_wait_for_element_plugin(),
			floorp_click_element_plugin(),
			floorp_element_text_plugin(),
			floorp_element_value_plugin(),
			floorp_fill_form_plugin(),
			floorp_submit_form_plugin(),
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
		],
	)
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
#[op2(async)]
#[string]
async fn op_floorp_health(#[string] base: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_health(&c).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_create_scraper_instance(#[string] base: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::create_scraper_instance(&c).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_create_tab_instance(#[string] base: Option<String>, #[string] url: String, #[string] in_background: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let mut body = openapi::models::CreateTabInstanceRequest { url, in_background: None };
	if let Some(b) = in_background { body.in_background = b.parse::<bool>().ok(); }
	openapi::apis::default_api::create_tab_instance(&c, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_navigate_scraper(#[string] base: Option<String>, #[string] id: String, #[string] url: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let body = openapi::models::NavigateRequest { url };
	openapi::apis::default_api::navigate_scraper_instance(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_navigate_tab(#[string] base: Option<String>, #[string] id: String, #[string] url: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let body = openapi::models::NavigateRequest { url };
	openapi::apis::default_api::navigate_tab_instance(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_scraper_html(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_scraper_instance_html(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_scraper_uri(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_scraper_instance_uri(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_tab_uri(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_tab_instance_uri(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_wait_for_element(#[string] base: Option<String>, #[string] id: String, #[string] selector: String, #[string] timeout_ms: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let timeout = timeout_ms.and_then(|s| s.parse::<i32>().ok());
	let body = openapi::models::WaitForElementRequest { selector: selector.clone(), timeout };
	openapi::apis::default_api::wait_for_scraper_element(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_click_element(#[string] base: Option<String>, #[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let body = openapi::models::SelectorRequest { selector };
	openapi::apis::default_api::click_scraper_element(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_element_text(#[string] base: Option<String>, #[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_scraper_element_text(&c, &id, &selector).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_element_value(#[string] base: Option<String>, #[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_scraper_element_value(&c, &id, &selector).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_fill_form(#[string] base: Option<String>, #[string] id: String, #[string] selector: String, #[string] value: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let mut map = std::collections::HashMap::new();
	map.insert(selector, value);
	let body = openapi::models::FillFormRequest { form_data: map };
	openapi::apis::default_api::fill_scraper_form(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_submit_form(#[string] base: Option<String>, #[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let body = openapi::models::SelectorRequest { selector };
	openapi::apis::default_api::submit_scraper_form(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_screenshot(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::take_scraper_screenshot(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_element_screenshot(#[string] base: Option<String>, #[string] id: String, #[string] selector: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::take_scraper_element_screenshot(&c, &id, &selector).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_fullpage_screenshot(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::take_scraper_full_page_screenshot(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_region_screenshot(#[string] base: Option<String>, #[string] id: String, #[string] x: Option<String>, #[string] y: Option<String>, #[string] w: Option<String>, #[string] h: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let mut rect = openapi::models::Rect::new();
	rect.x = x.and_then(|v| v.parse::<i32>().ok());
	rect.y = y.and_then(|v| v.parse::<i32>().ok());
	rect.width = w.and_then(|v| v.parse::<i32>().ok());
	rect.height = h.and_then(|v| v.parse::<i32>().ok());
	let body = openapi::models::RegionScreenshotRequest { rect: Some(Box::new(rect)) };
	openapi::apis::default_api::take_scraper_region_screenshot(&c, &id, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

// ---- Browser / Tab listing & context ----
#[op2(async)]
#[string]
async fn op_floorp_list_browser_tabs(#[string] base: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::list_browser_tabs(&c).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_browser_tabs(#[string] base: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::get_browser_tabs(&c).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_browser_history(#[string] base: Option<String>, #[string] limit: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let lim = limit.and_then(|v| v.parse::<i32>().ok());
	openapi::apis::default_api::get_browser_history(&c, lim).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_browser_downloads(#[string] base: Option<String>, #[string] limit: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let lim = limit.and_then(|v| v.parse::<i32>().ok());
	openapi::apis::default_api::get_browser_downloads(&c, lim).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_browser_context(#[string] base: Option<String>, #[string] history_limit: Option<String>, #[string] download_limit: Option<String>) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let h = history_limit.and_then(|v| v.parse::<i32>().ok());
	let d = download_limit.and_then(|v| v.parse::<i32>().ok());
	openapi::apis::default_api::get_browser_context(&c, h, d).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

// ---- Attach / Destroy / Exists ----
#[op2(async)]
#[string]
async fn op_floorp_attach_to_tab(#[string] base: Option<String>, #[string] browser_id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	let body = openapi::models::AttachRequest { browser_id };
	openapi::apis::default_api::attach_to_tab(&c, body).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_destroy_tab_instance(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::destroy_tab_instance(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_destroy_scraper_instance(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::destroy_scraper_instance(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_check_tab_instance_exists(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::check_tab_instance_exists(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
}

#[op2(async)]
#[string]
async fn op_floorp_check_scraper_instance_exists(#[string] base: Option<String>, #[string] id: String) -> Result<String, JsErrorBox> {
	let c = cfg(base, None);
	openapi::apis::default_api::check_scraper_instance_exists(&c, &id).await
		.map(|r| serde_json::to_string(&r).unwrap())
		.map_err(|e| JsErrorBox::new("Error", e.to_string()))
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
make_plugin!(floorp_create_scraper_instance_plugin, op_floorp_create_scraper_instance, "createScraperInstance", "Create Scraper Instance", "Creates a new scraper instance.");
make_plugin!(floorp_create_tab_instance_plugin, op_floorp_create_tab_instance, "createTabInstance", "Create Tab Instance", "Creates a new tab instance.");
make_plugin!(floorp_navigate_scraper_plugin, op_floorp_navigate_scraper, "navigateScraper", "Navigate Scraper", "Navigate a scraper instance to a URL.");
make_plugin!(floorp_navigate_tab_plugin, op_floorp_navigate_tab, "navigateTab", "Navigate Tab", "Navigate a tab instance to a URL.");
make_plugin!(floorp_scraper_html_plugin, op_floorp_scraper_html, "scraperHtml", "Scraper HTML", "Get current page HTML of scraper instance.");
make_plugin!(floorp_scraper_uri_plugin, op_floorp_scraper_uri, "scraperUri", "Scraper URI", "Get current URI of scraper instance.");
make_plugin!(floorp_tab_uri_plugin, op_floorp_tab_uri, "tabUri", "Tab URI", "Get current URI of tab instance.");
make_plugin!(floorp_wait_for_element_plugin, op_floorp_wait_for_element, "waitForElement", "Wait For Element", "Wait for an element by selector.");
make_plugin!(floorp_click_element_plugin, op_floorp_click_element, "clickElement", "Click Element", "Click an element by selector.");
make_plugin!(floorp_element_text_plugin, op_floorp_element_text, "elementText", "Element Text", "Get text content of element by selector.");
make_plugin!(floorp_element_value_plugin, op_floorp_element_value, "elementValue", "Element Value", "Get value of element by selector.");
make_plugin!(floorp_fill_form_plugin, op_floorp_fill_form, "fillForm", "Fill Form", "Fill a form element.");
make_plugin!(floorp_submit_form_plugin, op_floorp_submit_form, "submitForm", "Submit Form", "Submit a form element.");
make_plugin!(floorp_screenshot_plugin, op_floorp_screenshot, "screenshot", "Screenshot", "Take a screenshot of the page (PNG base64).");
make_plugin!(floorp_element_screenshot_plugin, op_floorp_element_screenshot, "elementScreenshot", "Element Screenshot", "Take a screenshot of an element (PNG base64).");
make_plugin!(floorp_fullpage_screenshot_plugin, op_floorp_fullpage_screenshot, "fullPageScreenshot", "Full Page Screenshot", "Take a full page screenshot (PNG base64).");
make_plugin!(floorp_region_screenshot_plugin, op_floorp_region_screenshot, "regionScreenshot", "Region Screenshot", "Take a region screenshot (PNG base64).");
make_plugin!(floorp_list_browser_tabs_plugin, op_floorp_list_browser_tabs, "listBrowserTabs", "List Browser Tabs", "List browser tabs (lightweight)." );
make_plugin!(floorp_browser_tabs_plugin, op_floorp_browser_tabs, "browserTabs", "Browser Tabs", "Get browser tabs (detailed)." );
make_plugin!(floorp_browser_history_plugin, op_floorp_browser_history, "browserHistory", "Browser History", "Get browser history list." );
make_plugin!(floorp_browser_downloads_plugin, op_floorp_browser_downloads, "browserDownloads", "Browser Downloads", "Get browser downloads list." );
make_plugin!(floorp_browser_context_plugin, op_floorp_browser_context, "browserContext", "Browser Context", "Get browser context (history/tabs/downloads)." );
make_plugin!(floorp_attach_to_tab_plugin, op_floorp_attach_to_tab, "attachToTab", "Attach To Tab", "Attach to an existing tab instance." );
make_plugin!(floorp_destroy_tab_instance_plugin, op_floorp_destroy_tab_instance, "destroyTabInstance", "Destroy Tab Instance", "Destroy a tab instance." );
make_plugin!(floorp_destroy_scraper_instance_plugin, op_floorp_destroy_scraper_instance, "destroyScraperInstance", "Destroy Scraper Instance", "Destroy a scraper instance." );
make_plugin!(floorp_check_tab_instance_exists_plugin, op_floorp_check_tab_instance_exists, "checkTabInstanceExists", "Check Tab Instance Exists", "Check if tab instance exists." );
make_plugin!(floorp_check_scraper_instance_exists_plugin, op_floorp_check_scraper_instance_exists, "checkScraperInstanceExists", "Check Scraper Instance Exists", "Check if scraper instance exists." );

