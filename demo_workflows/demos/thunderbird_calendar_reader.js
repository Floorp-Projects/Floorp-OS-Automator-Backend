// Calendar Aggregation Demo - Thunderbird Calendar Reader
// This workflow reads calendar events from Thunderbird's local SQLite database
// and outputs them in a structured format.

// =============================================================================
// THUNDERBIRD CALENDAR READING
// =============================================================================

/**
 * Reads events from Thunderbird calendar database
 * Uses exec plugin to run sqlite3 commands
 */
async function readThunderbirdCalendar() {
  console.log("=== Reading Thunderbird Calendar ===");

  // Thunderbird profile path (macOS)
  // Note: This needs to be adjusted for the actual profile name
  const profilePath = "~/Library/Thunderbird/Profiles/awq9kwrc.default-release";
  const dbPath = `${profilePath}/calendar-data/cache.sqlite`;

  // Copy database to avoid lock issues (Thunderbird may be running)
  const tempDb = "/tmp/thunderbird_calendar_temp.sqlite";

  try {
    // Copy the database to temp location
    exec(`cp ${dbPath} ${tempDb}`);

    // Query upcoming events (next 30 days)
    const now = Date.now() * 1000; // Thunderbird uses microseconds
    const thirtyDaysLater = now + 30 * 24 * 60 * 60 * 1000000;

    const query = `
      SELECT 
        title,
        datetime(event_start/1000000, 'unixepoch', 'localtime') as start_time,
        datetime(event_end/1000000, 'unixepoch', 'localtime') as end_time,
        event_start,
        event_end
      FROM cal_events 
      WHERE event_start >= ${now} AND event_start <= ${thirtyDaysLater}
      ORDER BY event_start ASC;
    `;

    const result = exec(`sqlite3 -separator '|' ${tempDb} "${query}"`);

    // Parse results
    const events = [];
    const lines = result
      .trim()
      .split("\n")
      .filter((line) => line.length > 0);

    for (const line of lines) {
      const [title, startTime, endTime, startMicros, endMicros] =
        line.split("|");
      events.push({
        title: title,
        startTime: startTime,
        endTime: endTime,
        startTimestamp: parseInt(startMicros) / 1000000,
        endTimestamp: parseInt(endMicros) / 1000000,
        source: "thunderbird",
      });
    }

    console.log(`Found ${events.length} upcoming events in Thunderbird`);

    // Clean up temp file
    exec(`rm -f ${tempDb}`);

    return events;
  } catch (error) {
    console.error("Error reading Thunderbird calendar:", error);
    return [];
  }
}

// =============================================================================
// MAIN EXECUTION
// =============================================================================

async function main() {
  console.log("Starting Calendar Aggregation Demo");
  console.log("==================================");

  // Step 1: Read Thunderbird calendar
  const thunderbirdEvents = await readThunderbirdCalendar();

  console.log("\n--- Thunderbird Events ---");
  for (const event of thunderbirdEvents) {
    console.log(`📅 ${event.title}`);
    console.log(`   ${event.startTime} → ${event.endTime}`);
  }

  // Step 2: Calculate busy times
  console.log("\n--- Busy Time Slots ---");
  const busySlots = thunderbirdEvents.map((e) => ({
    date: e.startTime.split(" ")[0],
    startTime: e.startTime.split(" ")[1],
    endTime: e.endTime.split(" ")[1],
    title: e.title,
  }));

  for (const slot of busySlots) {
    console.log(
      `🔴 ${slot.date}: ${slot.startTime} - ${slot.endTime} (${slot.title})`
    );
  }

  return {
    thunderbirdEvents: thunderbirdEvents,
    busySlots: busySlots,
    totalEvents: thunderbirdEvents.length,
  };
}

// Run the workflow
const result = await main();
console.log("\n=== Workflow Complete ===");
console.log(JSON.stringify(result, null, 2));
