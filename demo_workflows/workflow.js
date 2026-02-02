// Calendar Aggregation Demo - Combined Workflow
// This workflow:
// 1. Opens and analyzes Google Form HTML structure
// 2. Reads user identity from Thunderbird
// 3. Reads calendar events from Thunderbird (via Plugin API)
// 4. Scrapes events from Google Calendar (web)
// 5. Calculates available dates and fills the form
// 6. Adds an event to Google Calendar

// =============================================================================
// EXTERNAL PLUGIN LOADING
// =============================================================================

// Check if external plugins are available (silent check)
try {
  // External plugins are loaded as: globalThis["sapphillon"]["thunderbird"]
  if (
    typeof sapphillon === "undefined" ||
    typeof sapphillon.thunderbird === "undefined"
  ) {
    // Plugin not available - will use fallback
  }
} catch (error) {
  // Silent error handling
}

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

// Demo-friendly progress logging with emojis
var STEP_ICONS = {
  form: "📋",
  identity: "👤",
  calendar: "📅",
  google: "🌐",
  calculate: "🧮",
  fill: "✏️",
  add: "➕",
  complete: "✅",
  error: "❌",
  info: "ℹ️",
};

function log(msg, icon) {
  var prefix = icon ? STEP_ICONS[icon] + " " : "";
  console.log(prefix + msg);
}

function logStep(stepNum, totalSteps, msg, icon) {
  var iconStr = icon ? STEP_ICONS[icon] + " " : "";
  console.log("[" + stepNum + "/" + totalSteps + "] " + iconStr + msg);
}

function logResult(label, value) {
  console.log("    → " + label + ": " + value);
}

// Generate date range starting from tomorrow
function generateDateRange(daysFromNow) {
  var dates = [];
  var today = new Date();

  for (var i = 1; i <= daysFromNow; i++) {
    var date = new Date(today);
    date.setDate(today.getDate() + i);

    var year = date.getFullYear();
    var month = String(date.getMonth() + 1).padStart(2, "0");
    var day = String(date.getDate()).padStart(2, "0");

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
  if (typeof window === "undefined") return;

  window.addEventListener("error", function (e) {
    console.error("Error: " + e.message);
  });

  window.addEventListener("unhandledrejection", function (e) {
    console.error("Promise rejection: " + e.reason);
  });
}

// =============================================================================
// READ THUNDERBIRD CALENDAR
// =============================================================================

function readThunderbirdCalendar() {
  var events = [];

  try {
    var rawEvents = sapphillon.thunderbird.getCalendarEvents(
      CONFIG.daysToCheck,
    );

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
// GET USER IDENTITY FROM THUNDERBIRD
// =============================================================================

function getThunderbirdIdentity() {
  try {
    var identity = sapphillon.thunderbird.getIdentity();
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

  try {
    floorp.closeTab(tabId);
  } catch (e) {}

  return events;
}

// =============================================================================
// ANALYZE FORM HTML STRUCTURE (AI-Powered)
// =============================================================================

function analyzeFormStructure(tabId) {
  var formFields = [];

  // Get all form field containers using Google Forms structure
  try {
    var fieldContainersRaw = floorp.tabGetElements(
      tabId,
      "div[role='listitem']",
    );
    var parsed = JSON.parse(fieldContainersRaw);
    var containers = parsed.elements || [];

    // Combine all field HTML for AI analysis (limit to first 8000 chars per field)
    var fieldsHtmlForAI = [];
    for (var i = 0; i < containers.length; i++) {
      var html = containers[i];
      if (html && html.length > 50) {
        fieldsHtmlForAI.push({
          index: i + 1,
          html: html.substring(0, 8000),
        });
      }
    }

    if (fieldsHtmlForAI.length > 0) {
      // Use AI to extract form field information
      formFields = extractFormFieldsWithAI(fieldsHtmlForAI);
    }
  } catch (e) {
    console.error("Error analyzing form: " + e);
  }

  // Count by type
  var counts = {
    text: 0,
    email: 0,
    date: 0,
    radio: 0,
    checkbox: 0,
    textarea: 0,
    other: 0,
  };

  for (var j = 0; j < formFields.length; j++) {
    var type = formFields[j].type;
    if (counts[type] !== undefined) {
      counts[type]++;
    } else {
      counts.other++;
    }
  }

  return {
    fields: formFields,
    fieldCounts: {
      textInputs: counts.text + counts.email,
      dateInputs: counts.date,
      radioButtons: counts.radio,
      checkboxes: counts.checkbox,
      textareas: counts.textarea,
    },
    totalFields: formFields.length,
  };
}

// Use INIAD AI MOP to extract form field labels and types
function extractFormFieldsWithAI(fieldsHtmlArray) {
  var formFields = [];
  var MAX_RETRIES = 3;

  // Build prompt with all field HTML
  var fieldsDescription = "";
  for (var i = 0; i < fieldsHtmlArray.length; i++) {
    fieldsDescription +=
      "=== フィールド " +
      fieldsHtmlArray[i].index +
      " ===\n" +
      fieldsHtmlArray[i].html.substring(0, 3000) +
      "\n\n";
  }

  var systemPrompt =
    "あなたはHTMLフォーム解析の専門家です。Google Forms のHTMLから各フィールドの情報を正確に抽出してください。必ず有効なJSONのみを出力してください。";

  var userPrompt =
    "以下のGoogle FormsのHTMLから、各フィールドの情報を抽出してJSON配列で出力してください。\n\n" +
    "重要: 有効なJSON配列のみを出力してください。説明やマークダウンは不要です。\n\n" +
    "出力形式（JSON配列のみ）:\n" +
    '[{"index":1,"label":"お名前","type":"text","required":true,"options":[]},{"index":2,"label":"メールアドレス","type":"email","required":true,"options":[]}]\n\n' +
    "type は以下のいずれか: text, email, date, radio, checkbox, textarea, unknown\n" +
    "radio や checkbox の場合は options に選択肢を配列で含める\n" +
    "「必須」や「*」マークがあれば required: true\n\n" +
    "HTMLフィールド一覧:\n" +
    fieldsDescription;

  var lastError = null;

  for (var retry = 0; retry < MAX_RETRIES; retry++) {
    try {
      var retryPrompt = userPrompt;
      
      // If previous attempt failed, add error context to prompt
      if (lastError && retry > 0) {
        retryPrompt = 
          "前回のJSON出力にエラーがありました: " + lastError + "\n" +
          "今回は必ず有効なJSONを出力してください。\n\n" + userPrompt;
      }

      var aiResponse = iniad_ai_mop.chat(systemPrompt, retryPrompt);

      // Clean up response and parse JSON
      aiResponse = aiResponse
        .replace(/```json\n?/g, "")
        .replace(/```\n?/g, "")
        .trim();

      var jsonStart = aiResponse.indexOf("[");
      var jsonEnd = aiResponse.lastIndexOf("]") + 1;

      if (jsonStart >= 0 && jsonEnd > jsonStart) {
        var jsonStr = aiResponse.slice(jsonStart, jsonEnd);
        
        // Try to fix common JSON issues before parsing
        jsonStr = jsonStr
          .replace(/,\s*]/g, "]")  // Remove trailing commas in arrays
          .replace(/,\s*}/g, "}")  // Remove trailing commas in objects
          .replace(/'/g, '"');     // Replace single quotes with double quotes

        formFields = JSON.parse(jsonStr);

        // Validate and normalize each field
        for (var j = 0; j < formFields.length; j++) {
          var f = formFields[j];
          if (!f.label) f.label = "フィールド " + f.index;
          if (!f.type) f.type = "unknown";
          if (f.required === undefined) f.required = false;
          if (!f.options) f.options = [];
        }
        
        // Success - return the parsed fields
        return formFields;
      } else {
        lastError = "JSON配列が見つかりませんでした";
      }
    } catch (e) {
      lastError = String(e);
      console.log("AI form analysis attempt " + (retry + 1) + " failed: " + e);
    }
  }

  // All retries failed - use fallback
  console.error("AI form analysis failed after " + MAX_RETRIES + " retries: " + lastError);
  for (var k = 0; k < fieldsHtmlArray.length; k++) {
    formFields.push({
      index: fieldsHtmlArray[k].index,
      label: "フィールド " + fieldsHtmlArray[k].index,
      type: "unknown",
      required: false,
      options: [],
    });
  }

  return formFields;
}

function logFormFields(formStructure) {
  var fields = formStructure.fields;
  if (!fields || fields.length === 0) {
    console.log("    → フォームフィールドが見つかりませんでした");
    return;
  }

  for (var i = 0; i < fields.length; i++) {
    var f = fields[i];
    var typeIcon = {
      text: "📝",
      email: "✉️",
      date: "📅",
      radio: "🔘",
      checkbox: "☑️",
      textarea: "📄",
      unknown: "❓",
    };
    var icon = typeIcon[f.type] || "❓";
    var reqMark = f.required ? " *" : "";

    // オプションを文字列に変換（オブジェクトの場合は値を抽出）
    var optionsStr = "";
    if (f.options && f.options.length > 0) {
      var optionLabels = [];
      for (var j = 0; j < Math.min(f.options.length, 3); j++) {
        var opt = f.options[j];
        if (typeof opt === "string") {
          optionLabels.push(opt);
        } else if (opt && typeof opt === "object") {
          // オブジェクトの場合はlabel, value, textなどのプロパティを探す
          optionLabels.push(opt.label || opt.value || opt.text || String(opt));
        }
      }
      optionsStr =
        optionLabels.length > 0
          ? " [" +
            optionLabels.join(", ") +
            (f.options.length > 3 ? "..." : "") +
            "]"
          : "";
    }

    console.log(
      "    " +
        f.index +
        ". " +
        icon +
        " " +
        (f.label || "(ラベルなし)") +
        reqMark +
        optionsStr,
    );
  }
}

// =============================================================================
// FILL THE FORM
// =============================================================================

function fillForm(tabId, availableDates, timeSlots) {
  var firstDate = availableDates[0] || "";
  var secondDate = availableDates[1] || "";
  var timeSlot1 = (timeSlots && timeSlots[0]) || CONFIG.preferredTimeSlots[0];
  var timeSlot2 = (timeSlots && timeSlots[1]) || CONFIG.preferredTimeSlots[1] || timeSlot1;

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
      try {
        floorp.tabInput(tabId, "input[type='date']", firstDate);
      } catch (e2) {}
    }
  }

  // Select first time slot
  try {
    floorp.tabClick(
      tabId,
      "div[aria-label='" + timeSlot1 + "']",
    );
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
    "div[role='listitem']:nth-child(6) div[aria-label='" + timeSlot2 + "']",
    "div[role='listitem']:nth-child(6) div[role='radio']",
    "li:nth-child(6) div[role='radio']",
  ];
  for (var ti = 0; ti < timeSlotSelectors.length; ti++) {
    try {
      floorp.tabClick(tabId, timeSlotSelectors[ti]);
      break;
    } catch (e) {}
  }
}

// =============================================================================
// CALCULATE AVAILABLE DATES
// =============================================================================

function calculateAvailableDates(thunderbirdEvents, googleEvents) {
  var dateRange = generateDateRange(CONFIG.schedulingDaysLookAhead);
  var BUSINESS_START = 10;
  var BUSINESS_END = 19;
  var MIN_GAP_HOURS = 3; // Minimum gap between events to consider as available

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
    } else {
      console.log("[DEBUG] Google event missing date: " + gEvent.title + " time: " + gEvent.time);
    }
  }

  // Debug: Show busy times for upcoming days
  var debugDates = generateDateRange(7);
  for (var dd = 0; dd < debugDates.length; dd++) {
    var dDate = debugDates[dd];
    var dBusy = busyTimesByDate[dDate] || [];
    if (dBusy.length > 0) {
      console.log("[DEBUG] " + dDate + ": " + dBusy.length + " events");
      for (var de = 0; de < dBusy.length; de++) {
        console.log("  - " + dBusy[de].title + " (" + dBusy[de].start + ":00-" + dBusy[de].end + ":00)");
      }
    }
  }

  var availableSlots = [];

  for (var i = 0; i < dateRange.length; i++) {
    var dateStr = dateRange[i];
    var busyTimes = busyTimesByDate[dateStr] || [];

    // Sort busy times by start hour
    busyTimes.sort(function (a, b) {
      return a.start - b.start;
    });

    // Find gaps of MIN_GAP_HOURS or more within business hours
    var gaps = [];
    var currentTime = BUSINESS_START;

    for (var b = 0; b < busyTimes.length; b++) {
      var busy = busyTimes[b];
      // Only consider events within business hours
      var eventStart = Math.max(busy.start, BUSINESS_START);
      var eventEnd = Math.min(busy.end, BUSINESS_END);

      if (eventStart > currentTime) {
        // Found a gap before this event
        var gapDuration = eventStart - currentTime;
        if (gapDuration >= MIN_GAP_HOURS) {
          gaps.push({ start: currentTime, end: eventStart });
        }
      }
      // Move current time to end of this event (if within business hours)
      if (busy.end > currentTime) {
        currentTime = Math.max(currentTime, busy.end);
      }
    }

    // Check for gap after last event until end of business hours
    if (currentTime < BUSINESS_END) {
      var finalGap = BUSINESS_END - currentTime;
      if (finalGap >= MIN_GAP_HOURS) {
        gaps.push({ start: currentTime, end: BUSINESS_END });
      }
    }

    // Add available slots for this date
    for (var g = 0; g < gaps.length; g++) {
      var gap = gaps[g];
      // Find a matching preferred time slot within this gap
      for (var p = 0; p < CONFIG.preferredTimeSlots.length; p++) {
        var slot = CONFIG.preferredTimeSlots[p];
        var slotParts = slot.split("-");
        var slotStart = parseInt(slotParts[0].split(":")[0], 10);
        var slotEnd = parseInt(slotParts[1].split(":")[0], 10);

        if (slotStart >= gap.start && slotEnd <= gap.end) {
          availableSlots.push({
            date: dateStr,
            timeSlot: slot,
            gapStart: gap.start,
            gapEnd: gap.end,
          });
          break; // Only add first matching slot per gap
        }
      }
    }
  }

  // Return unique dates with their best time slots
  var result = [];
  var seenDates = {};
  for (var s = 0; s < availableSlots.length; s++) {
    var slotInfo = availableSlots[s];
    if (!seenDates[slotInfo.date]) {
      seenDates[slotInfo.date] = true;
      result.push(slotInfo.date);
    }
  }

  // Store the full slot info for later use
  calculateAvailableDates.lastSlots = availableSlots;

  return result;
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
    "&text=" +
    encodeURIComponent(title) +
    "&dates=" +
    startDateTime +
    "/" +
    endDateTime +
    "&details=" +
    encodeURIComponent("Floorp OS Automator により自動追加された予定です。");

  var calendarTabRaw = floorp.createTab(calendarUrl, false);
  var calendarTabId = calendarTabRaw;

  try {
    var parsed = JSON.parse(calendarTabRaw);
    if (parsed && parsed.instanceId) {
      calendarTabId = parsed.instanceId;
    }
  } catch (e) {}

  // Wait for page to load and click the save button
  try {
    floorp.tabWaitForNetworkIdle(calendarTabId);
    floorp.tabWaitForElement(calendarTabId, "#xSaveBu", 10000);
    floorp.tabClick(calendarTabId, "#xSaveBu");
    // Wait for save to complete
    floorp.tabWaitForNetworkIdle(calendarTabId);
  } catch (e) {
    // Save button click failed - user will need to save manually
  }

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
    return (
      isoMatch[1] +
      "-" +
      String(isoMatch[2]).padStart(2, "0") +
      "-" +
      String(isoMatch[3]).padStart(2, "0")
    );
  }

  var monthDayMatch = prompt.match(/(\d{1,2})月(\d{1,2})日/);
  if (monthDayMatch) {
    var currentYear = new Date().getFullYear();
    return (
      currentYear +
      "-" +
      String(monthDayMatch[1]).padStart(2, "0") +
      "-" +
      String(monthDayMatch[2]).padStart(2, "0")
    );
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

  var timeRangeMatch = prompt.match(
    /(\d{1,2}):(\d{2})\s*[-~〜－]\s*(\d{1,2}):(\d{2})/,
  );
  if (timeRangeMatch) {
    return (
      timeRangeMatch[1] +
      ":" +
      timeRangeMatch[2] +
      "-" +
      timeRangeMatch[3] +
      ":" +
      timeRangeMatch[4]
    );
  }

  var jpTimeMatch = prompt.match(/(\d{1,2})時\s*(から|ー|-)\s*(\d{1,2})時/);
  if (jpTimeMatch) {
    return jpTimeMatch[1] + ":00-" + jpTimeMatch[3] + ":00";
  }

  var singleTimeMatch = prompt.match(/(\d{1,2})時/);
  if (singleTimeMatch) {
    var hour = String(singleTimeMatch[1]).padStart(2, "0");
    return hour + ":00-" + hour + ":59";
  }

  return CONFIG.preferredTimeSlots[0] || "10:00-12:00";
}

function parseSecondDateFromPrompt(prompt, firstDate) {
  var secondMatch = prompt.match(/と\s*(\d{1,2})月(\d{1,2})日/);
  if (secondMatch) {
    var currentYear = new Date().getFullYear();
    return (
      currentYear +
      "-" +
      String(secondMatch[1]).padStart(2, "0") +
      "-" +
      String(secondMatch[2]).padStart(2, "0")
    );
  }

  var allDates = prompt.match(/(\d{4}-\d{2}-\d{2})/g);
  if (allDates && allDates.length >= 2) {
    return allDates[1];
  }

  var nextDay = new Date(firstDate);
  nextDay.setDate(nextDay.getDate() + 1);
  return nextDay.toISOString().split("T")[0];
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
    // If already have an instanceId, use it; otherwise attach using browserId
    if (formTab.instanceId) {
      formTabId = formTab.instanceId;
    } else {
      // attachToTab expects browserId and returns new instanceId
      var browserId = String(formTab.browserId || formTab.id);
      formTabId = floorp.attachToTab(browserId);
    }
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
  console.log("🚀 フォームの解析と入力を実行...");
  console.log("─".repeat(30));

  setupErrorHandlers();
  checkPluginAvailability();

  var TOTAL_STEPS = 6;

  // =========================================================================
  // STEP 1: Open and analyze the Google Form first
  // =========================================================================
  logStep(1, TOTAL_STEPS, "フォームを開いて構造を解析中...", "form");
  var formTabId = openOrCreateFormTab();
  var formStructure = analyzeFormStructure(formTabId);
  logResult("フィールド数", formStructure.totalFields + "個");
  logFormFields(formStructure);

  // =========================================================================
  // STEP 2: Get user identity from Thunderbird
  // =========================================================================
  logStep(2, TOTAL_STEPS, "Thunderbird ユーザー情報取得", "identity");
  var identity = getThunderbirdIdentity();
  logResult("名前", identity.name);
  logResult("Email", identity.email);

  // =========================================================================
  // STEP 3: Gather calendar events from Thunderbird
  // =========================================================================
  logStep(3, TOTAL_STEPS, "Thunderbird カレンダー取得", "calendar");
  var thunderbirdEvents = readThunderbirdCalendar();
  logResult("予定", thunderbirdEvents.length + "件");

  // =========================================================================
  // STEP 4: Activate Floorp and scrape Google Calendar
  // =========================================================================
  logStep(4, TOTAL_STEPS, "Google Calendar 取得", "google");
  try {
    sapphillon.thunderbird.activateFloorp();
  } catch (e) {}
  var googleEvents = scrapeGoogleCalendar();
  logResult("予定", googleEvents.length + "件");

  // =========================================================================
  // STEP 5: Calculate available dates and fill the form
  // =========================================================================
  logStep(5, TOTAL_STEPS, "空き日程計算・入力", "calculate");
  var availableDates = calculateAvailableDates(thunderbirdEvents, googleEvents);
  logResult(
    "空き",
    availableDates.slice(0, 2).join(", ") +
      (availableDates.length > 2 ? " +" + (availableDates.length - 2) : ""),
  );

  var firstDate = availableDates[0];
  var secondDate = availableDates[1];
  if (!firstDate) {
    var tomorrowDates = generateDateRange(2);
    firstDate = tomorrowDates[0];
    secondDate = tomorrowDates[1];
  }
  if (!secondDate) {
    secondDate = firstDate;
  }

  // Get the actual available time slots from lastSlots
  var lastSlots = calculateAvailableDates.lastSlots || [];
  var timeSlot1 = CONFIG.preferredTimeSlots[0];
  var timeSlot2 = CONFIG.preferredTimeSlots[1] || timeSlot1;
  var timeSlot1Found = false;
  var timeSlot2Found = false;

  // Find the correct time slot for each date
  for (var si = 0; si < lastSlots.length; si++) {
    if (lastSlots[si].date === firstDate && !timeSlot1Found) {
      timeSlot1 = lastSlots[si].timeSlot;
      timeSlot1Found = true;
    }
    if (lastSlots[si].date === secondDate && !timeSlot2Found) {
      timeSlot2 = lastSlots[si].timeSlot;
      timeSlot2Found = true;
    }
  }

  // Fill the form with correct dates and time slots
  fillForm(formTabId, [firstDate, secondDate], [timeSlot1, timeSlot2]);

  // =========================================================================
  // STEP 6: Add events to Google Calendar (first and second date)
  // =========================================================================
  logStep(6, TOTAL_STEPS, "Google Calendar 登録", "add");

  // Register first date
  var calendarTabId = addEventToGoogleCalendar(
    firstDate,
    timeSlot1,
    "病院の予約（第一希望）",
  );
  logResult("第一希望", firstDate + " " + timeSlot1);

  // Close the first calendar tab
  try {
    floorp.closeTab(calendarTabId);
  } catch (e) {
    // Silent cleanup
  }

  // Register second date
  addEventToGoogleCalendar(secondDate, timeSlot2, "病院の予約（第二希望）");
  logResult("第二希望", secondDate + " " + timeSlot2);

  console.log("─".repeat(30));
  console.log("✅ 完了");

  // Cleanup
  try {
    floorp.destroyTabInstance(formTabId);
  } catch (e) {
    // Silent cleanup
  }
  try {
    if (calendarTabId) {
      floorp.destroyTabInstance(calendarTabId);
    }
  } catch (e) {
    // Silent cleanup
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
  console.log("📅 予約登録: " + userPrompt);

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
    // Silent fallback
  }

  var calendarTabId = addEventToGoogleCalendar(
    firstDate,
    timeSlot,
    "病院の予約",
  );

  console.log("✅ 予約完了: " + firstDate + " " + timeSlot);

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
