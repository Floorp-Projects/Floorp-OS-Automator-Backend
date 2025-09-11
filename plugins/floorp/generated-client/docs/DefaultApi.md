# \DefaultApi

All URIs are relative to *http://127.0.0.1:58261*

Method | HTTP request | Description
------------- | ------------- | -------------
[**attach_to_tab**](DefaultApi.md#attach_to_tab) | **POST** /tabs/attach | Attach to existing browser tab
[**check_scraper_instance_exists**](DefaultApi.md#check_scraper_instance_exists) | **GET** /scraper/instances/{id}/exists | Check if scraper instance exists
[**check_tab_instance_exists**](DefaultApi.md#check_tab_instance_exists) | **GET** /tabs/instances/{id}/exists | Check if tab instance exists
[**click_scraper_element**](DefaultApi.md#click_scraper_element) | **POST** /scraper/instances/{id}/click | Click an element in scraper instance
[**create_scraper_instance**](DefaultApi.md#create_scraper_instance) | **POST** /scraper/instances | Create a new scraper instance
[**create_tab_instance**](DefaultApi.md#create_tab_instance) | **POST** /tabs/instances | Create a new tab instance
[**destroy_scraper_instance**](DefaultApi.md#destroy_scraper_instance) | **DELETE** /scraper/instances/{id} | Destroy a scraper instance
[**destroy_tab_instance**](DefaultApi.md#destroy_tab_instance) | **DELETE** /tabs/instances/{id} | Destroy tab instance
[**execute_scraper_script**](DefaultApi.md#execute_scraper_script) | **POST** /scraper/instances/{id}/executeScript | Execute JavaScript in scraper instance
[**fill_scraper_form**](DefaultApi.md#fill_scraper_form) | **POST** /scraper/instances/{id}/fillForm | Fill form fields in scraper instance
[**get_browser_context**](DefaultApi.md#get_browser_context) | **GET** /browser/context | Get combined browser context (history, tabs, downloads)
[**get_browser_downloads**](DefaultApi.md#get_browser_downloads) | **GET** /browser/downloads | Get recent browser downloads
[**get_browser_history**](DefaultApi.md#get_browser_history) | **GET** /browser/history | Get recent browser history
[**get_browser_tabs**](DefaultApi.md#get_browser_tabs) | **GET** /browser/tabs | Get recent browser tabs
[**get_health**](DefaultApi.md#get_health) | **GET** /health | Health check endpoint
[**get_scraper_element_text**](DefaultApi.md#get_scraper_element_text) | **GET** /scraper/instances/{id}/elementText | Get text content of element
[**get_scraper_element_value**](DefaultApi.md#get_scraper_element_value) | **GET** /scraper/instances/{id}/value | Get value of form element
[**get_scraper_instance_html**](DefaultApi.md#get_scraper_instance_html) | **GET** /scraper/instances/{id}/html | Get HTML content of scraper instance
[**get_scraper_instance_uri**](DefaultApi.md#get_scraper_instance_uri) | **GET** /scraper/instances/{id}/uri | Get current URI of scraper instance
[**get_tab_instance**](DefaultApi.md#get_tab_instance) | **GET** /tabs/instances/{id} | Get tab instance information
[**get_tab_instance_uri**](DefaultApi.md#get_tab_instance_uri) | **GET** /tabs/instances/{id}/uri | Get current URI of tab instance
[**list_browser_tabs**](DefaultApi.md#list_browser_tabs) | **GET** /tabs/list | List all browser tabs
[**navigate_scraper_instance**](DefaultApi.md#navigate_scraper_instance) | **POST** /scraper/instances/{id}/navigate | Navigate scraper instance to URL
[**navigate_tab_instance**](DefaultApi.md#navigate_tab_instance) | **POST** /tabs/instances/{id}/navigate | Navigate tab instance to URL
[**submit_scraper_form**](DefaultApi.md#submit_scraper_form) | **POST** /scraper/instances/{id}/submit | Submit form in scraper instance
[**take_scraper_element_screenshot**](DefaultApi.md#take_scraper_element_screenshot) | **GET** /scraper/instances/{id}/elementScreenshot | Take screenshot of specific element
[**take_scraper_full_page_screenshot**](DefaultApi.md#take_scraper_full_page_screenshot) | **GET** /scraper/instances/{id}/fullPageScreenshot | Take full page screenshot
[**take_scraper_region_screenshot**](DefaultApi.md#take_scraper_region_screenshot) | **POST** /scraper/instances/{id}/regionScreenshot | Take screenshot of specific region
[**take_scraper_screenshot**](DefaultApi.md#take_scraper_screenshot) | **GET** /scraper/instances/{id}/screenshot | Take screenshot of scraper instance
[**wait_for_scraper_element**](DefaultApi.md#wait_for_scraper_element) | **POST** /scraper/instances/{id}/waitForElement | Wait for element to appear



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


## execute_scraper_script

> models::OkResponse execute_scraper_script(id, execute_script_request)
Execute JavaScript in scraper instance

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**id** | **String** | Instance identifier | [required] |
**execute_script_request** | [**ExecuteScriptRequest**](ExecuteScriptRequest.md) |  | [required] |

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

