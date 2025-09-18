use deno_core::op2;
use deno_core::serde;
use deno_error::JsErrorBox;
use sapphillon_core::plugin::{CorePluginFunction, CorePluginPackage};

const DEFAULT_BASE: &str = "http://localhost:58261";

fn cfg(token: Option<String>) -> openapi::apis::configuration::Configuration {
	let mut c = openapi::apis::configuration::Configuration::new();
	c.base_path = DEFAULT_BASE.to_string();
	if let Some(t) = token { c.bearer_access_token = Some(t); }
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
		f()
			.map(|r| serde_json::to_string(&r).unwrap())
			.map_err(|e| JsErrorBox::new("Error", e.to_string()))
	})
	.join()
	.map_err(|_| JsErrorBox::new("Error", "thread panicked".to_string()))?
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
			// tab-specific plugins
			floorp_tab_html_plugin(),
			floorp_tab_screenshot_plugin(),
			floorp_tab_element_plugin(),
			floorp_tab_element_text_plugin(),
			floorp_tab_click_element_plugin(),
			floorp_tab_wait_for_element_plugin(),
			floorp_tab_execute_script_plugin(),
			floorp_tab_element_screenshot_plugin(),
			floorp_tab_fullpage_screenshot_plugin(),
			floorp_tab_region_screenshot_plugin(),
			floorp_tab_fill_form_plugin(),
			floorp_tab_element_value_plugin(),
			floorp_tab_submit_form_plugin(),
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
		openapi::apis::default_api::create_scraper_instance(&c)
			.map(|r| serde_json::json!({
				"instanceId": r.instance_id,
				"id": r.instance_id,
			}))
	})
}

#[op2]
#[string]
fn op_floorp_create_tab_instance(
	#[string] url: String,
	#[string] in_background: Option<String>,
) -> Result<String, JsErrorBox> {
	let mut body = openapi::models::CreateTabInstanceRequest { url, in_background: None };
	if let Some(b) = in_background {
		body.in_background = b.parse::<bool>().ok();
	}
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::create_tab_instance(&c, body)
			.map(|r| serde_json::json!({
				"instanceId": r.instance_id,
				"id": r.instance_id,
			}))
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
fn op_floorp_scraper_html(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::get_scraper_instance_html(&c, &id)
			.map(|r| {
				let html = r.html.and_then(|h| h).unwrap_or_default();
				serde_json::json!({ "html": html })
			})
	})
}

#[op2]
#[string]
fn op_floorp_scraper_uri(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::get_scraper_instance_uri(&c, &id)
			.map(|r| {
				let uri = r.uri.and_then(|u| u).unwrap_or_default();
				serde_json::json!({ "uri": uri })
			})
	})
}

#[op2]
#[string]
fn op_floorp_tab_uri(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
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
	let body = openapi::models::WaitForElementRequest { selector: selector.clone(), timeout };
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
		openapi::apis::default_api::get_scraper_element_text(&c, &id, &selector)
			.map(|r| {
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
		openapi::apis::default_api::get_scraper_element_value(&c, &id, &selector)
			.map(|r| {
				let value = r.value.and_then(|v| v).unwrap_or_default();
				serde_json::json!({ "value": value })
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
fn op_floorp_screenshot(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::take_scraper_screenshot(&c, &id)
			.map(|r| {
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
		openapi::apis::default_api::take_scraper_element_screenshot(&c, &id, &selector)
			.map(|r| {
				let image = r.image.and_then(|i| i).unwrap_or_default();
				serde_json::json!({ "image": image })
			})
	})
}

#[op2]
#[string]
fn op_floorp_fullpage_screenshot(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::take_scraper_full_page_screenshot(&c, &id)
			.map(|r| {
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
	let body = openapi::models::RegionScreenshotRequest { rect: Some(Box::new(rect)) };
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::take_scraper_region_screenshot(&c, &id, body)
			.map(|r| {
				let image = r.image.and_then(|i| i).unwrap_or_default();
				serde_json::json!({ "image": image })
			})
	})
}

// --- Tab ops implementations ---
#[op2]
#[string]
fn op_floorp_tab_html(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::get_tab_instance_html(&c, &id)
			.map(|r| {
				let html = r.html.and_then(|h| h).unwrap_or_default();
				serde_json::json!({ "html": html })
			})
	})
}

#[op2]
#[string]
fn op_floorp_tab_screenshot(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::take_tab_screenshot(&c, &id)
			.map(|r| {
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
		openapi::apis::default_api::get_tab_element_text(&c, &id, &selector)
			.map(|r| {
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
	let body = openapi::models::WaitForElementRequest { selector: selector.clone(), timeout };
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::wait_for_tab_element(&c, &id, body)
	})
}

#[op2]
#[string]
fn op_floorp_tab_execute_script(
	#[string] id: String,
	#[string] script: String,
) -> Result<String, JsErrorBox> {
	let body = openapi::models::ExecuteScriptRequest { script };
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::execute_tab_script(&c, &id, body)
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
		openapi::apis::default_api::take_tab_element_screenshot(&c, &id, &selector)
			.map(|r| {
				let image = r.image.and_then(|i| i).unwrap_or_default();
				serde_json::json!({ "image": image })
			})
	})
}

#[op2]
#[string]
fn op_floorp_tab_fullpage_screenshot(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::take_tab_full_page_screenshot(&c, &id)
			.map(|r| {
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
	let body = openapi::models::RegionScreenshotRequest { rect: Some(Box::new(rect)) };
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::take_tab_region_screenshot(&c, &id, body)
			.map(|r| {
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
fn op_floorp_tab_element_value(
	#[string] id: String,
	#[string] selector: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::get_tab_element_value(&c, &id, &selector)
			.map(|r| {
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
fn op_floorp_browser_history(
	#[string] limit: Option<String>,
) -> Result<String, JsErrorBox> {
	let lim = limit.and_then(|v| v.parse::<i32>().ok());
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::get_browser_history(&c, lim)
	})
}

#[op2]
#[string]
fn op_floorp_browser_downloads(
	#[string] limit: Option<String>,
) -> Result<String, JsErrorBox> {
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
fn op_floorp_attach_to_tab(
	#[string] browser_id: String,
) -> Result<String, JsErrorBox> {
	let body = openapi::models::AttachRequest { browser_id };
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::attach_to_tab(&c, body)
	})
}

#[op2]
#[string]
fn op_floorp_destroy_tab_instance(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::destroy_tab_instance(&c, &id)
	})
}

#[op2]
#[string]
fn op_floorp_destroy_scraper_instance(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::destroy_scraper_instance(&c, &id)
	})
}

#[op2]
#[string]
fn op_floorp_check_tab_instance_exists(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
	run_blocking_json(move || {
		let c = cfg(None);
		openapi::apis::default_api::check_tab_instance_exists(&c, &id)
	})
}

#[op2]
#[string]
fn op_floorp_check_scraper_instance_exists(
	#[string] id: String,
) -> Result<String, JsErrorBox> {
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
make_plugin!(floorp_create_scraper_instance_plugin, op_floorp_create_scraper_instance, "createScraperInstance", "Create Scraper Instance", "Creates a new scraper instance.");
make_plugin!(floorp_create_tab_instance_plugin, op_floorp_create_tab_instance, "createTabInstance", "Create Tab Instance", "Creates a new tab instance.");
make_plugin!(floorp_navigate_scraper_plugin, op_floorp_navigate_scraper, "navigateScraper", "Navigate Scraper", "Navigate a scraper instance to a URL.");
make_plugin!(floorp_navigate_tab_plugin, op_floorp_navigate_tab, "navigateTab", "Navigate Tab", "Navigate a tab instance to a URL.");
make_plugin!(floorp_scraper_html_plugin, op_floorp_scraper_html, "scraperHtml", "Scraper HTML", "Get current page HTML of scraper instance.");
make_plugin!(floorp_tab_html_plugin, op_floorp_tab_html, "tabHtml", "Tab HTML", "Get current page HTML of tab instance.");
make_plugin!(floorp_tab_screenshot_plugin, op_floorp_tab_screenshot, "tabScreenshot", "Tab Screenshot", "Take a screenshot of the tab page (PNG base64)." );
make_plugin!(floorp_tab_element_plugin, op_floorp_tab_element, "tabElement", "Tab Element", "Get element information from tab by selector.");
make_plugin!(floorp_tab_element_text_plugin, op_floorp_tab_element_text, "tabElementText", "Tab Element Text", "Get text content of element in tab by selector.");
make_plugin!(floorp_tab_click_element_plugin, op_floorp_tab_click_element, "tabClickElement", "Tab Click Element", "Click an element in tab by selector.");
make_plugin!(floorp_tab_wait_for_element_plugin, op_floorp_tab_wait_for_element, "tabWaitForElement", "Tab Wait For Element", "Wait for an element in tab by selector.");
make_plugin!(floorp_tab_execute_script_plugin, op_floorp_tab_execute_script, "tabExecuteScript", "Tab Execute Script", "Execute JS in tab.");
make_plugin!(floorp_tab_element_screenshot_plugin, op_floorp_tab_element_screenshot, "tabElementScreenshot", "Tab Element Screenshot", "Take a screenshot of an element in tab (PNG base64).");
make_plugin!(floorp_tab_fullpage_screenshot_plugin, op_floorp_tab_fullpage_screenshot, "tabFullPageScreenshot", "Tab Full Page Screenshot", "Take a full page screenshot of tab (PNG base64).");
make_plugin!(floorp_tab_region_screenshot_plugin, op_floorp_tab_region_screenshot, "tabRegionScreenshot", "Tab Region Screenshot", "Take a region screenshot of tab (PNG base64).");
make_plugin!(floorp_tab_fill_form_plugin, op_floorp_tab_fill_form, "tabFillForm", "Tab Fill Form", "Fill a form in tab.");
make_plugin!(floorp_tab_element_value_plugin, op_floorp_tab_element_value, "tabElementValue", "Tab Element Value", "Get element value in tab by selector.");
make_plugin!(floorp_tab_submit_form_plugin, op_floorp_tab_submit_form, "tabSubmitForm", "Tab Submit Form", "Submit a form element in tab.");
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
