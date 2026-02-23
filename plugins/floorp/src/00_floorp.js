// @ts-nocheck @reason
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
function floorpGetElements(id, selector) {
  return Deno.core.ops.op_floorp_get_elements(id, selector);
}
function floorpTabGetElements(id, selector) {
  return Deno.core.ops.op_floorp_tab_get_elements(id, selector);
}
// Input / Press Key / Upload File functions
function floorpInput(id, selector, value) {
  return Deno.core.ops.op_floorp_input(id, selector, value);
}
function floorpTabInput(id, selector, value, typingMode) {
  var result = Deno.core.ops.op_floorp_tab_input(
    id,
    selector,
    value,
    typingMode === true
  );
  // Parse result and convert null to false for consistency
  try {
    var parsed = JSON.parse(result);
    return parsed.ok === true ? "true" : "false";
  } catch (e) {
    return "false";
  }
}
function floorpPressKey(id, key) {
  return Deno.core.ops.op_floorp_press_key(id, key);
}
function floorpTabPressKey(id, key) {
  return Deno.core.ops.op_floorp_tab_press_key(id, key);
}
function floorpUploadFile(id, selector, filePath) {
  return Deno.core.ops.op_floorp_upload_file(id, selector, filePath);
}
function floorpTabUploadFile(id, selector, filePath) {
  return Deno.core.ops.op_floorp_tab_upload_file(id, selector, filePath);
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
function floorpElementByText(id, text) {
  return Deno.core.ops.op_floorp_element_by_text(id, text);
}
function floorpElementTextContent(id, selector) {
  return Deno.core.ops.op_floorp_element_text_content(id, selector);
}
function floorpTabElementByText(id, text) {
  return Deno.core.ops.op_floorp_tab_element_by_text(id, text);
}
function floorpTabElementTextContent(id, selector) {
  return Deno.core.ops.op_floorp_tab_element_text_content(id, selector);
}
function floorpFillForm(id, selector, value) {
  return Deno.core.ops.op_floorp_fill_form(id, selector, value);
}
function floorpSubmitForm(id, selector) {
  return Deno.core.ops.op_floorp_submit_form(id, selector);
}
function floorpClearInput(id, selector) {
  return Deno.core.ops.op_floorp_clear_input(id, selector);
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
  var result = Deno.core.ops.op_floorp_create_tab_instance(
    url,
    inBackground?.toString()
  );
  // Parse JSON and return instanceId directly for consistency
  try {
    var parsed = JSON.parse(result);
    return parsed.instanceId || result;
  } catch (e) {
    return result;
  }
}
function floorpNavigateTab(id, url) {
  return Deno.core.ops.op_floorp_navigate_tab(id, url);
}
function floorpTabUri(id) {
  return Deno.core.ops.op_floorp_tab_uri(id);
}
function floorpTabHtml(id) {
  return Deno.core.ops.op_floorp_tab_html(id);
}
function floorpTabScreenshot(id) {
  return Deno.core.ops.op_floorp_tab_screenshot(id);
}
function floorpTabElement(id, selector) {
  return Deno.core.ops.op_floorp_tab_element(id, selector);
}
function floorpTabElementText(id, selector) {
  return Deno.core.ops.op_floorp_tab_element_text(id, selector);
}
function floorpTabClick(id, selector) {
  var result = Deno.core.ops.op_floorp_tab_click_element(id, selector);
  // Parse result and convert null to false for consistency
  try {
    var parsed = JSON.parse(result);
    return parsed.ok === true ? "true" : "false";
  } catch (e) {
    return "false";
  }
}
function floorpTabWaitForElement(id, selector, timeoutMs) {
  var result = Deno.core.ops.op_floorp_tab_wait_for_element(
    id,
    selector,
    timeoutMs?.toString()
  );
  // Parse result and convert null to false for consistency
  try {
    var parsed = JSON.parse(result);
    return parsed.ok === true ? "true" : "false";
  } catch (e) {
    return "false";
  }
}
function floorpTabClearInput(id, selector) {
  return Deno.core.ops.op_floorp_tab_clear_input(id, selector);
}
function floorpTabElementScreenshot(id, selector) {
  return Deno.core.ops.op_floorp_tab_element_screenshot(id, selector);
}
function floorpTabFullPageScreenshot(id) {
  return Deno.core.ops.op_floorp_tab_fullpage_screenshot(id);
}
function floorpTabRegionScreenshot(id, x, y, w, h) {
  return Deno.core.ops.op_floorp_tab_region_screenshot(
    id,
    x?.toString(),
    y?.toString(),
    w?.toString(),
    h?.toString()
  );
}
function floorpTabFillForm(id, selector, value) {
  return Deno.core.ops.op_floorp_tab_fill_form(id, selector, value);
}
function floorpTabSetInnerHTML(id, selector, html) {
  return Deno.core.ops.op_floorp_tab_set_inner_html(id, selector, html);
}
function floorpTabSetTextContent(id, selector, text) {
  return Deno.core.ops.op_floorp_tab_set_text_content(id, selector, text);
}
function floorpTabDispatchEvent(id, selector, eventType) {
  return Deno.core.ops.op_floorp_tab_dispatch_event(id, selector, eventType);
}
function floorpTabElementValue(id, selector) {
  return Deno.core.ops.op_floorp_tab_element_value(id, selector);
}
function floorpTabSubmitForm(id, selector) {
  return Deno.core.ops.op_floorp_tab_submit_form(id, selector);
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
function floorpAttachToTab(browserId) {
  var result = Deno.core.ops.op_floorp_attach_to_tab(browserId);
  // Parse JSON and return instanceId directly for consistency
  try {
    var parsed = JSON.parse(result);
    return parsed.instanceId || null;
  } catch (e) {
    return null;
  }
}
function floorpDestroyTabInstance(id) {
  return Deno.core.ops.op_floorp_destroy_tab_instance(id);
}
function floorpDestroyScraperInstance(id) {
  return Deno.core.ops.op_floorp_destroy_scraper_instance(id);
}
function floorpCloseTab(id) {
  return Deno.core.ops.op_floorp_close_tab(id);
}
function floorpCheckTabInstanceExists(id) {
  return Deno.core.ops.op_floorp_check_tab_instance_exists(id);
}
function floorpCheckScraperInstanceExists(id) {
  return Deno.core.ops.op_floorp_check_scraper_instance_exists(id);
}

function floorpAttribute(id, selector, name) {
  return Deno.core.ops.op_floorp_attribute(id, selector, name);
}
function floorpIsVisible(id, selector) {
  return Deno.core.ops.op_floorp_is_visible(id, selector);
}
function floorpIsEnabled(id, selector) {
  return Deno.core.ops.op_floorp_is_enabled(id, selector);
}
function floorpSelectOption(id, selector, value) {
  return Deno.core.ops.op_floorp_select_option(id, selector, value);
}
function floorpSetChecked(id, selector, checked) {
  return Deno.core.ops.op_floorp_set_checked(id, selector, checked?.toString());
}
function floorpHover(id, selector) {
  return Deno.core.ops.op_floorp_hover(id, selector);
}
function floorpScrollTo(id, selector) {
  return Deno.core.ops.op_floorp_scroll_to(id, selector);
}
function floorpTitle(id) {
  return Deno.core.ops.op_floorp_title(id);
}
function floorpDoubleClick(id, selector) {
  return Deno.core.ops.op_floorp_double_click(id, selector);
}
function floorpRightClick(id, selector) {
  return Deno.core.ops.op_floorp_right_click(id, selector);
}
function floorpFocus(id, selector) {
  return Deno.core.ops.op_floorp_focus(id, selector);
}
function floorpDragAndDrop(id, source, target) {
  return Deno.core.ops.op_floorp_drag_and_drop(id, source, target);
}
function floorpCookies(id) {
  return Deno.core.ops.op_floorp_cookies(id);
}
function floorpSetCookie(id, cookie) {
  return Deno.core.ops.op_floorp_set_cookie(id, JSON.stringify(cookie));
}
function floorpAcceptAlert(id) {
  return Deno.core.ops.op_floorp_accept_alert(id);
}
function floorpDismissAlert(id) {
  return Deno.core.ops.op_floorp_dismiss_alert(id);
}
function floorpPdf(id) {
  return Deno.core.ops.op_floorp_pdf(id);
}
function floorpWaitForNetworkIdle(id, timeoutMs) {
  return Deno.core.ops.op_floorp_wait_for_network_idle(
    id,
    timeoutMs?.toString()
  );
}

function floorpTabAttribute(id, selector, name) {
  return Deno.core.ops.op_floorp_tab_attribute(id, selector, name);
}
function floorpTabIsVisible(id, selector) {
  return Deno.core.ops.op_floorp_tab_is_visible(id, selector);
}
function floorpTabIsEnabled(id, selector) {
  return Deno.core.ops.op_floorp_tab_is_enabled(id, selector);
}
function floorpTabSelectOption(id, selector, value) {
  return Deno.core.ops.op_floorp_tab_select_option(id, selector, value);
}
function floorpTabSetChecked(id, selector, checked) {
  return Deno.core.ops.op_floorp_tab_set_checked(
    id,
    selector,
    checked?.toString()
  );
}
function floorpTabHover(id, selector) {
  return Deno.core.ops.op_floorp_tab_hover(id, selector);
}
function floorpTabScrollTo(id, selector) {
  return Deno.core.ops.op_floorp_tab_scroll_to(id, selector);
}
function floorpTabTitle(id) {
  return Deno.core.ops.op_floorp_tab_title(id);
}
function floorpTabDoubleClick(id, selector) {
  return Deno.core.ops.op_floorp_tab_double_click(id, selector);
}
function floorpTabRightClick(id, selector) {
  return Deno.core.ops.op_floorp_tab_right_click(id, selector);
}
function floorpTabFocus(id, selector) {
  return Deno.core.ops.op_floorp_tab_focus(id, selector);
}
function floorpTabDragAndDrop(id, source, target) {
  return Deno.core.ops.op_floorp_tab_drag_and_drop(id, source, target);
}
function floorpTabCookies(id) {
  return Deno.core.ops.op_floorp_tab_cookies(id);
}
function floorpTabSetCookie(id, cookie) {
  return Deno.core.ops.op_floorp_tab_set_cookie(id, JSON.stringify(cookie));
}
function floorpTabAcceptAlert(id) {
  return Deno.core.ops.op_floorp_tab_accept_alert(id);
}
function floorpTabDismissAlert(id) {
  return Deno.core.ops.op_floorp_tab_dismiss_alert(id);
}
function floorpTabPdf(id) {
  return Deno.core.ops.op_floorp_tab_pdf(id);
}
function floorpTabWaitForNetworkIdle(id, timeoutMs) {
  var result = Deno.core.ops.op_floorp_tab_wait_for_network_idle(
    id,
    timeoutMs?.toString()
  );
  // Parse result and convert null to false for consistency
  try {
    var parsed = JSON.parse(result);
    return parsed.ok === true ? "true" : "false";
  } catch (e) {
    return "false";
  }
}
function floorpListWorkspaces() {
  return Deno.core.ops.op_floorp_list_workspaces();
}
function floorpGetCurrentWorkspace() {
  return Deno.core.ops.op_floorp_get_current_workspace();
}
function floorpSwitchToNextWorkspace() {
  return Deno.core.ops.op_floorp_switch_to_next_workspace();
}
function floorpSwitchToPreviousWorkspace() {
  return Deno.core.ops.op_floorp_switch_to_previous_workspace();
}
function floorpSwitchToWorkspace(workspaceId) {
  return Deno.core.ops.op_floorp_switch_to_workspace(workspaceId);
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
  elementByText: floorpElementByText,
  elementTextContent: floorpElementTextContent,
  tabElementByText: floorpTabElementByText,
  tabElementTextContent: floorpTabElementTextContent,
  fillForm: floorpFillForm,
  submitForm: floorpSubmitForm,
  clearInput: floorpClearInput,
  screenshot: floorpScreenshot,
  elementScreenshot: floorpElementScreenshot,
  fullPageScreenshot: floorpFullPageScreenshot,
  regionScreenshot: floorpRegionScreenshot,
  createTab: floorpCreateTab,
  navigateTab: floorpNavigateTab,
  tabUri: floorpTabUri,
  tabHtml: floorpTabHtml,
  tabScreenshot: floorpTabScreenshot,
  tabElement: floorpTabElement,
  tabElementText: floorpTabElementText,
  tabClick: floorpTabClick,
  tabWaitForElement: floorpTabWaitForElement,
  tabElementScreenshot: floorpTabElementScreenshot,
  tabFullPageScreenshot: floorpTabFullPageScreenshot,
  tabRegionScreenshot: floorpTabRegionScreenshot,
  tabFillForm: floorpTabFillForm,
  tabSetInnerHTML: floorpTabSetInnerHTML,
  tabSetTextContent: floorpTabSetTextContent,
  tabDispatchEvent: floorpTabDispatchEvent,
  tabElementValue: floorpTabElementValue,
  tabSubmitForm: floorpTabSubmitForm,
  tabClearInput: floorpTabClearInput,
  listBrowserTabs: floorpListBrowserTabs,
  browserTabs: floorpBrowserTabs,
  browserHistory: floorpBrowserHistory,
  browserDownloads: floorpBrowserDownloads,
  browserContext: floorpBrowserContext,
  attachToTab: floorpAttachToTab,
  destroyTabInstance: floorpDestroyTabInstance,
  destroyScraperInstance: floorpDestroyScraperInstance,
  closeTab: floorpCloseTab,
  checkTabInstanceExists: floorpCheckTabInstanceExists,
  checkScraperInstanceExists: floorpCheckScraperInstanceExists,
  // New Scraper functions
  attribute: floorpAttribute,
  isVisible: floorpIsVisible,
  isEnabled: floorpIsEnabled,
  selectOption: floorpSelectOption,
  setChecked: floorpSetChecked,
  hover: floorpHover,
  scrollTo: floorpScrollTo,
  title: floorpTitle,
  doubleClick: floorpDoubleClick,
  rightClick: floorpRightClick,
  focus: floorpFocus,
  dragAndDrop: floorpDragAndDrop,
  cookies: floorpCookies,
  setCookie: floorpSetCookie,
  acceptAlert: floorpAcceptAlert,
  dismissAlert: floorpDismissAlert,
  pdf: floorpPdf,
  waitForNetworkIdle: floorpWaitForNetworkIdle,
  getElements: floorpGetElements,
  tabGetElements: floorpTabGetElements,
  // New Tab functions
  tabAttribute: floorpTabAttribute,
  tabIsVisible: floorpTabIsVisible,
  tabIsEnabled: floorpTabIsEnabled,
  tabSelectOption: floorpTabSelectOption,
  tabSetChecked: floorpTabSetChecked,
  tabHover: floorpTabHover,
  tabScrollTo: floorpTabScrollTo,
  tabTitle: floorpTabTitle,
  tabDoubleClick: floorpTabDoubleClick,
  tabRightClick: floorpTabRightClick,
  tabFocus: floorpTabFocus,
  tabDragAndDrop: floorpTabDragAndDrop,
  tabCookies: floorpTabCookies,
  tabSetCookie: floorpTabSetCookie,
  tabAcceptAlert: floorpTabAcceptAlert,
  tabDismissAlert: floorpTabDismissAlert,
  tabPdf: floorpTabPdf,
  tabWaitForNetworkIdle: floorpTabWaitForNetworkIdle,
  // Workspace functions
  listWorkspaces: floorpListWorkspaces,
  getCurrentWorkspace: floorpGetCurrentWorkspace,
  switchToNextWorkspace: floorpSwitchToNextWorkspace,
  switchToPreviousWorkspace: floorpSwitchToPreviousWorkspace,
  switchToWorkspace: floorpSwitchToWorkspace,
  // Input / Press Key / Upload File functions
  input: floorpInput,
  tabInput: floorpTabInput,
  pressKey: floorpPressKey,
  tabPressKey: floorpTabPressKey,
  uploadFile: floorpUploadFile,
  tabUploadFile: floorpTabUploadFile,
  // Screenshot functions
  tabElementScreenshot: floorpTabElementScreenshot,
  tabFullPageScreenshot: floorpTabFullPageScreenshot,
  tabRegionScreenshot: floorpTabRegionScreenshot,
};
