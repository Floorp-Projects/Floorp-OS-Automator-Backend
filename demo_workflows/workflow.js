// Calendar Aggregation Demo - Combined Workflow
// This workflow:
// 1. Reads calendar events from Thunderbird (via Plugin API)
// 2. Scrapes events from Google Calendar (web)
// 3. Opens Google Form and fills it with available dates

// =============================================================================
// CONFIGURATION
// =============================================================================

var CONFIG = {
  formUrl:
    "https://docs.google.com/forms/d/e/1FAIpQLSfcJRyBQFYcS6sbFYaWfT1GMsk411I-Bl1ODE7oQSxpYh3nDg/viewform",
  userName: "テスト太郎",
  userEmail: "test@example.com",
  daysToCheck: 14,
  preferredTimeSlots: [
    "10:00-12:00",
    "13:00-15:00",
    "15:00-17:00",
    "17:00-19:00",
  ],
};

// =============================================================================
// MAIN WORKFLOW
// =============================================================================

function workflow() {
  console.log("========================================");
  console.log("  Calendar Aggregation Demo");
  console.log("  Thunderbird + Google Calendar -> Form");
  console.log("========================================");
  console.log("");

  var thunderbirdEvents = [];
  var googleEvents = [];
  var availableDates = [];

  // =========================================================================
  // STEP 1: Read Thunderbird Calendar
  // =========================================================================
  console.log("=== Step 1: Reading Thunderbird Calendar ===");
  try {
    var rawEvents = thunderbird.getCalendarEvents(CONFIG.daysToCheck);
    console.log("Found " + rawEvents.length + " events in Thunderbird");

    for (var i = 0; i < rawEvents.length; i++) {
      var ev = rawEvents[i];
      thunderbirdEvents.push({
        title: ev.title,
        date: ev.date,
        source: "thunderbird",
      });
      console.log("  - " + ev.title + " (" + ev.date + ")");
    }
  } catch (e) {
    console.log("Error reading Thunderbird calendar: " + e);
  }
  console.log("");

  // =========================================================================
  // STEP 2: Scrape Google Calendar
  // =========================================================================
  console.log("=== Step 2: Scraping Google Calendar ===");
  var gcalTabId = null;
  try {
    var gcalResponse = floorp.createTab(
      "https://calendar.google.com/calendar/u/0/r/week",
      false
    );
    // Parse JSON response to get instanceId
    try {
      var gcalData = JSON.parse(gcalResponse);
      gcalTabId = gcalData.instanceId || gcalData.id;
    } catch (parseErr) {
      // If not JSON, use as-is
      gcalTabId = gcalResponse;
    }
    console.log("Opened Google Calendar tab: " + gcalTabId);

    // Wait for calendar grid to load
    try {
      floorp.tabWaitForElement(gcalTabId, '[role="grid"]', 10000);
      console.log("Calendar grid detected");
    } catch (e) {
      console.log("Warning: Calendar grid not found or timeout");
    }

    // Wait for network idle
    try {
      floorp.tabWaitForNetworkIdle(gcalTabId, 3000);
    } catch (e) {
      // Continue even if timeout
    }

    // Get event count
    console.log("Google Calendar loaded");

    // Get HTML for analysis
    var gcalHtml = "";
    try {
      gcalHtml = floorp.tabHtml(gcalTabId);
      console.log("GCal HTML length: " + gcalHtml.length);

      // Look for events using data-eventid AND data-eventchip (more reliable)
      // Pattern: data-eventchip="" data-eventid="xxx" or data-eventid="xxx" data-eventchip=""
      var eventChipMatches = gcalHtml.match(
        /data-eventchip[^>]*data-eventid="([^"]+)"/g
      );
      if (!eventChipMatches) {
        // Try alternative order
        eventChipMatches = gcalHtml.match(
          /data-eventid="([^"]+)"[^>]*data-eventchip/g
        );
      }
      console.log(
        "Found " +
          (eventChipMatches ? eventChipMatches.length : 0) +
          " event chips (data-eventchip+data-eventid)"
      );

      // Also try simple data-eventid for fallback
      var eventIdMatches = gcalHtml.match(/data-eventid="([^"]+)"/g);
      console.log(
        "Found " +
          (eventIdMatches ? eventIdMatches.length : 0) +
          " total data-eventid markers"
      );

      // Extract event text from the pattern: >EventTitle< or innerText
      // GCal format: 午前9時～午後12時、「Project Planning」、カレンダー: ...
      // Look for patterns like 「xxx」 which indicate event titles
      var titleMatches = gcalHtml.match(/「([^」]+)」/g);
      if (titleMatches) {
        console.log(
          "Found " + titleMatches.length + " event titles (「xxx」 format):"
        );
        for (var t = 0; t < Math.min(10, titleMatches.length); t++) {
          console.log("  " + titleMatches[t]);
          googleEvents.push({ title: titleMatches[t].replace(/[「」]/g, "") });
        }
      }

      // Also look for time patterns like 午前9時～午後12時
      var jpTimeMatches = gcalHtml.match(
        /[午前後]+\d{1,2}時[～~\-][午前後]+\d{1,2}時/g
      );
      if (jpTimeMatches) {
        console.log("Found " + jpTimeMatches.length + " JP time patterns");
      }

      // Fallback: check for standard time patterns
      var timeMatches = gcalHtml.match(
        /\d{1,2}:\d{2}\s*[-~～]\s*\d{1,2}:\d{2}/g
      );
      if (timeMatches) {
        console.log(
          "Found " + timeMatches.length + " time ranges (e.g. 10:00-11:00)"
        );
      }
    } catch (htmlErr) {
      console.log("Failed to get GCal HTML: " + htmlErr);
    }
  } catch (e) {
    console.log("Error scraping Google Calendar: " + e);
  } finally {
    if (gcalTabId) {
      try {
        floorp.destroyTabInstance(gcalTabId);
        console.log("Closed Google Calendar tab");
      } catch (closeErr) {
        console.log("Note: Tab may have already closed: " + closeErr);
      }
    }
  }
  console.log("");

  // =========================================================================
  // STEP 3: Calculate Available Dates
  // =========================================================================
  console.log("=== Step 3: Calculating Available Dates ===");
  var busyDates = {};

  for (var j = 0; j < thunderbirdEvents.length; j++) {
    var event = thunderbirdEvents[j];
    if (event.date) {
      busyDates[event.date] = true;
    }
  }

  var busyDatesList = Object.keys(busyDates);
  console.log("Busy dates from Thunderbird: " + busyDatesList.join(", "));

  // Generate next 14 days
  var today = new Date();
  for (var k = 1; k <= CONFIG.daysToCheck; k++) {
    var date = new Date(today);
    date.setDate(date.getDate() + k);
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
  console.log("");

  // =========================================================================
  // STEP 4: Analyze and Fill Form (Deterministic Approach)
  // =========================================================================
  console.log("=== Step 4: Form Analysis & Input ===");
  var formTabId = null;
  try {
    var formResponse = floorp.createTab(CONFIG.formUrl, false);
    try {
      var formData = JSON.parse(formResponse);
      formTabId = formData.instanceId || formData.id;
    } catch (parseErr) {
      formTabId = formResponse;
    }
    console.log("Opened Google Form tab: " + formTabId);

    // Wait for form to load
    try {
      floorp.tabWaitForNetworkIdle(formTabId, 5000);
      // Explicitly wait for the form items to be rendered
      console.log("Waiting for form items to render...");
      floorp.tabWaitForElement(formTabId, 'div[role="listitem"]', 10000);
    } catch (e) {
      console.log("Wait warning: " + e);
    }

    console.log("Google Form loaded, retrieving HTML...");

    // Get form HTML
    var formHtml = "";
    try {
      formHtml = floorp.tabHtml(formTabId);
      console.log("Retrieved form HTML (" + formHtml.length + " chars)");
    } catch (e) {
      console.log("Failed to get form HTML: " + e);
    }

    if (formHtml) {
      // Debug: Check for listitem string
      var debugIndex = formHtml.indexOf("listitem");
      console.log("'listitem' found at: " + debugIndex);
      if (debugIndex !== -1) {
        console.log(
          "Context: " +
            formHtml.substring(Math.max(0, debugIndex - 20), debugIndex + 30)
        );
      } else {
        console.log("Warn: 'listitem' NOT FOUND in HTML");
      }

      // Parse Form Structure
      var formFields = parseGoogleForm(formHtml);
      console.log("Found " + formFields.length + " question blocks");

      // Process each field
      for (var i = 0; i < formFields.length; i++) {
        var field = formFields[i];
        console.log(
          "Field " +
            (i + 1) +
            ": [" +
            field.type +
            "] " +
            field.question.substring(0, 30) +
            "..."
        );

        var action = null; // { type: 'input'|'click', selector, value }

        // MAPPING LOGIC
        if (
          field.question.indexOf("名前") !== -1 ||
          field.question.indexOf("Name") !== -1
        ) {
          if (field.type === "text") {
            action = {
              type: "input",
              selector: field.selector,
              value: CONFIG.userName,
            };
          }
        } else if (
          field.question.indexOf("メール") !== -1 ||
          field.question.indexOf("Email") !== -1
        ) {
          if (field.type === "text") {
            action = {
              type: "input",
              selector: field.selector,
              value: CONFIG.userEmail,
            };
          }
        } else if (
          field.question.indexOf("1") !== -1 ||
          field.question.indexOf("１") !== -1 ||
          field.question.indexOf("First") !== -1
        ) {
          // First Date
          if (availableDates.length > 0) {
            if (field.type === "date") {
              action = {
                type: "input",
                selector: field.selector,
                value: availableDates[0],
              };
            } else if (field.type === "text") {
              action = {
                type: "input",
                selector: field.selector,
                value: availableDates[0],
              };
            }
          }
        } else if (
          field.question.indexOf("2") !== -1 ||
          field.question.indexOf("２") !== -1 ||
          field.question.indexOf("Second") !== -1
        ) {
          // Second Date
          var dateVal = availableDates[1] || availableDates[0];
          if (availableDates.length > 0) {
            if (field.type === "date") {
              action = {
                type: "input",
                selector: field.selector,
                value: dateVal,
              };
            } else if (field.type === "text") {
              action = {
                type: "input",
                selector: field.selector,
                value: dateVal,
              };
            }
          }
        } else if (
          field.question.indexOf("時間") !== -1 ||
          field.question.indexOf("Time") !== -1
        ) {
          // Time Slot (Radio)
          if (field.type === "radio") {
            // Find matching option
            for (var t = 0; t < CONFIG.preferredTimeSlots.length; t++) {
              var pref = CONFIG.preferredTimeSlots[t];
              // Find option selector that matches preference
              for (var op = 0; op < field.options.length; op++) {
                if (field.options[op].label.indexOf(pref) !== -1) {
                  action = {
                    type: "click",
                    selector: field.options[op].selector,
                  };
                  break;
                }
              }
              if (action) break;
            }
            // Default to first option if no match
            if (!action && field.options.length > 0) {
              action = { type: "click", selector: field.options[0].selector };
            }
          }
        } else if (field.type === "text" || field.type === "textarea") {
          // Comment / Other
          action = {
            type: "input",
            selector: field.selector,
            value: "候補日: " + availableDates.slice(0, 3).join(", "),
          };
        }

        // EXECUTE ACTION
        if (action) {
          try {
            if (action.type === "input") {
              console.log("  -> Input: " + action.value);
              if (action.selector) {
                // First, explicitly focus the element to trigger Google Form's "input mode"
                try {
                  floorp.tabFocus(formTabId, action.selector);
                  console.log("    tabFocus succeeded");
                } catch (e) {
                  console.log("    tabFocus failed: " + e);
                }

                // Then click to reinforce focus
                try {
                  floorp.tabClick(formTabId, action.selector);
                } catch (e) {
                  console.log("    Click failed: " + e);
                }

                try {
                  // Use tabInput with typingMode=true for character-by-character input
                  // This properly triggers Google Form's validation
                  floorp.tabInput(
                    formTabId,
                    action.selector,
                    action.value,
                    true
                  );
                  console.log("    tabInput with typingMode succeeded");
                } catch (e) {
                  console.log("    tabInput failed: " + e);
                  // Fallback to tabFillForm
                  try {
                    floorp.tabFillForm(
                      formTabId,
                      action.selector,
                      action.value
                    );
                    floorp.tabDispatchEvent(
                      formTabId,
                      action.selector,
                      "input"
                    );
                    floorp.tabDispatchEvent(
                      formTabId,
                      action.selector,
                      "change"
                    );
                  } catch (e2) {
                    console.log("    Fallback tabFillForm also failed: " + e2);
                  }
                }
              }
            } else if (action.type === "click") {
              console.log("  -> Click");
              floorp.tabClick(formTabId, action.selector);
            }
          } catch (actErr) {
            console.log("  Failed to execute action wrapper: " + actErr);
          }
        } else {
          console.log("  -> No matching action found");
        }
      }
    } else {
      console.log("No form HTML - skipping analysis");
    }

    console.log("");
    console.log("Available dates:");
    for (var m = 0; m < Math.min(5, availableDates.length); m++) {
      console.log("  " + (m + 1) + ". " + availableDates[m]);
    }
    // Close the form tab
    if (formTabId) {
      try {
        floorp.destroyTabInstance(formTabId);
        console.log("Closed Google Form tab");
      } catch (closeErr) {
        console.log("Failed to close form tab: " + closeErr);
      }
    }
  } catch (e) {
    console.log("Error with Google Form: " + e);
    if (formTabId) {
      try {
        floorp.destroyTabInstance(formTabId);
      } catch (err) {}
    }
  }
  console.log("");

  // =========================================================================
  // STEP 5: Summary
  // =========================================================================
  console.log("=== Workflow Complete ===");
  console.log("Thunderbird events: " + thunderbirdEvents.length);
  console.log("Available dates: " + availableDates.length);
  console.log("Google Form is open for review.");

  return {
    ok: true,
    thunderbirdEvents: thunderbirdEvents.length,
    availableDates: availableDates.slice(0, 5),
    formOpened: formTabId ? true : false,
  };
}

function parseGoogleForm(html) {
  var fields = [];

  // Debug regex match
  var regex = /role.{1,5}listitem/gi;
  var matches = html.match(regex);
  console.log(
    "Regex match check: " + (matches ? matches.length : 0) + " matches found"
  );
  if (matches && matches.length > 0) {
    console.log("First match example: '" + matches[0] + "'");
  }

  // Split by role="listitem" using relaxed regex
  var chunks = html.split(regex);

  // chunks[0] is content before the first listitem
  for (var i = 1; i < chunks.length; i++) {
    var chunk = chunks[i];
    var question = "";

    // Extract Question Text
    // Remove scripts and styles to avoid parsing noise
    var cleanChunk = chunk
      .replace(/<script[^>]*>[\s\S]*?<\/script>/gi, "")
      .replace(/<style[^>]*>[\s\S]*?<\/style>/gi, "");
    // Remove tags and extra spaces
    var textContent = cleanChunk
      .replace(/<[^>]+>/g, " ")
      .replace(/\s+/g, " ")
      .trim();
    question = textContent.substring(0, 100);

    var type = "text";
    var options = [];
    var baseSelector = 'div[role="listitem"]:nth-of-type(' + i + ")";

    // Determine item type
    if (chunk.indexOf('type="date"') !== -1) {
      type = "date";
    } else if (
      chunk.indexOf('role="radio"') !== -1 ||
      chunk.indexOf('role="radiogroup"') !== -1
    ) {
      type = "radio";
      // Extract options
      var radioMatches = chunk.match(/role="radio"[^>]*aria-label="([^"]+)"/g);
      if (radioMatches) {
        for (var r = 0; r < radioMatches.length; r++) {
          var m = radioMatches[r].match(/aria-label="([^"]+)"/);
          if (m) {
            options.push({
              label: m[1],
              selector:
                baseSelector +
                ' div[role="radio"][aria-label="' +
                m[1].replace(/"/g, '\\"') +
                '"]',
            });
          }
        }
      }
    } else if (
      chunk.indexOf('role="checkbox"') !== -1 ||
      chunk.indexOf('role="listbox"') !== -1
    ) {
      type = "radio"; // Treat as radio for simplicity
    } else if (chunk.indexOf("<textarea") !== -1) {
      type = "textarea";
    } else if (chunk.indexOf("<input") === -1) {
      type = "static";
    }

    if (type !== "static") {
      var itemSelector = baseSelector + ' input:not([type="hidden"])';
      if (type === "date") itemSelector = baseSelector + ' input[type="date"]';
      else if (type === "radio")
        itemSelector = baseSelector + ' div[role="radio"]';
      else if (type === "textarea") itemSelector = baseSelector + " textarea";

      fields.push({
        question: question,
        type: type,
        selector: itemSelector,
        options: options,
      });
    }
  }
  return fields;
}
