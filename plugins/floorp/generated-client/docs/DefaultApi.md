# \DefaultApi

All URIs are relative to *http://127.0.0.1:58261*

Method | HTTP request | Description
------------- | ------------- | -------------
[**accept_scraper_alert**](DefaultApi.md#accept_scraper_alert) | **POST** /scraper/instances/{id}/acceptAlert | Accept alert
[**accept_tab_alert**](DefaultApi.md#accept_tab_alert) | **POST** /tabs/instances/{id}/acceptAlert | Accept alert
[**attach_to_tab**](DefaultApi.md#attach_to_tab) | **POST** /tabs/attach | Attach to existing browser tab
[**check_scraper_instance_exists**](DefaultApi.md#check_scraper_instance_exists) | **GET** /scraper/instances/{id}/exists | Check if scraper instance exists
[**check_tab_instance_exists**](DefaultApi.md#check_tab_instance_exists) | **GET** /tabs/instances/{id}/exists | Check if tab instance exists
[**clear_scraper_input**](DefaultApi.md#clear_scraper_input) | **POST** /scraper/instances/{id}/clearInput | Clear input field in scraper instance
[**clear_tab_input**](DefaultApi.md#clear_tab_input) | **POST** /tabs/instances/{id}/clearInput | Clear input field in tab instance
[**click_scraper_element**](DefaultApi.md#click_scraper_element) | **POST** /scraper/instances/{id}/click | Click an element in scraper instance
[**click_tab_element**](DefaultApi.md#click_tab_element) | **POST** /tabs/instances/{id}/click | Click an element in tab instance
[**close_tab**](DefaultApi.md#close_tab) | **POST** /tabs/instances/{id}/close | Close tab (destroy instance and close browser tab)
[**create_scraper_instance**](DefaultApi.md#create_scraper_instance) | **POST** /scraper/instances | Create a new scraper instance
[**create_tab_instance**](DefaultApi.md#create_tab_instance) | **POST** /tabs/instances | Create a new tab instance
[**destroy_scraper_instance**](DefaultApi.md#destroy_scraper_instance) | **DELETE** /scraper/instances/{id} | Destroy a scraper instance
[**destroy_tab_instance**](DefaultApi.md#destroy_tab_instance) | **DELETE** /tabs/instances/{id} | Destroy tab instance
[**dismiss_scraper_alert**](DefaultApi.md#dismiss_scraper_alert) | **POST** /scraper/instances/{id}/dismissAlert | Dismiss alert
[**dismiss_tab_alert**](DefaultApi.md#dismiss_tab_alert) | **POST** /tabs/instances/{id}/dismissAlert | Dismiss alert
[**dispatch_scraper_event**](DefaultApi.md#dispatch_scraper_event) | **POST** /scraper/instances/{id}/dispatchEvent | Dispatch event on element
[**dispatch_tab_event**](DefaultApi.md#dispatch_tab_event) | **POST** /tabs/instances/{id}/dispatchEvent | Dispatch event on element in tab
[**double_click_scraper_element**](DefaultApi.md#double_click_scraper_element) | **POST** /scraper/instances/{id}/doubleClick | Double click element
[**double_click_tab_element**](DefaultApi.md#double_click_tab_element) | **POST** /tabs/instances/{id}/doubleClick | Double click element
[**drag_and_drop_scraper_element**](DefaultApi.md#drag_and_drop_scraper_element) | **POST** /scraper/instances/{id}/dragAndDrop | Drag and drop element
[**drag_and_drop_tab_element**](DefaultApi.md#drag_and_drop_tab_element) | **POST** /tabs/instances/{id}/dragAndDrop | Drag and drop element
[**fill_scraper_form**](DefaultApi.md#fill_scraper_form) | **POST** /scraper/instances/{id}/fillForm | Fill form fields in scraper instance
[**fill_tab_form**](DefaultApi.md#fill_tab_form) | **POST** /tabs/instances/{id}/fillForm | Fill form fields in tab instance
[**focus_scraper_element**](DefaultApi.md#focus_scraper_element) | **POST** /scraper/instances/{id}/focus | Focus element
[**focus_tab_element**](DefaultApi.md#focus_tab_element) | **POST** /tabs/instances/{id}/focus | Focus element
[**get_browser_context**](DefaultApi.md#get_browser_context) | **GET** /browser/context | Get combined browser context (history, tabs, downloads)
[**get_browser_downloads**](DefaultApi.md#get_browser_downloads) | **GET** /browser/downloads | Get recent browser downloads
[**get_browser_history**](DefaultApi.md#get_browser_history) | **GET** /browser/history | Get recent browser history
[**get_browser_tabs**](DefaultApi.md#get_browser_tabs) | **GET** /browser/tabs | Get recent browser tabs
[**get_current_workspace**](DefaultApi.md#get_current_workspace) | **GET** /workspaces/current | Get current workspace ID
[**get_health**](DefaultApi.md#get_health) | **GET** /health | Health check endpoint
[**get_scraper_attribute**](DefaultApi.md#get_scraper_attribute) | **GET** /scraper/instances/{id}/attribute | Get element attribute
[**get_scraper_cookies**](DefaultApi.md#get_scraper_cookies) | **GET** /scraper/instances/{id}/cookies | Get cookies
[**get_scraper_element_by_text**](DefaultApi.md#get_scraper_element_by_text) | **GET** /scraper/instances/{id}/elementByText | Find first element containing text
[**get_scraper_element_text**](DefaultApi.md#get_scraper_element_text) | **GET** /scraper/instances/{id}/elementText | Get text content of element
[**get_scraper_element_text_content**](DefaultApi.md#get_scraper_element_text_content) | **GET** /scraper/instances/{id}/elementTextContent | Get trimmed text content for selector
[**get_scraper_element_value**](DefaultApi.md#get_scraper_element_value) | **GET** /scraper/instances/{id}/value | Get value of form element
[**get_scraper_elements**](DefaultApi.md#get_scraper_elements) | **GET** /scraper/instances/{id}/elements | Get all matching elements' outerHTML
[**get_scraper_instance_html**](DefaultApi.md#get_scraper_instance_html) | **GET** /scraper/instances/{id}/html | Get HTML content of scraper instance
[**get_scraper_instance_uri**](DefaultApi.md#get_scraper_instance_uri) | **GET** /scraper/instances/{id}/uri | Get current URI of scraper instance
[**get_scraper_title**](DefaultApi.md#get_scraper_title) | **GET** /scraper/instances/{id}/title | Get page title
[**get_tab_attribute**](DefaultApi.md#get_tab_attribute) | **GET** /tabs/instances/{id}/attribute | Get element attribute
[**get_tab_cookies**](DefaultApi.md#get_tab_cookies) | **GET** /tabs/instances/{id}/cookies | Get cookies
[**get_tab_element**](DefaultApi.md#get_tab_element) | **GET** /tabs/instances/{id}/element | Get element HTML for selector in tab instance
[**get_tab_element_by_text**](DefaultApi.md#get_tab_element_by_text) | **GET** /tabs/instances/{id}/elementByText | Find first element containing text in tab
[**get_tab_element_text**](DefaultApi.md#get_tab_element_text) | **GET** /tabs/instances/{id}/elementText | Get text content of element in tab instance
[**get_tab_element_text_content**](DefaultApi.md#get_tab_element_text_content) | **GET** /tabs/instances/{id}/elementTextContent | Get trimmed text content for selector in tab
[**get_tab_element_value**](DefaultApi.md#get_tab_element_value) | **GET** /tabs/instances/{id}/value | Get value of form element in tab instance
[**get_tab_elements**](DefaultApi.md#get_tab_elements) | **GET** /tabs/instances/{id}/elements | Get all matching elements' outerHTML for tab
[**get_tab_instance**](DefaultApi.md#get_tab_instance) | **GET** /tabs/instances/{id} | Get tab instance information
[**get_tab_instance_html**](DefaultApi.md#get_tab_instance_html) | **GET** /tabs/instances/{id}/html | Get HTML content of tab instance
[**get_tab_instance_uri**](DefaultApi.md#get_tab_instance_uri) | **GET** /tabs/instances/{id}/uri | Get current URI of tab instance
[**get_tab_title**](DefaultApi.md#get_tab_title) | **GET** /tabs/instances/{id}/title | Get page title
[**hover_scraper_element**](DefaultApi.md#hover_scraper_element) | **POST** /scraper/instances/{id}/hover | Hover over element
[**hover_tab_element**](DefaultApi.md#hover_tab_element) | **POST** /tabs/instances/{id}/hover | Hover over element
[**input_scraper_element**](DefaultApi.md#input_scraper_element) | **POST** /scraper/instances/{id}/input | Type into an element
[**input_tab_element**](DefaultApi.md#input_tab_element) | **POST** /tabs/instances/{id}/input | Type into an element in tab
[**is_scraper_enabled**](DefaultApi.md#is_scraper_enabled) | **GET** /scraper/instances/{id}/isEnabled | Check if element is enabled
[**is_scraper_visible**](DefaultApi.md#is_scraper_visible) | **GET** /scraper/instances/{id}/isVisible | Check if element is visible
[**is_tab_enabled**](DefaultApi.md#is_tab_enabled) | **GET** /tabs/instances/{id}/isEnabled | Check if element is enabled
[**is_tab_visible**](DefaultApi.md#is_tab_visible) | **GET** /tabs/instances/{id}/isVisible | Check if element is visible
[**list_browser_tabs**](DefaultApi.md#list_browser_tabs) | **GET** /tabs/list | List all browser tabs
[**list_workspaces**](DefaultApi.md#list_workspaces) | **GET** /workspaces | List all workspaces
[**navigate_scraper_instance**](DefaultApi.md#navigate_scraper_instance) | **POST** /scraper/instances/{id}/navigate | Navigate scraper instance to URL
[**navigate_tab_instance**](DefaultApi.md#navigate_tab_instance) | **POST** /tabs/instances/{id}/navigate | Navigate tab instance to URL
[**press_scraper_key**](DefaultApi.md#press_scraper_key) | **POST** /scraper/instances/{id}/pressKey | Press key or key combination
[**press_tab_key**](DefaultApi.md#press_tab_key) | **POST** /tabs/instances/{id}/pressKey | Press key or key combination in tab
[**right_click_scraper_element**](DefaultApi.md#right_click_scraper_element) | **POST** /scraper/instances/{id}/rightClick | Right click element
[**right_click_tab_element**](DefaultApi.md#right_click_tab_element) | **POST** /tabs/instances/{id}/rightClick | Right click element
[**save_scraper_pdf**](DefaultApi.md#save_scraper_pdf) | **GET** /scraper/instances/{id}/pdf | Save as PDF
[**save_tab_pdf**](DefaultApi.md#save_tab_pdf) | **GET** /tabs/instances/{id}/pdf | Save as PDF
[**scroll_to_scraper_element**](DefaultApi.md#scroll_to_scraper_element) | **POST** /scraper/instances/{id}/scrollTo | Scroll to element
[**scroll_to_tab_element**](DefaultApi.md#scroll_to_tab_element) | **POST** /tabs/instances/{id}/scrollTo | Scroll to element
[**select_scraper_option**](DefaultApi.md#select_scraper_option) | **POST** /scraper/instances/{id}/selectOption | Select option in dropdown
[**select_tab_option**](DefaultApi.md#select_tab_option) | **POST** /tabs/instances/{id}/selectOption | Select option in dropdown
[**set_scraper_checked**](DefaultApi.md#set_scraper_checked) | **POST** /scraper/instances/{id}/setChecked | Set checkbox/radio checked state
[**set_scraper_cookie**](DefaultApi.md#set_scraper_cookie) | **POST** /scraper/instances/{id}/cookie | Set cookie
[**set_scraper_inner_html**](DefaultApi.md#set_scraper_inner_html) | **POST** /scraper/instances/{id}/setInnerHTML | Set innerHTML of element
[**set_scraper_text_content**](DefaultApi.md#set_scraper_text_content) | **POST** /scraper/instances/{id}/setTextContent | Set textContent of element
[**set_tab_checked**](DefaultApi.md#set_tab_checked) | **POST** /tabs/instances/{id}/setChecked | Set checkbox/radio checked state
[**set_tab_cookie**](DefaultApi.md#set_tab_cookie) | **POST** /tabs/instances/{id}/cookie | Set cookie
[**set_tab_inner_html**](DefaultApi.md#set_tab_inner_html) | **POST** /tabs/instances/{id}/setInnerHTML | Set innerHTML of element in tab
[**set_tab_text_content**](DefaultApi.md#set_tab_text_content) | **POST** /tabs/instances/{id}/setTextContent | Set textContent of element in tab
[**submit_scraper_form**](DefaultApi.md#submit_scraper_form) | **POST** /scraper/instances/{id}/submit | Submit form in scraper instance
[**submit_tab_form**](DefaultApi.md#submit_tab_form) | **POST** /tabs/instances/{id}/submit | Submit form in tab instance
[**subscribe_browser_events**](DefaultApi.md#subscribe_browser_events) | **GET** /browser/events | Subscribe to browser events
[**switch_to_next_workspace**](DefaultApi.md#switch_to_next_workspace) | **POST** /workspaces/next | Switch to next workspace
[**switch_to_previous_workspace**](DefaultApi.md#switch_to_previous_workspace) | **POST** /workspaces/previous | Switch to previous workspace
[**switch_to_workspace**](DefaultApi.md#switch_to_workspace) | **POST** /workspaces/{workspaceId}/switch | Switch to specific workspace
[**take_scraper_element_screenshot**](DefaultApi.md#take_scraper_element_screenshot) | **GET** /scraper/instances/{id}/elementScreenshot | Take screenshot of specific element
[**take_scraper_full_page_screenshot**](DefaultApi.md#take_scraper_full_page_screenshot) | **GET** /scraper/instances/{id}/fullPageScreenshot | Take full page screenshot
[**take_scraper_region_screenshot**](DefaultApi.md#take_scraper_region_screenshot) | **POST** /scraper/instances/{id}/regionScreenshot | Take screenshot of specific region
[**take_scraper_screenshot**](DefaultApi.md#take_scraper_screenshot) | **GET** /scraper/instances/{id}/screenshot | Take screenshot of scraper instance
[**take_tab_element_screenshot**](DefaultApi.md#take_tab_element_screenshot) | **GET** /tabs/instances/{id}/elementScreenshot | Take screenshot of a specific element in tab instance
[**take_tab_full_page_screenshot**](DefaultApi.md#take_tab_full_page_screenshot) | **GET** /tabs/instances/{id}/fullPageScreenshot | Take full page screenshot of tab instance
[**take_tab_region_screenshot**](DefaultApi.md#take_tab_region_screenshot) | **POST** /tabs/instances/{id}/regionScreenshot | Take screenshot of specific region in tab instance
[**take_tab_screenshot**](DefaultApi.md#take_tab_screenshot) | **GET** /tabs/instances/{id}/screenshot | Take screenshot of tab instance
[**upload_scraper_file**](DefaultApi.md#upload_scraper_file) | **POST** /scraper/instances/{id}/uploadFile | Upload file via input[type=file]
[**upload_tab_file**](DefaultApi.md#upload_tab_file) | **POST** /tabs/instances/{id}/uploadFile | Upload file via input[type=file] in tab
[**wait_for_scraper_element**](DefaultApi.md#wait_for_scraper_element) | **POST** /scraper/instances/{id}/waitForElement | Wait for element to appear
[**wait_for_scraper_network_idle**](DefaultApi.md#wait_for_scraper_network_idle) | **POST** /scraper/instances/{id}/waitForNetworkIdle | Wait for network idle
[**wait_for_tab_element**](DefaultApi.md#wait_for_tab_element) | **POST** /tabs/instances/{id}/waitForElement | Wait for element to appear in tab instance
[**wait_for_tab_network_idle**](DefaultApi.md#wait_for_tab_network_idle) | **POST** /tabs/instances/{id}/waitForNetworkIdle | Wait for network idle



## accept_scraper_alert

> models::OkResponse accept_scraper_alert(id)
Accept alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## accept_tab_alert

> models::OkResponse accept_tab_alert(id)
Accept alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## attach_to_tab

> models::AttachResponse attach_to_tab(attach_request)
Attach to existing browser tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**attach_request** | [**AttachRequest**](AttachRequest.md) |  | [required] |

### Return type

[**models::AttachResponse**](AttachResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## check_scraper_instance_exists

> models::ExistsResponse check_scraper_instance_exists(id)
Check if scraper instance exists

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::ExistsResponse**](ExistsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## check_tab_instance_exists

> models::ExistsResponse check_tab_instance_exists(id)
Check if tab instance exists

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::ExistsResponse**](ExistsResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## clear_scraper_input

> models::OkResponse clear_scraper_input(id, selector_request)
Clear input field in scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## clear_tab_input

> models::OkResponse clear_tab_input(id, selector_request)
Clear input field in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## click_scraper_element

> models::OkResponse click_scraper_element(id, selector_request)
Click an element in scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## click_tab_element

> models::OkResponse click_tab_element(id, selector_request)
Click an element in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## close_tab

> models::OkResponse close_tab(id)
Close tab (destroy instance and close browser tab)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_scraper_instance

> models::CreateInstanceResponse create_scraper_instance()
Create a new scraper instance

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::CreateInstanceResponse**](CreateInstanceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## create_tab_instance

> models::CreateInstanceResponse create_tab_instance(create_tab_instance_request)
Create a new tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**create_tab_instance_request** | [**CreateTabInstanceRequest**](CreateTabInstanceRequest.md) |  | [required] |

### Return type

[**models::CreateInstanceResponse**](CreateInstanceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## destroy_scraper_instance

> models::OkResponse destroy_scraper_instance(id)
Destroy a scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## destroy_tab_instance

> models::OkResponse destroy_tab_instance(id)
Destroy tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dismiss_scraper_alert

> models::OkResponse dismiss_scraper_alert(id)
Dismiss alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dismiss_tab_alert

> models::OkResponse dismiss_tab_alert(id)
Dismiss alert

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dispatch_scraper_event

> models::OkResponse dispatch_scraper_event(id, dispatch_event_request)
Dispatch event on element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**dispatch_event_request** | [**DispatchEventRequest**](DispatchEventRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## dispatch_tab_event

> models::OkResponse dispatch_tab_event(id, dispatch_event_request)
Dispatch event on element in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**dispatch_event_request** | [**DispatchEventRequest**](DispatchEventRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## double_click_scraper_element

> models::OkResponse double_click_scraper_element(id, selector_request)
Double click element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## double_click_tab_element

> models::OkResponse double_click_tab_element(id, selector_request)
Double click element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## drag_and_drop_scraper_element

> models::OkResponse drag_and_drop_scraper_element(id, drag_and_drop_request)
Drag and drop element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**drag_and_drop_request** | [**DragAndDropRequest**](DragAndDropRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## drag_and_drop_tab_element

> models::OkResponse drag_and_drop_tab_element(id, drag_and_drop_request)
Drag and drop element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**drag_and_drop_request** | [**DragAndDropRequest**](DragAndDropRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fill_scraper_form

> models::OkResponse fill_scraper_form(id, fill_form_request)
Fill form fields in scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**fill_form_request** | [**FillFormRequest**](FillFormRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fill_tab_form

> models::OkResponse fill_tab_form(id, fill_form_request)
Fill form fields in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**fill_form_request** | [**FillFormRequest**](FillFormRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## focus_scraper_element

> models::OkResponse focus_scraper_element(id, selector_request)
Focus element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## focus_tab_element

> models::OkResponse focus_tab_element(id, selector_request)
Focus element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_browser_context

> models::BrowserContext get_browser_context(history_limit, download_limit)
Get combined browser context (history, tabs, downloads)

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**history_limit** | Option<**i32**> | Maximum number of history items to return |  |[default to 10]
**download_limit** | Option<**i32**> | Maximum number of download items to return |  |[default to 10]

### Return type

[**models::BrowserContext**](BrowserContext.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_browser_downloads

> Vec<models::Download> get_browser_downloads(limit)
Get recent browser downloads

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of download items to return |  |[default to 10]

### Return type

[**Vec<models::Download>**](Download.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_browser_history

> Vec<models::HistoryItem> get_browser_history(limit)
Get recent browser history

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**limit** | Option<**i32**> | Maximum number of history items to return |  |[default to 10]

### Return type

[**Vec<models::HistoryItem>**](HistoryItem.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_browser_tabs

> Vec<models::Tab> get_browser_tabs()
Get recent browser tabs

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::Tab>**](Tab.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_current_workspace

> models::CurrentWorkspaceResponse get_current_workspace()
Get current workspace ID

Returns the ID of the currently selected workspace.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::CurrentWorkspaceResponse**](CurrentWorkspaceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_health

> models::HealthResponse get_health()
Health check endpoint

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::HealthResponse**](HealthResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_attribute

> models::AttributeResponse get_scraper_attribute(id, selector, name)
Get element attribute

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |
**name** | **String** |  | [required] |

### Return type

[**models::AttributeResponse**](AttributeResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_cookies

> models::CookieResponse get_scraper_cookies(id)
Get cookies

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::CookieResponse**](CookieResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_element_by_text

> models::ElementResponse get_scraper_element_by_text(id, text)
Find first element containing text

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**text** | **String** |  | [required] |

### Return type

[**models::ElementResponse**](ElementResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_element_text

> models::TextResponse get_scraper_element_text(id, selector)
Get text content of element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the element | [required] |

### Return type

[**models::TextResponse**](TextResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_element_text_content

> models::TextResponse get_scraper_element_text_content(id, selector)
Get trimmed text content for selector

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::TextResponse**](TextResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_element_value

> models::ValueResponse get_scraper_element_value(id, selector)
Get value of form element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the form element | [required] |

### Return type

[**models::ValueResponse**](ValueResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_elements

> models::GetScraperElements200Response get_scraper_elements(id, selector)
Get all matching elements' outerHTML

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::GetScraperElements200Response**](getScraperElements_200_response.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_instance_html

> models::HtmlResponse get_scraper_instance_html(id)
Get HTML content of scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::HtmlResponse**](HTMLResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_instance_uri

> models::UriResponse get_scraper_instance_uri(id)
Get current URI of scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::UriResponse**](URIResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_scraper_title

> models::TitleResponse get_scraper_title(id)
Get page title

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::TitleResponse**](TitleResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_attribute

> models::AttributeResponse get_tab_attribute(id, selector, name)
Get element attribute

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |
**name** | **String** |  | [required] |

### Return type

[**models::AttributeResponse**](AttributeResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_cookies

> models::CookieResponse get_tab_cookies(id)
Get cookies

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::CookieResponse**](CookieResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_element

> models::GetTabElement200Response get_tab_element(id, selector)
Get element HTML for selector in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the element | [required] |

### Return type

[**models::GetTabElement200Response**](getTabElement_200_response.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_element_by_text

> models::ElementResponse get_tab_element_by_text(id, text)
Find first element containing text in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**text** | **String** |  | [required] |

### Return type

[**models::ElementResponse**](ElementResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_element_text

> models::TextResponse get_tab_element_text(id, selector)
Get text content of element in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the element | [required] |

### Return type

[**models::TextResponse**](TextResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_element_text_content

> models::TextResponse get_tab_element_text_content(id, selector)
Get trimmed text content for selector in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::TextResponse**](TextResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_element_value

> models::ValueResponse get_tab_element_value(id, selector)
Get value of form element in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the form element | [required] |

### Return type

[**models::ValueResponse**](ValueResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_elements

> models::GetScraperElements200Response get_tab_elements(id, selector)
Get all matching elements' outerHTML for tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::GetScraperElements200Response**](getScraperElements_200_response.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_instance

> serde_json::Value get_tab_instance(id)
Get tab instance information

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**serde_json::Value**](serde_json::Value.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_instance_html

> models::HtmlResponse get_tab_instance_html(id)
Get HTML content of tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::HtmlResponse**](HTMLResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_instance_uri

> models::TabUriResponse get_tab_instance_uri(id)
Get current URI of tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::TabUriResponse**](TabURIResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_tab_title

> models::TitleResponse get_tab_title(id)
Get page title

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::TitleResponse**](TitleResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## hover_scraper_element

> models::OkResponse hover_scraper_element(id, selector_request)
Hover over element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## hover_tab_element

> models::OkResponse hover_tab_element(id, selector_request)
Hover over element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## input_scraper_element

> models::OkResponse input_scraper_element(id, input_request)
Type into an element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**input_request** | [**InputRequest**](InputRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## input_tab_element

> models::OkResponse input_tab_element(id, input_request)
Type into an element in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**input_request** | [**InputRequest**](InputRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## is_scraper_enabled

> models::EnabledResponse is_scraper_enabled(id, selector)
Check if element is enabled

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::EnabledResponse**](EnabledResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## is_scraper_visible

> models::VisibleResponse is_scraper_visible(id, selector)
Check if element is visible

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::VisibleResponse**](VisibleResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## is_tab_enabled

> models::EnabledResponse is_tab_enabled(id, selector)
Check if element is enabled

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::EnabledResponse**](EnabledResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## is_tab_visible

> models::VisibleResponse is_tab_visible(id, selector)
Check if element is visible

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** |  | [required] |

### Return type

[**models::VisibleResponse**](VisibleResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_browser_tabs

> Vec<models::TabEntry> list_browser_tabs()
List all browser tabs

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<models::TabEntry>**](TabEntry.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_workspaces

> models::WorkspaceListResponse list_workspaces()
List all workspaces

Returns a list of all workspaces with their metadata and the current workspace ID.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::WorkspaceListResponse**](WorkspaceListResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## navigate_scraper_instance

> models::OkResponse navigate_scraper_instance(id, navigate_request)
Navigate scraper instance to URL

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**navigate_request** | [**NavigateRequest**](NavigateRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## navigate_tab_instance

> models::OkResponse navigate_tab_instance(id, navigate_request)
Navigate tab instance to URL

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**navigate_request** | [**NavigateRequest**](NavigateRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## press_scraper_key

> models::OkResponse press_scraper_key(id, press_key_request)
Press key or key combination

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**press_key_request** | [**PressKeyRequest**](PressKeyRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## press_tab_key

> models::OkResponse press_tab_key(id, press_key_request)
Press key or key combination in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**press_key_request** | [**PressKeyRequest**](PressKeyRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## right_click_scraper_element

> models::OkResponse right_click_scraper_element(id, selector_request)
Right click element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## right_click_tab_element

> models::OkResponse right_click_tab_element(id, selector_request)
Right click element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## save_scraper_pdf

> models::PdfResponse save_scraper_pdf(id)
Save as PDF

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::PdfResponse**](PDFResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## save_tab_pdf

> models::PdfResponse save_tab_pdf(id)
Save as PDF

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::PdfResponse**](PDFResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## scroll_to_scraper_element

> models::OkResponse scroll_to_scraper_element(id, selector_request)
Scroll to element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## scroll_to_tab_element

> models::OkResponse scroll_to_tab_element(id, selector_request)
Scroll to element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## select_scraper_option

> models::OkResponse select_scraper_option(id, select_option_request)
Select option in dropdown

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**select_option_request** | [**SelectOptionRequest**](SelectOptionRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## select_tab_option

> models::OkResponse select_tab_option(id, select_option_request)
Select option in dropdown

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**select_option_request** | [**SelectOptionRequest**](SelectOptionRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_scraper_checked

> models::OkResponse set_scraper_checked(id, set_checked_request)
Set checkbox/radio checked state

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**set_checked_request** | [**SetCheckedRequest**](SetCheckedRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_scraper_cookie

> models::OkResponse set_scraper_cookie(id, cookie_data)
Set cookie

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**cookie_data** | [**CookieData**](CookieData.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_scraper_inner_html

> models::OkResponse set_scraper_inner_html(id, set_inner_html_request)
Set innerHTML of element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**set_inner_html_request** | [**SetInnerHtmlRequest**](SetInnerHtmlRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_scraper_text_content

> models::OkResponse set_scraper_text_content(id, set_text_content_request)
Set textContent of element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**set_text_content_request** | [**SetTextContentRequest**](SetTextContentRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_tab_checked

> models::OkResponse set_tab_checked(id, set_checked_request)
Set checkbox/radio checked state

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**set_checked_request** | [**SetCheckedRequest**](SetCheckedRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_tab_cookie

> models::OkResponse set_tab_cookie(id, cookie_data)
Set cookie

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**cookie_data** | [**CookieData**](CookieData.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_tab_inner_html

> models::OkResponse set_tab_inner_html(id, set_inner_html_request)
Set innerHTML of element in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**set_inner_html_request** | [**SetInnerHtmlRequest**](SetInnerHtmlRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_tab_text_content

> models::OkResponse set_tab_text_content(id, set_text_content_request)
Set textContent of element in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**set_text_content_request** | [**SetTextContentRequest**](SetTextContentRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## submit_scraper_form

> models::OkResponse submit_scraper_form(id, selector_request)
Submit form in scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## submit_tab_form

> models::OkResponse submit_tab_form(id, selector_request)
Submit form in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector_request** | [**SelectorRequest**](SelectorRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## subscribe_browser_events

> String subscribe_browser_events()
Subscribe to browser events

Establishes a Server-Sent Events (SSE) connection to receive real-time updates about browser state changes. 

### Parameters

This endpoint does not need any parameter.

### Return type

**String**

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: text/event-stream

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## switch_to_next_workspace

> models::SwitchWorkspaceResponse switch_to_next_workspace()
Switch to next workspace

Switches to the next workspace in the order.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SwitchWorkspaceResponse**](SwitchWorkspaceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## switch_to_previous_workspace

> models::SwitchWorkspaceResponse switch_to_previous_workspace()
Switch to previous workspace

Switches to the previous workspace in the order.

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::SwitchWorkspaceResponse**](SwitchWorkspaceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## switch_to_workspace

> models::SwitchWorkspaceResponse switch_to_workspace(workspace_id)
Switch to specific workspace

Switches to the workspace with the specified ID.

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**workspace_id** | **uuid::Uuid** | The UUID of the workspace to switch to | [required] |

### Return type

[**models::SwitchWorkspaceResponse**](SwitchWorkspaceResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_scraper_element_screenshot

> models::ImageResponse take_scraper_element_screenshot(id, selector)
Take screenshot of specific element

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the element | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_scraper_full_page_screenshot

> models::ImageResponse take_scraper_full_page_screenshot(id)
Take full page screenshot

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_scraper_region_screenshot

> models::ImageResponse take_scraper_region_screenshot(id, region_screenshot_request)
Take screenshot of specific region

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**region_screenshot_request** | [**RegionScreenshotRequest**](RegionScreenshotRequest.md) |  | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_scraper_screenshot

> models::ImageResponse take_scraper_screenshot(id)
Take screenshot of scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_tab_element_screenshot

> models::ImageResponse take_tab_element_screenshot(id, selector)
Take screenshot of a specific element in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**selector** | **String** | CSS selector for the element | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_tab_full_page_screenshot

> models::ImageResponse take_tab_full_page_screenshot(id)
Take full page screenshot of tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_tab_region_screenshot

> models::ImageResponse take_tab_region_screenshot(id, region_screenshot_request)
Take screenshot of specific region in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**region_screenshot_request** | [**RegionScreenshotRequest**](RegionScreenshotRequest.md) |  | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## take_tab_screenshot

> models::ImageResponse take_tab_screenshot(id)
Take screenshot of tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |

### Return type

[**models::ImageResponse**](ImageResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_scraper_file

> models::OkResponse upload_scraper_file(id, upload_file_request)
Upload file via input[type=file]

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**upload_file_request** | [**UploadFileRequest**](UploadFileRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## upload_tab_file

> models::OkResponse upload_tab_file(id, upload_file_request)
Upload file via input[type=file] in tab

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**upload_file_request** | [**UploadFileRequest**](UploadFileRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## wait_for_scraper_element

> models::FoundResponse wait_for_scraper_element(id, wait_for_element_request)
Wait for element to appear

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**wait_for_element_request** | [**WaitForElementRequest**](WaitForElementRequest.md) |  | [required] |

### Return type

[**models::FoundResponse**](FoundResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## wait_for_scraper_network_idle

> models::OkResponse wait_for_scraper_network_idle(id, wait_for_network_idle_request)
Wait for network idle

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**wait_for_network_idle_request** | [**WaitForNetworkIdleRequest**](WaitForNetworkIdleRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## wait_for_tab_element

> models::FoundResponse wait_for_tab_element(id, wait_for_element_request)
Wait for element to appear in tab instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**wait_for_element_request** | [**WaitForElementRequest**](WaitForElementRequest.md) |  | [required] |

### Return type

[**models::FoundResponse**](FoundResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## wait_for_tab_network_idle

> models::OkResponse wait_for_tab_network_idle(id, wait_for_network_idle_request)
Wait for network idle

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**wait_for_network_idle_request** | [**WaitForNetworkIdleRequest**](WaitForNetworkIdleRequest.md) |  | [required] |

### Return type

[**models::OkResponse**](OkResponse.md)

### Authorization

[BearerAuth](../README.md#BearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

