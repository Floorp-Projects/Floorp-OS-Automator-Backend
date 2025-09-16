// JavaScript glue exposing Floorp OS API operations to workflows
function floorpHealth() {
  return Deno.core.ops.op_floorp_health();
}
function floorpCreateScraper() {
  return Deno.core.ops.op_floorp_create_scraper_instance();
}
function floorpNavigate(id, url) {
  return Deno.core.ops.op_floorp_navigate_scraper(id, url);
}
function floorpScraperHtml(id) {
  return Deno.core.ops.op_floorp_scraper_html(id);
}
function floorpScraperUri(id) {
  return Deno.core.ops.op_floorp_scraper_uri(id);
}
function floorpWaitForElement(id, selector, timeoutMs) {
  return Deno.core.ops.op_floorp_wait_for_element(
    id,
    selector,
    timeoutMs?.toString()
  );
}
function floorpClick(id, selector) {
  return Deno.core.ops.op_floorp_click_element(id, selector);
}
function floorpElementText(id, selector) {
  return Deno.core.ops.op_floorp_element_text(id, selector);
}
function floorpElementValue(id, selector) {
  return Deno.core.ops.op_floorp_element_value(id, selector);
}
function floorpFillForm(id, selector, value) {
  return Deno.core.ops.op_floorp_fill_form(id, selector, value);
}
function floorpSubmitForm(id, selector) {
  return Deno.core.ops.op_floorp_submit_form(id, selector);
}
function floorpScreenshot(id) {
  return Deno.core.ops.op_floorp_screenshot(id);
}
function floorpElementScreenshot(id, selector) {
  return Deno.core.ops.op_floorp_element_screenshot(id, selector);
}
function floorpFullPageScreenshot(id) {
  return Deno.core.ops.op_floorp_fullpage_screenshot(id);
}
function floorpRegionScreenshot(id, x, y, w, h) {
  return Deno.core.ops.op_floorp_region_screenshot(
    id,
    x?.toString(),
    y?.toString(),
    w?.toString(),
h?.toString()
  );
}
function floorpCreateTab(url, inBackground) {
  return Deno.core.ops.op_floorp_create_tab_instance(
    url,
    inBackground?.toString()
  );
}
function floorpNavigateTab(id, url) {
  return Deno.core.ops.op_floorp_navigate_tab(id, url);
}
function floorpTabUri(id) {
  return Deno.core.ops.op_floorp_tab_uri(id);
}
function floorpListBrowserTabs() {
  return Deno.core.ops.op_floorp_list_browser_tabs();
}
function floorpBrowserTabs() {
  return Deno.core.ops.op_floorp_browser_tabs();
}
function floorpBrowserHistory(limit) {
  return Deno.core.ops.op_floorp_browser_history(limit?.toString());
}
function floorpBrowserDownloads(limit) {
  return Deno.core.ops.op_floorp_browser_downloads(limit?.toString());
}
function floorpBrowserContext(historyLimit, downloadLimit) {
  return Deno.core.ops.op_floorp_browser_context(
    historyLimit?.toString(),
    downloadLimit?.toString()
  );
}
function floorpAttachToTab(instanceId) {
  return Deno.core.ops.op_floorp_attach_to_tab(instanceId);
}
function floorpDestroyTabInstance(id) {
  return Deno.core.ops.op_floorp_destroy_tab_instance(id);
}
function floorpDestroyScraperInstance(id) {
  return Deno.core.ops.op_floorp_destroy_scraper_instance(id);
}
function floorpCheckTabInstanceExists(id) {
  return Deno.core.ops.op_floorp_check_tab_instance_exists(id);
}
function floorpCheckScraperInstanceExists(id) {
  return Deno.core.ops.op_floorp_check_scraper_instance_exists(id);
}

globalThis.floorp = {
  health: floorpHealth,
  createScraper: floorpCreateScraper,
  navigate: floorpNavigate,
  html: floorpScraperHtml,
  uri: floorpScraperUri,
  waitForElement: floorpWaitForElement,
  click: floorpClick,
  text: floorpElementText,
  value: floorpElementValue,
  fillForm: floorpFillForm,
  submitForm: floorpSubmitForm,
  screenshot: floorpScreenshot,
  elementScreenshot: floorpElementScreenshot,
  fullPageScreenshot: floorpFullPageScreenshot,
  regionScreenshot: floorpRegionScreenshot,
  createTab: floorpCreateTab,
  navigateTab: floorpNavigateTab,
  tabUri: floorpTabUri,
  listBrowserTabs: floorpListBrowserTabs,
  browserTabs: floorpBrowserTabs,
  browserHistory: floorpBrowserHistory,
  browserDownloads: floorpBrowserDownloads,
  browserContext: floorpBrowserContext,
  attachToTab: floorpAttachToTab,
  destroyTabInstance: floorpDestroyTabInstance,
  destroyScraperInstance: floorpDestroyScraperInstance,
  checkTabInstanceExists: floorpCheckTabInstanceExists,
  checkScraperInstanceExists: floorpCheckScraperInstanceExists,
};
