// Calendar Aggregation Demo - Combined Workflow
// This workflow:
// 1. Reads calendar events from Thunderbird (local SQLite)
// 2. Scrapes events from Google Calendar (web)
// 3. Analyzes Google Form HTML structure
// 4. Fills the form with available dates

// =============================================================================
// CONFIGURATION
// =============================================================================

const CONFIG = {
  // Google Form URL
  formUrl:
    "https://docs.google.com/forms/d/e/1FAIpQLSfcJRyBQFYcS6sbFYaWfT1GMsk411I-Bl1ODE7oQSxpYh3nDg/viewform",

  // Thunderbird profile path (macOS)
  thunderbirdProfile: "awq9kwrc.default-release",

  // User info for form
  userName: "テスト太郎",
  userEmail: "test@example.com",

  // Time slots to prefer
  preferredTimeSlots: [
    "10:00-12:00",
    "13:00-15:00",
    "15:00-17:00",
    "17:00-19:00",
  ],
};

// =============================================================================
// STEP 1: READ THUNDERBIRD CALENDAR
// =============================================================================

async function readThunderbirdCalendar() {
  console.log("\n=== Step 1: Reading Thunderbird Calendar ===");

  const profilePath = `~/Library/Thunderbird/Profiles/${CONFIG.thunderbirdProfile}`;
  const dbPath = `${profilePath}/calendar-data/cache.sqlite`;
  const tempDb = "/tmp/thunderbird_calendar_temp.sqlite";

  try {
    // Copy database to avoid lock issues
    exec(`cp ${dbPath} ${tempDb}`);

    // Query upcoming events (next 14 days)
    const now = Date.now() * 1000;
    const fourteenDaysLater = now + 14 * 24 * 60 * 60 * 1000000;

    const query = `
      SELECT
        title,
        datetime(event_start/1000000, 'unixepoch', 'localtime') as start_time,
        datetime(event_end/1000000, 'unixepoch', 'localtime') as end_time
      FROM cal_events
      WHERE event_start >= ${now} AND event_start <= ${fourteenDaysLater}
      ORDER BY event_start ASC;
    `;

    const result = exec(`sqlite3 -separator '|' ${tempDb} "${query}"`);

    const events = [];
    const lines = result
      .trim()
      .split("\n")
      .filter((line) => line.length > 0);

    for (const line of lines) {
      const [title, startTime, endTime] = line.split("|");
      events.push({
        title,
        startTime,
        endTime,
        date: startTime.split(" ")[0],
        source: "thunderbird",
      });
    }

    exec(`rm -f ${tempDb}`);

    console.log(`Found ${events.length} events in Thunderbird`);
    return events;
  } catch (error) {
    console.error("Error reading Thunderbird:", error);
    return [];
  }
}

// =============================================================================
// STEP 2: SCRAPE GOOGLE CALENDAR
// =============================================================================

async function scrapeGoogleCalendar() {
  console.log("\n=== Step 2: Scraping Google Calendar ===");

  const tabId = await floorp.createTab(
    "https://calendar.google.com/calendar/u/0/r/week"
  );

  await floorp.waitForNetworkIdle(tabId);
  await new Promise((resolve) => setTimeout(resolve, 3000));

  const events = [];

  try {
    const elements = await floorp.getElements(tabId, "[data-eventid]");
    console.log(
      `Found ${elements ? elements.length : 0} events on Google Calendar`
    );

    // Note: Getting text from Google Calendar events requires additional parsing
    // For demo purposes, we'll use basic detection

    if (elements && elements.length > 0) {
      for (let i = 0; i < elements.length; i++) {
        events.push({
          title: `Google Event ${i + 1}`,
          source: "google_calendar",
          index: i,
        });
      }
    }
  } catch (error) {
    console.error("Error scraping Google Calendar:", error);
  }

  await floorp.closeTab(tabId);

  console.log(`Scraped ${events.length} events from Google Calendar`);
  return events;
}

// =============================================================================
// STEP 3: ANALYZE FORM HTML STRUCTURE
// =============================================================================

async function analyzeFormStructure(tabId) {
  console.log("\n=== Step 3: Analyzing Form Structure ===");

  // Get the full HTML content of the form
  const htmlContent = await floorp.getContent(tabId);

  console.log("Form HTML retrieved. Analyzing structure...");
  console.log(`HTML length: ${htmlContent.length} characters`);

  // Extract form field information from HTML
  // Google Forms uses specific patterns we can detect:
  // - data-params contains field metadata
  // - aria-labelledby links to field labels
  // - role="listitem" for form sections

  const formAnalysis = {
    title: "",
    fields: [],
    submitButton: null,
  };

  // Find form title (usually in the header)
  const titleMatch = htmlContent.match(
    /<div[^>]*class="[^"]*M7eMe[^"]*"[^>]*>([^<]+)<\/div>/
  );
  if (titleMatch) {
    formAnalysis.title = titleMatch[1];
    console.log(`Form title: ${formAnalysis.title}`);
  }

  // Find text input fields
  const textInputs =
    htmlContent.match(/aria-labelledby=['"]([^'"]+)['"]/g) || [];
  console.log(`Found ${textInputs.length} labeled fields`);

  // Find date inputs
  const dateInputs = htmlContent.match(/type=['"]date['"]/g) || [];
  console.log(`Found ${dateInputs.length} date fields`);

  // Find radio buttons (time slots)
  const radioButtons = htmlContent.match(/role=['"]radio['"]/g) || [];
  console.log(`Found ${radioButtons.length} radio button options`);

  // Find textareas
  const textareas = htmlContent.match(/<textarea/g) || [];
  console.log(`Found ${textareas.length} textarea fields`);

  return {
    htmlContent,
    formAnalysis,
    fieldCounts: {
      textInputs: textInputs.length,
      dateInputs: dateInputs.length,
      radioButtons: radioButtons.length,
      textareas: textareas.length,
    },
  };
}

// =============================================================================
// STEP 4: FILL THE FORM
// =============================================================================

async function fillForm(tabId, availableDates) {
  console.log("\n=== Step 4: Filling the Form ===");

  // Find the first available date that's not busy
  const firstDate = availableDates[0] || new Date().toISOString().split("T")[0];
  const secondDate =
    availableDates[1] ||
    availableDates[0] ||
    new Date().toISOString().split("T")[0];

  console.log(`Using dates: ${firstDate}, ${secondDate}`);

  // Fill name field
  console.log("Filling name...");
  await floorp.type(tabId, "input[aria-labelledby='i1']", CONFIG.userName);

  // Fill email field
  console.log("Filling email...");
  await floorp.type(tabId, "input[aria-labelledby='i5']", CONFIG.userEmail);

  // Fill first date
  console.log("Filling first date...");
  const dateInputs = await floorp.getElements(tabId, "input[type='date']");
  if (dateInputs && dateInputs.length >= 1) {
    await floorp.type(tabId, "input[type='date']", firstDate);
  }

  // Select first time slot
  console.log("Selecting first time slot...");
  await floorp.click(
    tabId,
    `div[aria-label='${CONFIG.preferredTimeSlots[0]}']`
  );

  // Fill second date (if available)
  if (dateInputs && dateInputs.length >= 2) {
    console.log("Filling second date...");
    // Need to target the second date input specifically
  }

  // Fill remarks
  console.log("Filling remarks...");
  await floorp.type(
    tabId,
    "textarea",
    "Floorp OS Automator による自動入力テスト"
  );

  console.log("Form filling complete!");
}

// =============================================================================
// STEP 5: CALCULATE AVAILABLE DATES
// =============================================================================

function calculateAvailableDates(thunderbirdEvents, googleEvents) {
  console.log("\n=== Step 5: Calculating Available Dates ===");

  // Get all busy dates from both calendars
  const busyDates = new Set();

  for (const event of thunderbirdEvents) {
    if (event.date) {
      busyDates.add(event.date);
    }
  }

  console.log(`Busy dates: ${Array.from(busyDates).join(", ")}`);

  // Generate next 14 days
  const availableDates = [];
  const today = new Date();

  for (let i = 1; i <= 14; i++) {
    const date = new Date(today);
    date.setDate(date.getDate() + i);
    const dateStr = date.toISOString().split("T")[0];

    // Skip weekends
    if (date.getDay() === 0 || date.getDay() === 6) continue;

    // Skip busy dates
    if (busyDates.has(dateStr)) continue;

    availableDates.push(dateStr);
  }

  console.log(`Available dates: ${availableDates.slice(0, 5).join(", ")}...`);

  return availableDates;
}

// =============================================================================
// MAIN EXECUTION
// =============================================================================

async function main() {
  console.log("╔════════════════════════════════════════════╗");
  console.log("║  Calendar Aggregation Demo                 ║");
  console.log("║  Thunderbird + Google Calendar → Form      ║");
  console.log("╚════════════════════════════════════════════╝");

  // Step 1: Read Thunderbird calendar
  const thunderbirdEvents = await readThunderbirdCalendar();

  // Step 2: Scrape Google Calendar
  const googleEvents = await scrapeGoogleCalendar();

  // Step 3: Calculate available dates
  const availableDates = calculateAvailableDates(
    thunderbirdEvents,
    googleEvents
  );

  // Step 4: Open Google Form and analyze structure
  console.log("\n=== Opening Google Form ===");
  const formTabId = await floorp.createTab(CONFIG.formUrl);
  await floorp.waitForNetworkIdle(formTabId);
  await new Promise((resolve) => setTimeout(resolve, 2000));

  // Analyze form structure (get HTML first)
  const formData = await analyzeFormStructure(formTabId);

  // Step 5: Fill the form
  await fillForm(formTabId, availableDates);

  console.log("\n=== Workflow Complete ===");
  console.log("Form is ready for review. Submit manually if correct.");

  return {
    thunderbirdEvents: thunderbirdEvents.length,
    googleEvents: googleEvents.length,
    availableDates: availableDates.slice(0, 5),
    formAnalysis: formData.fieldCounts,
  };
}

// Run the workflow
const result = await main();
console.log("\n=== Summary ===");
console.log(JSON.stringify(result, null, 2));
