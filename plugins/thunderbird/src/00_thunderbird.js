// Thunderbird Plugin - JavaScript Interface
// Provides access to Thunderbird identity and calendar data

var thunderbird = {
  getIdentity: function () {
    return JSON.parse(Deno.core.ops.op2_thunderbird_get_identity());
  },

  getCalendarEvents: function (days) {
    return JSON.parse(
      Deno.core.ops.op2_thunderbird_get_calendar_events(days || 14)
    );
  },

  getProfile: function () {
    return Deno.core.ops.op2_thunderbird_get_profile();
  },
};

globalThis.thunderbird = thunderbird;
