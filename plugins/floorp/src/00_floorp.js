// JavaScript glue exposing Floorp OS API operations to workflows
function floorpHealth(base) {
  return Deno.core.ops.op_floorp_health(base);
}
function floorpCreateScraper(base) {
  return Deno.core.ops.op_floorp_create_scraper_instance(base);
}
function floorpNavigate(base, id, url) {
  return Deno.core.ops.op_floorp_navigate_scraper(base, id, url);
}
function floorpScraperHtml(base, id) {
  return Deno.core.ops.op_floorp_scraper_html(base, id);
}
function floorpScraperUri(base, id) {
  return Deno.core.ops.op_floorp_scraper_uri(base, id);
}
function floorpWaitForElement(base, id, selector, timeoutMs) {
  return Deno.core.ops.op_floorp_wait_for_element(
    base,
    id,
    selector,
    timeoutMs?.toString()
  );
}
function floorpClick(base, id, selector) {
  return Deno.core.ops.op_floorp_click_element(base, id, selector);
}
function floorpElementText(base, id, selector) {
  return Deno.core.ops.op_floorp_element_text(base, id, selector);
}
function floorpElementValue(base, id, selector) {
  return Deno.core.ops.op_floorp_element_value(base, id, selector);
}
function floorpFillForm(base, id, selector, value) {
  return Deno.core.ops.op_floorp_fill_form(base, id, selector, value);
}
function floorpSubmitForm(base, id, selector) {
  return Deno.core.ops.op_floorp_submit_form(base, id, selector);
}
function floorpScreenshot(base, id) {
  return Deno.core.ops.op_floorp_screenshot(base, id);
}
function floorpElementScreenshot(base, id, selector) {
  return Deno.core.ops.op_floorp_element_screenshot(base, id, selector);
}
function floorpFullPageScreenshot(base, id) {
  return Deno.core.ops.op_floorp_fullpage_screenshot(base, id);
}
function floorpRegionScreenshot(base, id, x, y, w, h) {
  return Deno.core.ops.op_floorp_region_screenshot(
    base,
    id,
    x?.toString(),
    y?.toString(),
    w?.toString(),
    h?.toString()
  );
}
function floorpCreateTab(base, url, inBackground) {
  return Deno.core.ops.op_floorp_create_tab_instance(
    base,
    url,
    inBackground?.toString()
  );
}
function floorpNavigateTab(base, id, url) {
  return Deno.core.ops.op_floorp_navigate_tab(base, id, url);
}
function floorpTabUri(base, id) {
  return Deno.core.ops.op_floorp_tab_uri(base, id);
}
function floorpListBrowserTabs(base) {
  return Deno.core.ops.op_floorp_list_browser_tabs(base);
}
function floorpBrowserTabs(base) {
  return Deno.core.ops.op_floorp_browser_tabs(base);
}
function floorpBrowserHistory(base, limit) {
  return Deno.core.ops.op_floorp_browser_history(base, limit?.toString());
}
function floorpBrowserDownloads(base, limit) {
  return Deno.core.ops.op_floorp_browser_downloads(base, limit?.toString());
}
function floorpBrowserContext(base, historyLimit, downloadLimit) {
  return Deno.core.ops.op_floorp_browser_context(
    base,
    historyLimit?.toString(),
    downloadLimit?.toString()
  );
}
function floorpAttachToTab(base, instanceId) {
  return Deno.core.ops.op_floorp_attach_to_tab(base, instanceId);
}
function floorpDestroyTabInstance(base, id) {
  return Deno.core.ops.op_floorp_destroy_tab_instance(base, id);
}
function floorpDestroyScraperInstance(base, id) {
  return Deno.core.ops.op_floorp_destroy_scraper_instance(base, id);
}
function floorpCheckTabInstanceExists(base, id) {
  return Deno.core.ops.op_floorp_check_tab_instance_exists(base, id);
}
function floorpCheckScraperInstanceExists(base, id) {
  return Deno.core.ops.op_floorp_check_scraper_instance_exists(base, id);
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
