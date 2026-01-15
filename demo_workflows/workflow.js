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

  // Find the first available date that's not busy
  var firstDate = availableDates[0] || new Date().toISOString().split("T")[0];
  var secondDate =
    availableDates[1] ||
    availableDates[0] ||
    new Date().toISOString().split("T")[0];

  console.log("Using dates: " + firstDate + ", " + secondDate);
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
    var emailInputs = floorp.tabGetElements(tabId, "input[type='email']");
    if (emailInputs && emailInputs.length > 0) {
      floorp.tabInput(tabId, "input[type='email']", CONFIG.userEmail);
    }
  } catch (e) {
    console.error("Failed to fill email: " + e);
  }

  // Fill first date
  console.log("Filling first date...");
  var dateInputs = floorp.tabGetElements(tabId, "input[type='date']");
  if (dateInputs && dateInputs.length >= 1) {
    try {
      floorp.tabInput(tabId, "input[type='date']", firstDate);
    } catch (e) {
      console.error("Failed to fill date: " + e);
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

  // Fill second date (if available)
  if (dateInputs && dateInputs.length >= 2) {
    console.log("Filling second date...");
    // Need to target the second date input specifically
  }

  // Fill remarks (textarea)
  console.log("Filling remarks...");
  try {
    var textareas = floorp.tabGetElements(tabId, "textarea");
    if (textareas && textareas.length > 0) {
      floorp.tabInput(
        tabId,
        "textarea",
        "Floorp OS Automator による自動入力テスト"
      );
    }
  } catch (e) {
    console.error("Failed to fill remarks: " + e);
  }

  console.log("Form filling complete!");
}

// =============================================================================
// STEP 5: CALCULATE AVAILABLE DATES
// =============================================================================

function calculateAvailableDates(thunderbirdEvents, googleEvents) {
  console.log("");
  console.log("=== Step 5: Calculating Available Dates ===");

  // Get all busy dates from both calendars
  var busyDates = {};

  for (var j = 0; j < thunderbirdEvents.length; j++) {
    var event = thunderbirdEvents[j];
    if (event.date) {
      busyDates[event.date] = true;
    }
  }

  console.log("Busy dates: " + Object.keys(busyDates).join(", "));

  // Generate next 14 days
  var availableDates = [];
  var today = new Date();

  for (var i = 1; i <= 14; i++) {
    var date = new Date(today);
    date.setDate(date.getDate() + i);
    var dateStr = date.toISOString().split("T")[0];

    // Skip weekends
    if (date.getDay() === 0 || date.getDay() === 6) continue;

    // Skip busy dates
    if (busyDates[dateStr]) continue;

    availableDates.push(dateStr);
  }

  console.log(
    "Available dates: " + availableDates.slice(0, 5).join(", ") + "..."
  );

  return availableDates;
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
  var formTabId = floorp.createTab(CONFIG.formUrl);
  floorp.tabWaitForNetworkIdle(formTabId);

  // Analyze form structure (get HTML first)
  var formData = analyzeFormStructure(formTabId);

  // Step 5: Fill the form
  fillForm(formTabId, availableDates);

  console.log("");
  console.log("=== Workflow Complete ===");
  console.log("Form is ready for review. Submit manually if correct.");

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

  // Cleanup: destroy the form tab instance
  console.log("");
  console.log("=== Cleanup ===");
  try {
    floorp.destroyTabInstance(formTabId);
    console.log("Tab instance destroyed successfully");
  } catch (e) {
    console.error("Failed to destroy tab instance: " + e);
  }

  return result;
}

// Run the workflow
workflow();
