// Thunderbird Plugin Test Script
// Tests the thunderbird plugin API functions

function workflow() {
  console.log("=== Thunderbird Plugin Test ===");
  console.log("");

  // Test 1: Get Profile
  console.log("[Test 1] thunderbird.getProfile()");
  try {
    var profile = thunderbird.getProfile();
    console.log("  ✓ Profile: " + profile);
  } catch (e) {
    console.log("  ✗ Error: " + e);
  }

  // Test 2: Get Identity
  console.log("");
  console.log("[Test 2] thunderbird.getIdentity()");
  try {
    var identity = thunderbird.getIdentity();
    console.log("  ✓ Name: " + identity.name);
    console.log("  ✓ Email: " + identity.email);
    console.log("  ✓ Profile: " + identity.profile);
  } catch (e) {
    console.log("  ✗ Error: " + e);
  }

  // Test 3: Get Calendar Events (14 days)
  console.log("");
  console.log("[Test 3] thunderbird.getCalendarEvents(14)");
  try {
    var events = thunderbird.getCalendarEvents(14);
    console.log("  ✓ Found " + events.length + " events");
    for (var i = 0; i < Math.min(5, events.length); i++) {
      var ev = events[i];
      console.log(
        "    - " +
          ev.title +
          " (" +
          ev.date +
          " " +
          ev.start_time.split(" ")[1] +
          ")"
      );
    }
    if (events.length > 5) {
      console.log("    ... and " + (events.length - 5) + " more");
    }
  } catch (e) {
    console.log("  ✗ Error: " + e);
  }

  // Test 4: Get Calendar Events (7 days)
  console.log("");
  console.log("[Test 4] thunderbird.getCalendarEvents(7)");
  try {
    var events7 = thunderbird.getCalendarEvents(7);
    console.log("  ✓ Found " + events7.length + " events in next 7 days");
  } catch (e) {
    console.log("  ✗ Error: " + e);
  }

  // Test 5: Get Emails
  console.log("");
  console.log("[Test 5] thunderbird.getEmails('inbox', 5)");
  try {
    var emails = thunderbird.getEmails("inbox", 5);
    console.log("  ✓ Found " + emails.length + " emails");
    for (var i = 0; i < emails.length; i++) {
      var email = emails[i];
      console.log(
        "    - [" +
          email.date +
          "] " +
          email.subject.substring(0, 40) +
          (email.subject.length > 40 ? "..." : "")
      );
      console.log("      From: " + email.sender);
    }
  } catch (e) {
    console.log("  ✗ Error: " + e);
  }

  console.log("");
  console.log("=== Test Complete ===");
}

workflow();
