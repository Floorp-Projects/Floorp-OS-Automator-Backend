// Calendar Aggregation Demo - Combined Workflow
// This workflow:
// 1. Reads calendar events from Thunderbird (via Plugin API)
// 2. Scrapes events from Google Calendar (web)
// 3. Analyzes Google Form HTML structure
// 4. Fills the form with available dates

// =============================================================================
// CONFIGURATION
// =============================================================================

var CONFIG = {
  // Google Form URL
  formUrl:
    "https://docs.google.com/forms/d/e/1FAIpQLSfcJRyBQFYcS6sbFYaWfT1GMsk411I-Bl1ODE7oQSxpYh3nDg/viewform",

  // Days to check for calendar events
  daysToCheck: 14,

  // Days to look ahead for scheduling (starting from tomorrow)
  schedulingDaysLookAhead: 7,

  // Time slots to prefer
  preferredTimeSlots: [
    "10:00-12:00",
    "13:00-15:00",
    "15:00-17:00",
    "17:00-19:00",
  ],
};

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

// Show simple progress message
function log(msg) {
  console.log("[calendar] " + msg);
}

// Generate date range starting from tomorrow
function generateDateRange(daysFromNow) {
  var dates = [];
  var today = new Date();

  for (var i = 1; i <= daysFromNow; i++) {
    var date = new Date(today);
    date.setDate(today.getDate() + i);

    var year = date.getFullYear();
    var month = String(date.getMonth() + 1).padStart(2, '0');
    var day = String(date.getDate()).padStart(2, '0');

    dates.push(year + "-" + month + "-" + day);
  }

  return dates;
}

// =============================================================================
// PLUGIN CHECKS
// =============================================================================

function checkPluginAvailability() {
  if (typeof floorp === "undefined") {
    throw new Error("Floorp plugin is required for this workflow");
  }
}

function setupErrorHandlers() {
  if (typeof window === 'undefined') return;

  window.addEventListener('error', function(e) {
    console.error("Error: " + e.message);
  });

  window.addEventListener('unhandledrejection', function(e) {
    console.error("Promise rejection: " + e.reason);
  });
}

// =============================================================================
// READ THUNDERBIRD CALENDAR
// =============================================================================

function readThunderbirdCalendar() {
  var events = [];

  try {
    var rawEvents = thunderbird.getCalendarEvents(CONFIG.daysToCheck);
    log("Thunderbird: " + rawEvents.length + " events");

    for (var i = 0; i < rawEvents.length; i++) {
      var ev = rawEvents[i];
      events.push({
        title: ev.title,
        startTime: ev.start_time,
        endTime: ev.end_time,
        date: ev.date,
        source: "thunderbird",
      });
    }
  } catch (error) {
    // Continue without Thunderbird data
  }

  return events;
}

// =============================================================================
// GET EMAILS FROM THUNDERBIRD (side effects only)
// =============================================================================

function getThunderbirdEmails() {
  var emails = { inbox: [], sent: [] };
  try { emails.inbox = thunderbird.tryGetEmails("inbox", 5); } catch (e) {}
  try { emails.sent = thunderbird.tryGetEmails("sent", 5); } catch (e) {}
  return emails;
}

// =============================================================================
// GET USER IDENTITY FROM THUNDERBIRD
// =============================================================================

function getThunderbirdIdentity() {
  try {
    var identity = thunderbird.getIdentity();
    CONFIG.userName = identity.name;
    CONFIG.userEmail = identity.email;
    return identity;
  } catch (err) {
    CONFIG.userName = CONFIG.userName || "Unknown User";
    CONFIG.userEmail = CONFIG.userEmail || "unknown@example.com";
    return {
      name: CONFIG.userName,
      email: CONFIG.userEmail,
      profile: "fallback",
    };
  }
}

// =============================================================================
// SCRAPE GOOGLE CALENDAR
// =============================================================================

function scrapeGoogleCalendar() {
  var tabId = floorp.createTab(
    "https://calendar.google.com/calendar/u/0/r/week",
    false,
  );

  floorp.tabWaitForNetworkIdle(tabId);
  floorp.tabWaitForElement(tabId, "[data-view-heading]", 10000);
  floorp.tabWaitForElement(tabId, "[data-eventid]", 10000);

  var events = [];

  try {
    var elementsJson = floorp.tabGetElements(tabId, "[data-eventchip]");
    var parsed = JSON.parse(elementsJson);
    var elementHtmlList = parsed.elements || [];

    for (var i = 0; i < elementHtmlList.length; i++) {
      var html = elementHtmlList[i];
      var ariaMatch = html.match(/aria-label="([^"]+)"/);
      var fullDetails = ariaMatch ? ariaMatch[1] : "";

      if (!fullDetails) {
        var spanMatch = html.match(/class="XuJrye"[^>]*>([^<]+)</);
        fullDetails = spanMatch ? spanMatch[1] : "";
      }

      if (!fullDetails) continue;

      var skipPatterns = ["終日の予定", "件の予定", "予定はありません"];
      var shouldSkip = false;
      for (var s = 0; s < skipPatterns.length; s++) {
        if (fullDetails.indexOf(skipPatterns[s]) !== -1) {
          shouldSkip = true;
          break;
        }
      }
      if (shouldSkip) continue;

      var eventTitle = "";
      var eventDate = "";
      var eventTime = "";
      var calendarName = "";

      var titleMatch = fullDetails.match(/「([^」]+)」/);
      if (titleMatch) eventTitle = titleMatch[1];

      var titleIndex = fullDetails.indexOf("「");
      if (titleIndex > 0) {
        var timePart = fullDetails.substring(0, titleIndex);
        eventTime = timePart.replace(/、\s*$/, "").trim();
      }

      var datePattern = /(\d{4})年\s*(\d{1,2})月\s*(\d{1,2})日/g;
      var dateMatch;
      var lastDateMatch = null;
      while ((dateMatch = datePattern.exec(fullDetails)) !== null) {
        lastDateMatch = dateMatch;
      }
      if (lastDateMatch) {
        var month = lastDateMatch[2];
        var day = lastDateMatch[3];
        if (month.length === 1) month = "0" + month;
        if (day.length === 1) day = "0" + day;
        eventDate = lastDateMatch[1] + "-" + month + "-" + day;
      }

      var calMatch = fullDetails.match(/カレンダー:\s*([^、]+)/);
      if (calMatch) calendarName = calMatch[1].trim();

      var isDuplicate = false;
      for (var j = 0; j < events.length; j++) {
        if (
          events[j].title === eventTitle &&
          events[j].time === eventTime &&
          events[j].calendar === calendarName
        ) {
          isDuplicate = true;
          break;
        }
      }
      if (isDuplicate) continue;

      events.push({
        title: eventTitle,
        date: eventDate,
        time: eventTime,
        calendar: calendarName,
        source: "google_calendar",
      });
    }
  } catch (error) {
    console.error("Error parsing events: " + error);
  }

  try { floorp.closeTab(tabId); } catch (e) {}

  log("Google Calendar: " + events.length + " events");
  return events;
}

// =============================================================================
// ANALYZE FORM HTML STRUCTURE
// =============================================================================

function analyzeFormStructure(tabId) {
  var htmlContent = floorp.tabHtml(tabId);

  var textInputCount = 0;
  var searchPos = 0;
  while ((searchPos = htmlContent.indexOf("aria-labelledby=", searchPos)) !== -1) {
    textInputCount++;
    searchPos++;
  }

  var dateInputCount = 0;
  searchPos = 0;
  while ((searchPos = htmlContent.indexOf('type="date"', searchPos)) !== -1) {
    dateInputCount++;
    searchPos++;
  }

  var radioCount = 0;
  searchPos = 0;
  while ((searchPos = htmlContent.indexOf('role="radio"', searchPos)) !== -1) {
    radioCount++;
    searchPos++;
  }

  return {
    htmlContent: htmlContent,
    fieldCounts: {
      textInputs: textInputCount,
      dateInputs: dateInputCount,
      radioButtons: radioCount,
    },
  };
}

// =============================================================================
// FILL THE FORM
// =============================================================================

function fillForm(tabId, availableDates) {
  var firstDate = availableDates[0] || "";
  var secondDate = availableDates[1] || "";

  if (!firstDate) {
    var tomorrowDates = generateDateRange(1);
    firstDate = tomorrowDates[0];
  }
  if (!secondDate) {
    secondDate = firstDate;
  }

  floorp.tabWaitForElement(tabId, "form", 10000);

  // Fill name
  try {
    floorp.tabWaitForElement(tabId, "input[type='text']", 5000);
    floorp.tabInput(tabId, "input[type='text']", CONFIG.userName);
  } catch (e) {
    console.error("Failed to fill name: " + e);
  }

  // Fill email
  try {
    floorp.tabInput(tabId, "input[type='email']", CONFIG.userEmail);
  } catch (e) {
    console.error("Failed to fill email: " + e);
  }

  // Fill first date
  var dateInputs = [];
  try {
    var dateInputsRaw = floorp.tabGetElements(tabId, "input[type='date']");
    var parsed = JSON.parse(dateInputsRaw);
    if (parsed && parsed.elements) {
      dateInputs = parsed.elements;
    }
  } catch (e) {}

  if (dateInputs.length >= 1) {
    try {
      floorp.tabInput(tabId, "input[type='date']:nth-of-type(1)", firstDate);
    } catch (e) {
      try { floorp.tabInput(tabId, "input[type='date']", firstDate); } catch (e2) {}
    }
  }

  // Select first time slot
  try {
    floorp.tabClick(tabId, "div[aria-label='" + CONFIG.preferredTimeSlots[0] + "']");
  } catch (e) {}

  // Fill second date
  if (dateInputs.length >= 2) {
    var selectors = [
      "div[role='listitem']:nth-child(5) input[type='date']",
      "li:nth-child(5) input[type='date']",
      "div[role='list'] > div:nth-child(5) input[type='date']",
      "form > div > div > div:nth-child(5) input[type='date']",
    ];
    for (var si = 0; si < selectors.length; si++) {
      try {
        floorp.tabClick(tabId, selectors[si]);
        floorp.tabInput(tabId, selectors[si], secondDate);
        break;
      } catch (e) {}
    }
  }

  // Select second time slot
  var timeSlotSelectors = [
    "div[role='listitem']:nth-child(6) div[role='radio']",
    "div[role='listitem']:nth-child(6) div[aria-label='" + CONFIG.preferredTimeSlots[0] + "']",
    "li:nth-child(6) div[role='radio']",
  ];
  for (var ti = 0; ti < timeSlotSelectors.length; ti++) {
    try {
      floorp.tabClick(tabId, timeSlotSelectors[ti]);
      break;
    } catch (e) {}
  }

  log("Form filled: " + firstDate + ", " + secondDate);
}

// =============================================================================
// CALCULATE AVAILABLE DATES
// =============================================================================

function calculateAvailableDates(thunderbirdEvents, googleEvents) {
  var dateRange = generateDateRange(CONFIG.schedulingDaysLookAhead);
  var BUSINESS_START = 10;
  var BUSINESS_END = 17;

  function parseJapaneseTime(timeStr) {
    if (!timeStr) return null;
    if (timeStr.indexOf("終日") !== -1) {
      return { start: 0, end: 24 };
    }
    var rangeMatch = timeStr.match(
      /(午前|午後)?(\d{1,2})(?:時|:)(\d{0,2})?\s*[～~－\-]\s*(午前|午後)?(\d{1,2})(?:時|:)?(\d{0,2})?/,
    );
    if (rangeMatch) {
      var startPeriod = rangeMatch[1] || "午前";
      var startHour = parseInt(rangeMatch[2], 10);
      var endPeriod = rangeMatch[4] || startPeriod;
      var endHour = parseInt(rangeMatch[5], 10);
      if (startPeriod === "午後" && startHour !== 12) startHour += 12;
      if (startPeriod === "午前" && startHour === 12) startHour = 0;
      if (endPeriod === "午後" && endHour !== 12) endHour += 12;
      if (endPeriod === "午前" && endHour === 12) endHour = 0;
      return { start: startHour, end: endHour };
    }
    var singleMatch = timeStr.match(/(午前|午後)?(\d{1,2})(?:時|:)/);
    if (singleMatch) {
      var period = singleMatch[1] || "午前";
      var hour = parseInt(singleMatch[2], 10);
      if (period === "午後" && hour !== 12) hour += 12;
      if (period === "午前" && hour === 12) hour = 0;
      return { start: hour, end: hour + 1 };
    }
    return null;
  }

  function parseISOTime(startTime, endTime) {
    if (!startTime) return null;
    var startMatch = startTime.match(/(\d{2}):(\d{2})/);
    var endMatch = endTime ? endTime.match(/(\d{2}):(\d{2})/) : null;
    if (startMatch) {
      var startHour = parseInt(startMatch[1], 10);
      var endHour = endMatch ? parseInt(endMatch[1], 10) : startHour + 1;
      return { start: startHour, end: endHour };
    }
    return null;
  }

  var busyTimesByDate = {};

  for (var j = 0; j < thunderbirdEvents.length; j++) {
    var event = thunderbirdEvents[j];
    if (event.date) {
      var timeRange = parseISOTime(event.startTime, event.endTime);
      if (!busyTimesByDate[event.date]) {
        busyTimesByDate[event.date] = [];
      }
      busyTimesByDate[event.date].push({
        title: event.title,
        start: timeRange ? timeRange.start : 0,
        end: timeRange ? timeRange.end : 24,
      });
    }
  }

  for (var k = 0; k < googleEvents.length; k++) {
    var gEvent = googleEvents[k];
    if (gEvent.date) {
      var gTimeRange = parseJapaneseTime(gEvent.time);
      if (!busyTimesByDate[gEvent.date]) {
        busyTimesByDate[gEvent.date] = [];
      }
      busyTimesByDate[gEvent.date].push({
        title: gEvent.title,
        start: gTimeRange ? gTimeRange.start : 0,
        end: gTimeRange ? gTimeRange.end : 24,
      });
    }
  }

  var availableDates = [];

  for (var i = 0; i < dateRange.length; i++) {
    var dateStr = dateRange[i];
    var busyTimes = busyTimesByDate[dateStr] || [];
    var isAvailable = true;

    for (var b = 0; b < busyTimes.length; b++) {
      var busy = busyTimes[b];
      if (busy.start < BUSINESS_END && busy.end > BUSINESS_START) {
        isAvailable = false;
        break;
      }
    }

    if (isAvailable) {
      availableDates.push(dateStr);
    }
  }

  log("Available dates: " + availableDates.join(", "));
  return availableDates;
}

// =============================================================================
// ADD EVENT TO GOOGLE CALENDAR
// =============================================================================

function addEventToGoogleCalendar(date, timeSlot, title) {
  var timeParts = timeSlot.split("-");
  var startTime = timeParts[0].replace(":", "");
  var endTime = timeParts[1].replace(":", "");
  var dateFormatted = date.replace(/-/g, "");
  var startDateTime = dateFormatted + "T" + startTime + "00";
  var endDateTime = dateFormatted + "T" + endTime + "00";

  var calendarUrl =
    "https://calendar.google.com/calendar/render?action=TEMPLATE" +
    "&text=" + encodeURIComponent(title) +
    "&dates=" + startDateTime + "/" + endDateTime +
    "&details=" + encodeURIComponent("Floorp OS Automator により自動追加された予定です。");

  var calendarTabRaw = floorp.createTab(calendarUrl, false);
  var calendarTabId = calendarTabRaw;

  try {
    var parsed = JSON.parse(calendarTabRaw);
    if (parsed && parsed.instanceId) {
      calendarTabId = parsed.instanceId;
    }
  } catch (e) {}

  log("Calendar event created: " + date + " " + timeSlot);
  return calendarTabId;
}

// =============================================================================
// NATURAL LANGUAGE PARSING FUNCTIONS
// =============================================================================

function parseDateFromPrompt(prompt) {
  if (!prompt) {
    return generateDateRange(1)[0];
  }

  var isoMatch = prompt.match(/(\d{4})-(\d{1,2})-(\d{1,2})/);
  if (isoMatch) {
    return isoMatch[1] + "-" +
           String(isoMatch[2]).padStart(2, '0') + "-" +
           String(isoMatch[3]).padStart(2, '0');
  }

  var monthDayMatch = prompt.match(/(\d{1,2})月(\d{1,2})日/);
  if (monthDayMatch) {
    var currentYear = new Date().getFullYear();
    return currentYear + "-" +
           String(monthDayMatch[1]).padStart(2, '0') + "-" +
           String(monthDayMatch[2]).padStart(2, '0');
  }

  if (prompt.indexOf("明日") !== -1 || prompt.indexOf("あした") !== -1) {
    return generateDateRange(1)[0];
  }

  if (prompt.indexOf("明後日") !== -1 || prompt.indexOf("あさって") !== -1) {
    return generateDateRange(2)[1];
  }

  if (prompt.indexOf("再来日") !== -1) {
    return generateDateRange(3)[2];
  }

  return generateDateRange(1)[0];
}

function parseTimeFromPrompt(prompt) {
  if (!prompt) {
    return CONFIG.preferredTimeSlots[0] || "10:00-12:00";
  }

  var timeRangeMatch = prompt.match(/(\d{1,2}):(\d{2})\s*[-~〜－]\s*(\d{1,2}):(\d{2})/);
  if (timeRangeMatch) {
    return timeRangeMatch[1] + ":" + timeRangeMatch[2] + "-" +
           timeRangeMatch[3] + ":" + timeRangeMatch[4];
  }

  var jpTimeMatch = prompt.match(/(\d{1,2})時\s*(から|ー|-)\s*(\d{1,2})時/);
  if (jpTimeMatch) {
    return jpTimeMatch[1] + ":00-" + jpTimeMatch[3] + ":00";
  }

  var singleTimeMatch = prompt.match(/(\d{1,2})時/);
  if (singleTimeMatch) {
    var hour = String(singleTimeMatch[1]).padStart(2, '0');
    return hour + ":00-" + hour + ":59";
  }

  return CONFIG.preferredTimeSlots[0] || "10:00-12:00";
}

function parseSecondDateFromPrompt(prompt, firstDate) {
  var secondMatch = prompt.match(/と\s*(\d{1,2})月(\d{1,2})日/);
  if (secondMatch) {
    var currentYear = new Date().getFullYear();
    return currentYear + "-" +
           String(secondMatch[1]).padStart(2, '0') + "-" +
           String(secondMatch[2]).padStart(2, '0');
  }

  var allDates = prompt.match(/(\d{4}-\d{2}-\d{2})/g);
  if (allDates && allDates.length >= 2) {
    return allDates[1];
  }

  var nextDay = new Date(firstDate);
  nextDay.setDate(nextDay.getDate() + 1);
  return nextDay.toISOString().split('T')[0];
}

// =============================================================================
// OPEN OR CREATE FORM TAB
// =============================================================================

function openOrCreateFormTab() {
  var formTabId = null;
  var formTab = null;

  try {
    var tabsResponse = floorp.browserTabs();
    var tabsData = JSON.parse(tabsResponse);
    var tabs = tabsData.tabs || tabsData;

    var formIdMatch = CONFIG.formUrl.match(/\/d\/e\/([^/]+)/);
    var formId = formIdMatch ? formIdMatch[1] : null;

    for (var i = 0; i < tabs.length; i++) {
      var url = tabs[i].url || "";
      if (
        url.indexOf(CONFIG.formUrl) !== -1 ||
        (formId && url.indexOf(formId) !== -1)
      ) {
        formTab = tabs[i];
        break;
      }
    }
  } catch (e) {}

  if (formTab) {
    formTabId = String(formTab.instance_id || formTab.id);
    floorp.attachToTab(formTabId);
  } else {
    formTabId = floorp.createTab(CONFIG.formUrl, false);
  }

  floorp.tabWaitForNetworkIdle(formTabId);
  return formTabId;
}

// =============================================================================
// MAIN EXECUTION
// =============================================================================

function workflow() {
  log("Starting workflow...");

  setupErrorHandlers();
  checkPluginAvailability();

  var identity = getThunderbirdIdentity();
  getThunderbirdEmails();

  var thunderbirdEvents = readThunderbirdCalendar();

  try {
    thunderbird.activateFloorp();
  } catch (e) {}

  var googleEvents = scrapeGoogleCalendar();
  var availableDates = calculateAvailableDates(thunderbirdEvents, googleEvents);

  var formTabId = openOrCreateFormTab();
  analyzeFormStructure(formTabId);
  fillForm(formTabId, availableDates);

  var firstDate = availableDates[0];
  if (!firstDate) {
    var tomorrowDates = generateDateRange(1);
    firstDate = tomorrowDates[0];
  }
  var timeSlot = CONFIG.preferredTimeSlots[0];

  var calendarTabId = addEventToGoogleCalendar(firstDate, timeSlot, "病院の予約");

  log("Workflow complete");

  // Cleanup
  try {
    floorp.destroyTabInstance(formTabId);
  } catch (e) {
    console.error("Failed to destroy form tab instance: " + e);
  }
  try {
    if (calendarTabId) {
      floorp.destroyTabInstance(calendarTabId);
    }
  } catch (e) {
    console.error("Failed to destroy calendar tab instance: " + e);
  }

  return {
    identity: identity ? { name: identity.name, email: identity.email } : null,
    thunderbirdEvents: thunderbirdEvents.length,
    googleEvents: googleEvents.length,
    availableDates: availableDates.slice(0, 5),
  };
}

// =============================================================================
// SUBMIT APPOINTMENT WORKFLOW (Natural Language Entry Point)
// =============================================================================

function submitAppointment() {
  var userPrompt = globalThis.userPrompt || "";
  log("Submit appointment: " + userPrompt);

  var firstDate = parseDateFromPrompt(userPrompt);
  var secondDate = parseSecondDateFromPrompt(userPrompt, firstDate);
  var timeSlot = parseTimeFromPrompt(userPrompt);

  setupErrorHandlers();
  checkPluginAvailability();
  getThunderbirdIdentity();

  var formTabId = openOrCreateFormTab();
  analyzeFormStructure(formTabId);
  fillForm(formTabId, [firstDate, secondDate]);

  try {
    floorp.tabClick(formTabId, "div[aria-label='" + timeSlot + "']");
  } catch (e) {
    console.log("Manual time selection needed: " + timeSlot);
  }

  var calendarTabId = addEventToGoogleCalendar(firstDate, timeSlot, "病院の予約");

  log("Appointment complete: " + firstDate + " " + timeSlot);

  // Cleanup
  try {
    floorp.destroyTabInstance(formTabId);
  } catch (e) {}
  try {
    if (calendarTabId) {
      floorp.destroyTabInstance(calendarTabId);
    }
  } catch (e) {}

  return {
    firstDate: firstDate,
    secondDate: secondDate,
    timeSlot: timeSlot,
    formTabId: formTabId,
    calendarTabId: calendarTabId,
  };
}
