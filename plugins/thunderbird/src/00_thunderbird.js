// Thunderbird Plugin - JavaScript Interface
// Provides access to Thunderbird identity, calendar, and email data

// Helper function for safe operation calls with error handling
function safeOpCall(opName, args) {
  try {
    // Check if Deno.core.ops exists
    if (typeof Deno === "undefined" || !Deno.core || !Deno.core.ops) {
      return {
        success: false,
        error: "Deno.core.ops not available - plugin not properly loaded",
      };
    }

    // Get the operation function
    var opFunc = Deno.core.ops[opName];
    if (typeof opFunc !== "function") {
      return {
        success: false,
        error: "Operation " + opName + " not found",
      };
    }

    // Call the operation
    var rawResult;
    if (args && args.length > 0) {
      rawResult = opFunc.apply(null, args);
    } else {
      rawResult = opFunc();
    }

    // Parse JSON result
    if (typeof rawResult === "string") {
      return {
        success: true,
        data: JSON.parse(rawResult),
      };
    } else if (rawResult !== null && rawResult !== undefined) {
      return {
        success: true,
        data: rawResult,
      };
    } else {
      return {
        success: false,
        error: "Operation returned null/undefined",
      };
    }
  } catch (e) {
    return {
      success: false,
      error: e.message || String(e),
    };
  }
}

var thunderbird = {
  // Plugin metadata
  _pluginInfo: {
    name: "Thunderbird",
    version: "0.1.0",
    description: "Access Thunderbird identity, calendar, and email data",
  },

  // Check if plugin is available
  isAvailable: function () {
    return typeof Deno !== "undefined" && Deno.core && Deno.core.ops;
  },

  // Get detailed availability info
  getAvailabilityInfo: function () {
    var info = {
      available: false,
      denoExists: typeof Deno !== "undefined",
      coreExists: false,
      opsExists: false,
      operations: {},
    };

    if (info.denoExists && Deno.core) {
      info.coreExists = true;
      if (Deno.core.ops) {
        info.opsExists = true;
        info.available = true;

        // Check each operation
        var ops = [
          "op2_thunderbird_get_identity",
          "op2_thunderbird_get_calendar_events",
          "op2_thunderbird_get_profile",
          "op2_thunderbird_get_emails",
          "op2_thunderbird_activate_floorp",
        ];

        for (var i = 0; i < ops.length; i++) {
          info.operations[ops[i]] = typeof Deno.core.ops[ops[i]] === "function";
        }
      }
    }

    return info;
  },

  getIdentity: function () {
    var result = safeOpCall("op2_thunderbird_get_identity");
    if (result.success) {
      return result.data;
    } else {
      throw new Error("thunderbird.getIdentity failed: " + result.error);
    }
  },

  getCalendarEvents: function (days) {
    var result = safeOpCall("op2_thunderbird_get_calendar_events", [
      days || 14,
    ]);
    if (result.success) {
      return result.data;
    } else {
      throw new Error(
        "thunderbird.getCalendarEvents failed: " + result.error
      );
    }
  },

  getProfile: function () {
    var result = safeOpCall("op2_thunderbird_get_profile");
    if (result.success) {
      return result.data;
    } else {
      throw new Error("thunderbird.getProfile failed: " + result.error);
    }
  },

  getEmails: function (folder, limit) {
    var result = safeOpCall("op2_thunderbird_get_emails", [
      folder || "inbox",
      limit || 20,
    ]);
    if (result.success) {
      return result.data;
    } else {
      throw new Error("thunderbird.getEmails failed: " + result.error);
    }
  },

  // Try to get identity safely (returns null on error instead of throwing)
  tryGetIdentity: function () {
    var result = safeOpCall("op2_thunderbird_get_identity");
    return result.success ? result.data : null;
  },

  // Try to get calendar events safely
  tryGetCalendarEvents: function (days) {
    var result = safeOpCall("op2_thunderbird_get_calendar_events", [
      days || 14,
    ]);
    return result.success ? result.data : [];
  },

  // Try to get emails safely
  tryGetEmails: function (folder, limit) {
    var result = safeOpCall("op2_thunderbird_get_emails", [
      folder || "inbox",
      limit || 20,
    ]);
    return result.success ? result.data : [];
  },

  activateFloorp: function () {
    var result = safeOpCall("op2_thunderbird_activate_floorp");
    if (!result.success) {
      throw new Error("thunderbird.activateFloorp failed: " + result.error);
    }
    // Return success status from result.data
    return result.data && result.data.success;
  },
};

// Export to globalThis
globalThis.thunderbird = thunderbird;
