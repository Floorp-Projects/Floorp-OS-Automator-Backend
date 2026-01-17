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

  // Time slots to prefer
  preferredTimeSlots: [
    "10:00-12:00",
    "13:00-15:00",
    "15:00-17:00",
    "17:00-19:00",
  ],
};

// =============================================================================
// STEP 1: READ THUNDERBIRD CALENDAR (via Plugin API)
// =============================================================================

function readThunderbirdCalendar() {
  console.log("");
  console.log("=== Step 1: Reading Thunderbird Calendar ===");

  var events = [];

  try {
    // Use Thunderbird plugin API to get calendar events
    var rawEvents = thunderbird.getCalendarEvents(CONFIG.daysToCheck);
    console.log("Found " + rawEvents.length + " events in Thunderbird");
    console.log("");
    console.log("--- Thunderbird Calendar Events ---");

    for (var i = 0; i < rawEvents.length; i++) {
      var ev = rawEvents[i];
      events.push({
        title: ev.title,
        startTime: ev.start_time,
        endTime: ev.end_time,
        date: ev.date,
        source: "thunderbird",
      });
      console.log("");
      console.log("[Event " + (i + 1) + "]");
      console.log("  Title: " + ev.title);
      console.log("  Date: " + ev.date);
      console.log("  Start Time: " + ev.start_time);
      console.log("  End Time: " + ev.end_time);
    }
  } catch (error) {
    console.error("Error reading Thunderbird calendar:", error);
  }

  return events;
}

// =============================================================================
// STEP 1.5: GET USER IDENTITY FROM THUNDERBIRD
// =============================================================================

function getThunderbirdIdentity() {
  console.log("");
  console.log("=== Getting User Identity from Thunderbird ===");

  var identity = thunderbird.getIdentity();
  console.log("User: " + identity.name + " <" + identity.email + ">");
  console.log("Profile: " + identity.profile);

  CONFIG.userName = identity.name;
  CONFIG.userEmail = identity.email;

  return identity;
}

// =============================================================================
// STEP 2: SCRAPE GOOGLE CALENDAR
// =============================================================================

function scrapeGoogleCalendar() {
  console.log("");
  console.log("=== Step 2: Scraping Google Calendar ===");

  var tabId = floorp.createTab(
    "https://calendar.google.com/calendar/u/0/r/week"
  );

  console.log("Tab created: " + tabId);
  floorp.tabWaitForNetworkIdle(tabId);
  console.log("Network idle");

  // Wait for calendar grid to appear (indicates page is rendered)
  console.log("Waiting for calendar grid to render...");
  floorp.tabWaitForElement(tabId, "[data-view-heading]", 10000);
  console.log("Calendar grid found");

  // Wait for events to load (wait for event elements with data-eventid)
  console.log("Waiting for events to load...");
  floorp.tabWaitForElement(tabId, "[data-eventid]", 10000);
  console.log("Events loaded");

  // Debug: Check the page HTML length
  var pageHtml = floorp.tabHtml(tabId);
  console.log("Page HTML length: " + (pageHtml ? pageHtml.length : 0));

  // Check if we can find specific elements
  var bodyCheck = floorp.tabGetElements(tabId, "body");
  var bodyParsed = JSON.parse(bodyCheck);
  console.log(
    "Body elements found: " +
      (bodyParsed.elements ? bodyParsed.elements.length : 0)
  );

  var events = [];

  try {
    // Get all event buttons - the button text contains the full event details
    // Selector: [data-eventchip] button (event chip buttons in the calendar grid)
    var selector = "[data-eventchip]";
    console.log("Using selector: " + selector);

    var elementsJson = floorp.tabGetElements(tabId, selector);

    // Debug: show raw response
    console.log("Raw tabGetElements response type: " + typeof elementsJson);
    console.log(
      "Raw response length: " + (elementsJson ? elementsJson.length : "null")
    );

    // Parse the JSON response - tabGetElements returns {"elements": [...]}
    var parsed = JSON.parse(elementsJson);
    var elementHtmlList = parsed.elements || [];

    console.log("Parsed elements count: " + elementHtmlList.length);

    // Extract event info from each element's outerHTML
    console.log("");
    console.log("--- Google Calendar Events ---");

    for (var i = 0; i < elementHtmlList.length; i++) {
      var html = elementHtmlList[i];

      // Extract aria-label from button element - it contains full event details
      // Format: aria-label="午後1時～午後4時、「タイトル」、カレンダー: カレンダー名、場所: 場所、2026年 1月 11日"
      var ariaMatch = html.match(/aria-label="([^"]+)"/);
      var fullDetails = ariaMatch ? ariaMatch[1] : "";

      // If no aria-label, try to extract from span.XuJrye inside the element
      if (!fullDetails) {
        var spanMatch = html.match(/class="XuJrye"[^>]*>([^<]+)</);
        fullDetails = spanMatch ? spanMatch[1] : "";
      }

      // Skip empty elements
      if (!fullDetails) {
        console.log("Skipping element " + i + ": no text content found");
        continue;
      }

      // Skip non-event content (day headers, etc.)
      var skipPatterns = ["終日の予定", "件の予定", "予定はありません"];
      var shouldSkip = false;
      for (var s = 0; s < skipPatterns.length; s++) {
        if (fullDetails.indexOf(skipPatterns[s]) !== -1) {
          shouldSkip = true;
          break;
        }
      }
      if (shouldSkip) {
        continue;
      }

      // Parse fullDetails to extract event info
      // Format examples:
      // "終日、「成人の日」、カレンダー: 日本の祝日、2026年 1月 12日"
      // "午後1時～午後4時、「タイトル」、カレンダー: カレンダー名、場所: 場所、2026年 1月 11日"
      // "2026年 1月 11日 午後11時～2026年 1月 12日 午前4:30、「タイトル」、カレンダー: 名前、場所: 場所"
      var eventTime = "";
      var eventTitle = "";
      var eventDate = "";
      var calendarName = "";
      var location = "";

      // Extract title - always in Japanese quotes
      var titleMatch = fullDetails.match(/「([^」]+)」/);
      if (titleMatch) {
        eventTitle = titleMatch[1];
      }

      // Extract time - first part before the title (before first 「)
      var titleIndex = fullDetails.indexOf("「");
      if (titleIndex > 0) {
        var timePart = fullDetails.substring(0, titleIndex);
        // Remove trailing comma and whitespace
        eventTime = timePart.replace(/、\s*$/, "").trim();
      }

      // Look for date pattern (年 月 日) - get the last occurrence for end date
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

      // Look for calendar name
      var calMatch = fullDetails.match(/カレンダー:\s*([^、]+)/);
      if (calMatch) {
        calendarName = calMatch[1].trim();
      }

      // Look for location
      var locMatch = fullDetails.match(/場所:\s*([^、]+)/);
      if (locMatch) {
        location = locMatch[1].trim();
      }

      // Check for duplicates (multi-day events appear on each day)
      // Use title + time + calendar as unique key
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
      if (isDuplicate) {
        console.log("Skipping duplicate event: " + eventTitle);
        continue;
      }

      events.push({
        title: eventTitle,
        date: eventDate,
        time: eventTime,
        calendar: calendarName,
        location: location,
        fullDetails: fullDetails,
        source: "google_calendar",
        index: i,
      });

      // Output detailed event info
      console.log("");
      console.log("[Event " + events.length + "]");
      console.log("  Title: " + eventTitle);
      console.log("  Date: " + eventDate);
      console.log("  Time: " + eventTime);
      if (calendarName) {
        console.log("  Calendar: " + calendarName);
      }
      if (location) {
        console.log("  Location: " + location);
      }
    }
  } catch (error) {
    console.error("Error scraping Google Calendar: " + error);
  }

  floorp.closeTab(tabId);

  console.log("Scraped " + events.length + " events from Google Calendar");
  return events;
}

// =============================================================================
// STEP 3: ANALYZE FORM HTML STRUCTURE
// =============================================================================

function analyzeFormStructure(tabId) {
  console.log("");
  console.log("=== Step 3: Analyzing Form Structure ===");

  // Get the full HTML content of the form
  var htmlContent = floorp.tabHtml(tabId);

  console.log("Form HTML retrieved. Analyzing structure...");
  console.log("HTML length: " + htmlContent.length + " characters");

  // Extract form field information from HTML
  // Google Forms uses specific patterns we can detect:
  // - data-params contains field metadata
  // - aria-labelledby links to field labels
  // - role="listitem" for form sections

  var formAnalysis = {
    title: "",
    fields: [],
    submitButton: null,
  };

  // Find form title (usually in the header)
  var titleStart = htmlContent.indexOf('class="M7eMe');
  if (titleStart !== -1) {
    var titleEnd = htmlContent.indexOf("<", titleStart + 20);
    var titleSection = htmlContent.substring(titleStart, titleEnd);
    var gtPos = titleSection.lastIndexOf(">");
    if (gtPos !== -1) {
      formAnalysis.title = titleSection.substring(gtPos + 1);
      console.log("Form title: " + formAnalysis.title);
    }
  }

  // Count text input fields by searching for aria-labelledby
  var textInputCount = 0;
  var searchPos = 0;
  while (
    (searchPos = htmlContent.indexOf("aria-labelledby=", searchPos)) !== -1
  ) {
    textInputCount++;
    searchPos++;
  }
  console.log("Found " + textInputCount + " labeled fields");

  // Count date inputs
  var dateInputCount = 0;
  searchPos = 0;
  while ((searchPos = htmlContent.indexOf('type="date"', searchPos)) !== -1) {
    dateInputCount++;
    searchPos++;
  }
  console.log("Found " + dateInputCount + " date fields");

  // Count radio buttons
  var radioCount = 0;
  searchPos = 0;
  while ((searchPos = htmlContent.indexOf('role="radio"', searchPos)) !== -1) {
    radioCount++;
    searchPos++;
  }
  console.log("Found " + radioCount + " radio button options");

  // Count textareas
  var textareaCount = 0;
  searchPos = 0;
  while ((searchPos = htmlContent.indexOf("<textarea", searchPos)) !== -1) {
    textareaCount++;
    searchPos++;
  }
  console.log("Found " + textareaCount + " textarea fields");

  return {
    htmlContent: htmlContent,
    formAnalysis: formAnalysis,
    fieldCounts: {
      textInputs: textInputCount,
      dateInputs: dateInputCount,
      radioButtons: radioCount,
      textareas: textareaCount,
    },
  };
}

// =============================================================================
// STEP 4: FILL THE FORM
// =============================================================================

function fillForm(tabId, availableDates) {
  console.log("");
  console.log("=== Step 4: Filling the Form ===");

  // Find the first and second available dates
  var firstDate = availableDates[0] || "";
  var secondDate = availableDates[1] || "";

  if (!firstDate) {
    console.log("WARNING: No available dates found!");
    firstDate = "2026-01-16"; // Fallback
  }
  if (!secondDate) {
    console.log(
      "WARNING: Only one available date found, using same for second choice"
    );
    secondDate = firstDate;
  }

  console.log("");
  console.log("--- Selected Dates ---");
  console.log("  First choice:  " + firstDate);
  console.log("  Second choice: " + secondDate);
  console.log("");
  console.log(
    "Using identity: " + CONFIG.userName + " <" + CONFIG.userEmail + ">"
  );

  // Wait for form to be fully loaded
  floorp.tabWaitForElement(tabId, "form", 10000);

  // Google Forms uses input fields within form elements
  // Try to find all text inputs in the form
  var inputs = floorp.tabGetElements(tabId, "input[type='text']");
  console.log("Found " + (inputs ? inputs.length : 0) + " text inputs");

  // Fill name field (first text input typically)
  console.log("Filling name: " + CONFIG.userName);
  try {
    floorp.tabWaitForElement(tabId, "input[type='text']", 5000);
    floorp.tabInput(tabId, "input[type='text']", CONFIG.userName);
  } catch (e) {
    console.error("Failed to fill name: " + e);
  }

  // Fill email field (look for email input or second text input)
  console.log("Filling email: " + CONFIG.userEmail);
  try {
    floorp.tabInput(tabId, "input[type='email']", CONFIG.userEmail);
  } catch (e) {
    console.error("Failed to fill email: " + e);
  }

  // Fill first date
  console.log("Filling first date: " + firstDate);
  var dateInputsRaw = floorp.tabGetElements(tabId, "input[type='date']");
  var dateInputs = [];
  if (dateInputsRaw) {
    try {
      var parsed = JSON.parse(dateInputsRaw);
      if (parsed && parsed.elements) {
        dateInputs = parsed.elements;
      }
    } catch (e) {
      console.error("Failed to parse date inputs: " + e);
    }
  }
  console.log("Found " + dateInputs.length + " date inputs");

  if (dateInputs.length >= 1) {
    try {
      // Use nth-child selector for the first date input
      floorp.tabInput(tabId, "input[type='date']:nth-of-type(1)", firstDate);
      console.log("First date filled successfully");
    } catch (e) {
      console.error("Failed to fill first date: " + e);
      // Fallback: try without nth-of-type
      try {
        floorp.tabInput(tabId, "input[type='date']", firstDate);
      } catch (e2) {
        console.error("Fallback also failed: " + e2);
      }
    }
  }

  // Select first time slot
  console.log("Selecting first time slot...");
  try {
    floorp.tabClick(
      tabId,
      "div[aria-label='" + CONFIG.preferredTimeSlots[0] + "']"
    );
  } catch (e) {
    console.error("Failed to click time slot: " + e);
  }

  // Fill second date
  // Based on form structure analysis:
  // - listitem 1: Name
  // - listitem 2: Email
  // - listitem 3: First date (第一希望日)
  // - listitem 4: First time slot (第一希望日・希望時間帯)
  // - listitem 5: Second date (第二希望日)
  // - listitem 6: Second time slot (第二希望日・希望時間帯)
  // - listitem 7: Remarks (ご意見・要望)
  console.log("Filling second date: " + secondDate);
  if (dateInputs.length >= 2) {
    var secondDateFilled = false;

    // Use listitem:nth-child to target the 5th question (second date)
    var selectors = [
      "div[role='listitem']:nth-child(5) input[type='date']",
      "li:nth-child(5) input[type='date']",
      "div[role='list'] > div:nth-child(5) input[type='date']",
      "form > div > div > div:nth-child(5) input[type='date']",
    ];

    for (var si = 0; si < selectors.length; si++) {
      if (secondDateFilled) break;
      try {
        console.log("Trying selector: " + selectors[si]);
        floorp.tabClick(tabId, selectors[si]);
        floorp.tabInput(tabId, selectors[si], secondDate);
        console.log("Second date filled with selector: " + selectors[si]);
        secondDateFilled = true;
      } catch (e) {
        console.log("Selector failed: " + selectors[si]);
      }
    }

    if (!secondDateFilled) {
      console.log("WARNING: Could not fill second date with any selector");
    }
  } else {
    console.log(
      "WARNING: Less than 2 date inputs found (" +
        dateInputs.length +
        "), cannot fill second date"
    );
  }

  // Select second time slot
  // The second time slot radiogroup is in the 6th listitem
  console.log("Selecting second time slot...");
  var secondTimeSlotClicked = false;
  var timeSlotSelectors = [
    "div[role='listitem']:nth-child(6) div[role='radio']",
    "div[role='listitem']:nth-child(6) div[aria-label='" +
      CONFIG.preferredTimeSlots[0] +
      "']",
    "li:nth-child(6) div[role='radio']",
    "div[role='radiogroup']:nth-of-type(2) div[role='radio']",
  ];

  for (var ti = 0; ti < timeSlotSelectors.length; ti++) {
    if (secondTimeSlotClicked) break;
    try {
      console.log("Trying time slot selector: " + timeSlotSelectors[ti]);
      floorp.tabClick(tabId, timeSlotSelectors[ti]);
      console.log(
        "Second time slot clicked with selector: " + timeSlotSelectors[ti]
      );
      secondTimeSlotClicked = true;
    } catch (e) {
      console.log("Time slot selector failed: " + timeSlotSelectors[ti]);
    }
  }

  if (!secondTimeSlotClicked) {
    console.log("WARNING: Could not click second time slot with any selector");
  }

  console.log("Form filling complete!");
}

// =============================================================================
// STEP 5: CALCULATE AVAILABLE DATES
// =============================================================================

function calculateAvailableDates(thunderbirdEvents, googleEvents) {
  console.log("");
  console.log("=== Step 5: Calculating Available Dates ===");

  // Define the date range for scheduling (January 11-18, 2026)
  // Use string-based date handling to avoid timezone issues
  var dateRange = [
    "2026-01-11",
    "2026-01-12",
    "2026-01-13",
    "2026-01-14",
    "2026-01-15",
    "2026-01-16",
    "2026-01-17",
    "2026-01-18",
  ];

  console.log("Date range: 2026-01-11 to 2026-01-18");

  // Define business hours for availability check (10:00 - 17:00)
  var BUSINESS_START = 10; // 10:00 AM
  var BUSINESS_END = 17; // 5:00 PM

  console.log(
    "Business hours: " + BUSINESS_START + ":00 - " + BUSINESS_END + ":00"
  );

  // Helper function to parse Japanese time format to hours
  // Examples: "午後1時", "午前10:30", "午後4時30分"
  function parseJapaneseTime(timeStr) {
    if (!timeStr) return null;

    // Check if it's all-day event
    if (timeStr.indexOf("終日") !== -1) {
      return { start: 0, end: 24 };
    }

    // Extract time range (e.g., "午後1時～午後4時")
    var rangeMatch = timeStr.match(
      /(午前|午後)?(\d{1,2})(?:時|:)(\d{0,2})?\s*[～~－\-]\s*(午前|午後)?(\d{1,2})(?:時|:)?(\d{0,2})?/
    );
    if (rangeMatch) {
      var startPeriod = rangeMatch[1] || "午前";
      var startHour = parseInt(rangeMatch[2], 10);
      var endPeriod = rangeMatch[4] || startPeriod;
      var endHour = parseInt(rangeMatch[5], 10);

      // Convert to 24-hour format
      if (startPeriod === "午後" && startHour !== 12) startHour += 12;
      if (startPeriod === "午前" && startHour === 12) startHour = 0;
      if (endPeriod === "午後" && endHour !== 12) endHour += 12;
      if (endPeriod === "午前" && endHour === 12) endHour = 0;

      return { start: startHour, end: endHour };
    }

    // If no range, try single time
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

  // Helper function to parse Thunderbird time format (ISO format: 2026-01-16 18:00:00)
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

  // Build a map of date -> array of busy time ranges
  var busyTimesByDate = {};

  // Add Thunderbird events with time info
  console.log("");
  console.log("--- Thunderbird events with times ---");
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
      console.log(
        "  " +
          event.date +
          ": " +
          event.title +
          " (" +
          (timeRange
            ? timeRange.start + ":00-" + timeRange.end + ":00"
            : "all day") +
          ")"
      );
    }
  }

  // Add Google Calendar events with time info
  console.log("");
  console.log("--- Google Calendar events with times ---");
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
      console.log(
        "  " +
          gEvent.date +
          ": " +
          gEvent.title +
          " (" +
          (gTimeRange
            ? gTimeRange.start + ":00-" + gTimeRange.end + ":00"
            : "all day") +
          ")"
      );
    }
  }

  // Check each date in the range
  var availableDates = [];

  console.log("");
  console.log("--- Checking availability ---");

  for (var i = 0; i < dateRange.length; i++) {
    var dateStr = dateRange[i];

    // Parse date to check day of week (YYYY-MM-DD format)
    var parts = dateStr.split("-");
    var year = parseInt(parts[0], 10);
    var month = parseInt(parts[1], 10) - 1; // JavaScript months are 0-indexed
    var day = parseInt(parts[2], 10);
    var dateObj = new Date(year, month, day);
    var dayOfWeek = dateObj.getDay();

    // Include weekends as well (no longer skipping)
    var dayNames = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    var dayName = dayNames[dayOfWeek];

    // Check if business hours are free
    var busyTimes = busyTimesByDate[dateStr] || [];
    var isAvailable = true;

    for (var b = 0; b < busyTimes.length; b++) {
      var busy = busyTimes[b];
      // Check if busy time overlaps with business hours
      // Overlap exists if: busy.start < BUSINESS_END AND busy.end > BUSINESS_START
      if (busy.start < BUSINESS_END && busy.end > BUSINESS_START) {
        console.log(
          "  " +
            dateStr +
            " (" +
            dayName +
            "): BUSY during business hours (" +
            busy.title +
            " " +
            busy.start +
            ":00-" +
            busy.end +
            ":00)"
        );
        isAvailable = false;
        break;
      }
    }

    if (isAvailable) {
      console.log(
        "  " + dateStr + " (" + dayName + "): AVAILABLE (business hours free)"
      );
      availableDates.push(dateStr);
    }
  }

  console.log("");
  console.log("Available dates: " + availableDates.join(", "));

  return availableDates;
}

// =============================================================================
// STEP 6: ADD EVENT TO GOOGLE CALENDAR
// =============================================================================

function addEventToGoogleCalendar(date, timeSlot, title) {
  console.log("");
  console.log("=== Step 6: Adding Event to Google Calendar ===");
  console.log("Date: " + date);
  console.log("Time: " + timeSlot);
  console.log("Title: " + title);

  // Parse the time slot (e.g., "10:00-12:00")
  var timeParts = timeSlot.split("-");
  var startTime = timeParts[0].replace(":", ""); // "1000"
  var endTime = timeParts[1].replace(":", ""); // "1200"

  // Format date for Google Calendar URL (YYYYMMDD)
  var dateFormatted = date.replace(/-/g, ""); // "20260116"

  // Build start and end datetime strings (YYYYMMDDTHHmmss)
  var startDateTime = dateFormatted + "T" + startTime + "00";
  var endDateTime = dateFormatted + "T" + endTime + "00";

  console.log("Start: " + startDateTime);
  console.log("End: " + endDateTime);

  // Build Google Calendar URL with parameters
  // Format: https://calendar.google.com/calendar/render?action=TEMPLATE&text=TITLE&dates=START/END
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

  console.log("Opening calendar URL...");

  var calendarTabRaw = floorp.createTab(calendarUrl);
  console.log("Calendar tab created (raw): " + calendarTabRaw);

  // Parse the returned JSON to get the instanceId
  var calendarTabId = calendarTabRaw;
  try {
    var parsed = JSON.parse(calendarTabRaw);
    if (parsed && parsed.instanceId) {
      calendarTabId = parsed.instanceId;
      console.log("Parsed instanceId: " + calendarTabId);
    }
  } catch (e) {
    console.log("Could not parse tab response, using raw value");
  }

  // Wait for page to load
  console.log("Waiting for page to load...");
  try {
    floorp.tabWaitForElement(calendarTabId, "#xSaveBu", 15000);
    console.log("Save button found");

    // Click the save button
    console.log("Clicking save button...");
    floorp.tabClick(calendarTabId, "#xSaveBu");
    console.log("Save button clicked - event saved!");
  } catch (e) {
    console.log("Could not auto-save: " + e);
    console.log("Please click 'Save' button manually.");
  }

  console.log("");
  console.log("Calendar event creation completed.");

  return calendarTabId;
}

// =============================================================================
// MAIN EXECUTION
// =============================================================================

function workflow() {
  console.log("========================================");
  console.log("  Calendar Aggregation Demo");
  console.log("  Thunderbird + Google Calendar -> Form");
  console.log("========================================");

  // Step 0: Get user identity from Thunderbird
  var identity = getThunderbirdIdentity();

  // Step 1: Read Thunderbird calendar
  var thunderbirdEvents = readThunderbirdCalendar();

  // Step 2: Scrape Google Calendar
  var googleEvents = scrapeGoogleCalendar();

  // Step 3: Calculate available dates
  var availableDates = calculateAvailableDates(thunderbirdEvents, googleEvents);

  // Step 4: Open Google Form and analyze structure
  console.log("");
  console.log("=== Opening Google Form ===");
  var formTabId = null;
  var createdFormTab = false;
  var formTab = null;
  var calendarTabId = null;

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
  } catch (e) {
    console.log("Failed to scan existing tabs: " + e);
  }

  if (formTab) {
    formTabId = String(formTab.instance_id || formTab.id);
    floorp.attachToTab(formTabId);
    console.log("Using existing form tab: " + (formTab.title || formTabId));
  } else {
    formTabId = floorp.createTab(CONFIG.formUrl);
    createdFormTab = true;
  }

  floorp.tabWaitForNetworkIdle(formTabId);

  // Analyze form structure (get HTML first)
  var formData = analyzeFormStructure(formTabId);

  // Step 5: Fill the form
  fillForm(formTabId, availableDates);

  // Get the selected dates for calendar
  var firstDate = availableDates[0] || "2026-01-16";
  var secondDate = availableDates[1] || firstDate;
  var timeSlot = CONFIG.preferredTimeSlots[0]; // "10:00-12:00"

  // Step 6: Add the appointment to Google Calendar
  console.log("");
  console.log("=== Adding Appointment to Google Calendar ===");
  calendarTabId = addEventToGoogleCalendar(firstDate, timeSlot, "病院の予約");

  console.log("");
  console.log("=== Workflow Complete ===");
  console.log("Form is ready for review. Submit manually if correct.");
  console.log("Calendar event is ready to save.");

  var result = {
    identity: identity ? { name: identity.name, email: identity.email } : null,
    thunderbirdEvents: thunderbirdEvents.length,
    googleEvents: googleEvents.length,
    availableDates: availableDates.slice(0, 5),
    formAnalysis: formData.fieldCounts,
  };

  console.log("");
  console.log("=== Summary ===");
  console.log(JSON.stringify(result));

  // Cleanup: destroy the tab instances
  console.log("");
  console.log("=== Cleanup ===");
  try {
    floorp.destroyTabInstance(formTabId);
    console.log("Form tab instance destroyed successfully");
  } catch (e) {
    console.error("Failed to destroy form tab instance: " + e);
  }
  try {
    if (calendarTabId) {
      floorp.destroyTabInstance(calendarTabId);
      console.log("Calendar tab instance destroyed successfully");
    }
  } catch (e) {
    console.error("Failed to destroy calendar tab instance: " + e);
  }

  return result;
}

// Run the workflow
workflow();
