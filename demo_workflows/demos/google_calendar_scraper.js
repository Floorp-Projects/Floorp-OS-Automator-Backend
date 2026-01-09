// Calendar Aggregation Demo - Google Calendar Scraper
// This workflow scrapes events from Google Calendar web interface
// Note: Requires user to be logged into Google Calendar in Floorp

// =============================================================================
// GOOGLE CALENDAR SCRAPING
// =============================================================================

/**
 * Scrapes events from Google Calendar web interface
 * Opens Google Calendar and extracts visible events
 *
 * Confirmed working selectors (tested 2026-01-09):
 * - [data-eventid]: Primary event identifier (works)
 * - [data-eventchip]: Visual event chip (works)
 * - Event elements are DIV with classes: vEJ0bc, ChfiMc, rFUW1c
 */
async function scrapeGoogleCalendar() {
  console.log("=== Scraping Google Calendar ===");

  // Create a new browser tab
  const tabId = await floorp.createTab(
    "https://calendar.google.com/calendar/u/0/r/week"
  );

  // Wait for the calendar to load
  console.log("Waiting for Google Calendar to load...");
  await floorp.waitForNetworkIdle(tabId);
  await floorp.wait(tabId, "[data-eventid]", 10000).catch(() => {
    console.log("Warning: No events found or calendar not loaded");
  });

  // Additional wait for dynamic content
  await new Promise((resolve) => setTimeout(resolve, 2000));

  const events = [];

  try {
    // Use confirmed working selector
    const elements = await floorp.getElements(tabId, "[data-eventid]");
    console.log(`Found ${elements ? elements.length : 0} event elements`);

    if (elements && elements.length > 0) {
      for (let i = 0; i < elements.length; i++) {
        try {
          // Get the event text content
          // The event title is typically inside the chip element
          const eventElement = elements[i];
          const eventText =
            eventElement.textContent || eventElement.innerText || "";

          if (eventText.trim()) {
            events.push({
              title: eventText.trim(),
              source: "google_calendar",
              index: i,
            });
          }
        } catch (e) {
          console.log(`Error extracting event ${i}:`, e.message);
        }
      }
    }

    // If direct text extraction failed, try clicking and reading
    if (events.length === 0 && elements && elements.length > 0) {
      console.log("Trying alternative text extraction...");
      for (let i = 0; i < Math.min(elements.length, 10); i++) {
        try {
          const text = await floorp.getText(
            tabId,
            `[data-eventid]:nth-child(${i + 1})`
          );
          if (text && text.trim()) {
            events.push({
              title: text.trim(),
              source: "google_calendar",
              index: i,
            });
          }
        } catch (e) {
          // Continue
        }
      }
    }
  } catch (error) {
    console.error("Error scraping Google Calendar:", error);
  }

  // Close the tab
  await floorp.closeTab(tabId);

  console.log(`Scraped ${events.length} events from Google Calendar`);
  return events;
}

// =============================================================================
// =============================================================================

/**
 * Tests various selectors on Google Calendar to find working ones
 * Run this to identify the correct selectors for the current Google Calendar version
 */
async function testGoogleCalendarSelectors() {
  console.log("=== Testing Google Calendar Selectors ===");

  const tabId = await floorp.createTab(
    "https://calendar.google.com/calendar/u/0/r/week"
  );

  console.log("Waiting for page load...");
  await floorp.waitForNetworkIdle(tabId);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  // List of selectors to test
  const selectorsToTest = [
    // Event-related selectors
    "[data-eventid]",
    "[data-eventchip]",
    "[role='button'][data-eventid]",

    // Grid and cell selectors
    "[role='gridcell']",
    "[role='row']",

    // Possible event classes (these may change with Google updates)
    ".WdTUxb",
    ".Jmftzc",
    ".NlL62b",
    ".FAxxKc",

    // Navigation elements (for reference)
    "[data-view='week']",
    "[data-view='month']",
    "[data-view='day']",
  ];

  console.log("\n--- Selector Test Results ---");

  for (const selector of selectorsToTest) {
    try {
      const elements = await floorp.getElements(tabId, selector);
      const count = elements ? elements.length : 0;
      const status = count > 0 ? "✅" : "❌";
      console.log(`${status} ${selector}: ${count} elements`);

      // If found, try to get text from first element
      if (count > 0) {
        try {
          const text = await floorp.getText(tabId, selector);
          if (text) {
            console.log(`   First element text: "${text.substring(0, 50)}..."`);
          }
        } catch (e) {
          // Text extraction failed
        }
      }
    } catch (error) {
      console.log(`❌ ${selector}: Error - ${error.message}`);
    }
  }

  // Take a screenshot for manual inspection
  console.log("\nCapturing screenshot for manual inspection...");
  // await floorp.screenshot(tabId, "/tmp/google_calendar_test.png");
  // console.log("Screenshot saved to /tmp/google_calendar_test.png");

  await floorp.closeTab(tabId);
  console.log("\n=== Selector Testing Complete ===");
}

// =============================================================================
// MAIN EXECUTION
// =============================================================================

async function main() {
  console.log("Google Calendar Scraper Demo");
  console.log("============================");
  console.log("");

  // First, run selector tests to identify working selectors
  await testGoogleCalendarSelectors();

  // Then try to scrape actual events
  console.log("\n");
  const events = await scrapeGoogleCalendar();

  console.log("\n--- Scraped Events ---");
  for (const event of events) {
    console.log(`📅 ${event.title}`);
  }

  return {
    events: events,
    total: events.length,
  };
}

// Run the workflow
const result = await main();
console.log("\n=== Workflow Complete ===");
console.log(JSON.stringify(result, null, 2));
