/**
 * AI Coding Tools — Subscription Deep Research
 *
 * Scoped to AI-powered coding assistants & developer AI tools ONLY.
 * Unifies browser context (logged-in billing pages) and local macOS assets
 * (installed applications + usage signals) to produce a focused
 * AI coding-tool subscription-spending optimisation report.
 *
 * Covered tools include:
 *   GitHub Copilot, Cursor, Windsurf/Codeium, Tabnine, Supermaven,
 *   Continue, Aider, Sourcegraph Cody, JetBrains AI Assistant,
 *   Amazon Q / CodeWhisperer, Augment Code, Devin, Replit AI,
 *   ChatGPT (OpenAI), Claude (Anthropic), Gemini Code Assist
 *
 * SAFETY RULES (non-negotiable):
 *   - NEVER cancel / upgrade / downgrade any subscription automatically.
 *   - NEVER submit purchases or modify payment methods.
 *   - READ-ONLY for web pages, local inventories, and optional PDFs.
 *   - WRITE only to the output directory (report artifacts).
 *   - All proposed changes are "actions queue" items with human instructions.
 *
 * PIPELINE:
 *   0. Build Pre-Execution Audit → log & display
 *   1. Build Local App Inventory  (macOS /Applications, Homebrew, usage signals)
 *   2. Build Subscription Inventory  (open browser tabs + optional PDF invoices)
 *   3. Normalise & deduplicate
 *   3.5. Web Exploration  (DuckDuckGo search for unknown pricing, plan
 *         verification, and checking if installed apps are subscription-based)
 *   4. Join apps ↔ subscriptions, detect waste / duplicates / unused
 *   5. Generate 3-category recommendations
 *   6. Write 4 artifacts:
 *        subscriptions_inventory.md
 *        optimization_plan.md
 *        actions_queue.json
 *        audit_log.json
 *   7. STOP — do not execute any changes.
 *
 * Required plugins: floorp, llm_chat, app.sapphillon.core.exec,
 *                   app.sapphillon.core.filesystem
 * Optional plugins: ocr (for PDF invoices)
 */

// ============================================================================
// Configuration
// ============================================================================

var DEBUG = true;
var AUDIT_ONLY = false; // true = generate audit plan & stop

var CONFIG = {
  // Output ------------------------------------------------------------------
  outputBaseDir: "", // resolved at runtime → ~/Documents/FloorpOS/SubsOptimization/YYYY-MM-DD

  // Local app sources -------------------------------------------------------
  appDirs: ["/Applications"],
  userAppDir: "", // resolved → ~/Applications
  useHomebrew: true,
  useUsageSignals: true,

  // Browser sources ---------------------------------------------------------
  // Keywords to identify billing/subscription tabs among all open tabs
  billingTabKeywords: [
    "billing",
    "subscription",
    "subscriptions",
    "plan",
    "pricing",
    "payment",
    "invoice",
    "account",
    "settings/billing",
    "manage",
    "usage",
    "manage-apikey",
    "orders",
    "receipts",
  ],
  // URL patterns that indicate a pricing/comparison page, NOT the user's billing
  pricingPageExcludePatterns: [
    "/subscribe?",
    "/subscribe#",
    "/pricing",
    "/plans",
    "/compare",
    "/upgrade",
  ],
  // Known billing domains — AI coding tools only
  knownBillingDomains: [
    "github.com",
    "cursor.com",
    "openai.com",
    "anthropic.com",
    "z.ai",
    "myaccount.google.com",
    "app.link.com",
    "codeium.com",
    "windsurf.com",
    "tabnine.com",
    "jetbrains.com",
    "sourcegraph.com",
    "augmentcode.com",
    "supermaven.com",
    "continue.dev",
    "replit.com",
    "aws.amazon.com",
  ],
  // Auto-open sources — AI coding tool billing pages only
  autoOpenSources: [
    {
      id: "link",
      label: "Link.com (Stripe)",
      url: "https://app.link.com/subscriptions",
      waitSelector: "body",
      selectors: {
        list: "main.RouteContent > div.ListAndDetailView > div.ListAndDetailView-list",
        detail:
          "main.RouteContent > div.ListAndDetailView > div.ListAndDetailView-detail",
        inactiveToggle: 'button:has-text("すべての非アクティブを表示")',
      },
    },
    {
      id: "github_copilot",
      label: "GitHub Copilot",
      url: "https://github.com/settings/billing",
      waitSelector: "body",
      selectors: {
        planCard: "[data-testid='copilot-plan-card']",
        billingContainer: "[data-hpc]",
      },
    },
    {
      id: "google_subscriptions",
      label: "Google Subscriptions",
      url: "https://myaccount.google.com/subscriptions",
      waitSelector: "body",
      selectors: {
        main: "[role='main'], main, c-wiz",
      },
    },
    {
      id: "zai",
      label: "Z.ai",
      url: "https://z.ai/manage-apikey/subscription",
      waitSelector: "div[role='tabpanel']",
    },
  ],
  // Additional user-defined URLs (same format as autoOpenSources)
  additionalSources: [],
  maxPageChars: 14000,

  // PDF / invoice sources ---------------------------------------------------
  invoiceSearchPaths: [], // resolved at runtime → ~/Documents, ~/Downloads
  invoiceExtensions: ["pdf"],
  maxInvoices: 10,

  // Currency ----------------------------------------------------------------
  targetCurrency: "JPY",
  fallbackRates: { USD: 150, EUR: 163, GBP: 190, JPY: 1 },

  // LLM settings ------------------------------------------------------------
  maxLlmRetries: 2,

  // Web exploration (DuckDuckGo search + page visits) ----------------------
  webExplore: {
    enabled: true, // set false to skip web exploration entirely
    maxSearches: 15, // max DuckDuckGo searches to perform
    resultsPerSearch: 5, // search results to collect per query
    maxVisitsPerSearch: 2, // pages to visit per search query
    maxTotalVisits: 30, // hard cap on total page visits
    searchDelay: 2500, // ms between DuckDuckGo searches
    visitDelay: 1500, // ms between page visits
    pageLoadTimeout: 12000, // ms to wait for page load
    maxPageChars: 6000, // max chars to extract per visited page
  },

  // Keywords that definitively EXCLUDE an app from being an AI coding tool.
  // Checked BEFORE knownAiToolKeywords to prevent false positives.
  notAiToolKeywords: [
    "clipy",
    "clipboard",
    "docker",
    "compressor",
    "discord",
    "1password",
    "lastpass",
    "bitwarden",
    "keychain",
    "canva",
    "affinity",
    "photoshop",
    "illustrator",
    "sketch",
    "figma",
    "zoom",
    "slack",
    "teams",
    "spotify",
    "vlc",
    "firefox",
    "chrome",
    "safari",
    "brave",
    "steam",
    "notion",
    "obsidian",
    "evernote",
    "finder",
    "preview",
    "calculator",
    "calendar",
    "mail",
    "messages",
    "music",
    "photos",
    "reminders",
    "notes",
    "iterm",
    "terminal",
    "warp",
    "alacritty",
    "kitty",
    "uninstaller",
    "cleaner",
    "cleanup",
    "virtualbox",
    "parallels",
    "vmware",
    "handbrake",
    "google sheets",
    "google docs",
    "google slides",
    "google drive",
    "google meet",
    "google earth",
    "keynote",
    "final cut",
    "imovie",
    "garageband",
    "pages",
    "numbers",
    "xcode",
    "instruments",
    "accessibility inspector",
    "font book",
    "grapher",
    "automator",
    "screen sharing",
    "migration assistant",
    "microsoft edge",
    "microsoft word",
    "microsoft excel",
    "microsoft powerpoint",
    "microsoft onenote",
    "microsoft outlook",
    "microsoft teams",
    "microsoft to do",
    "logic pro",
    "mainstage",
    "motion",
    "mactube",
    "creator studio",
    "antigravity",
    "floorp",
    "raycast",
    "alfred",
    "bartender",
    "magnet",
    "rectangle",
    "bettertouchtool",
    "karabiner",
    "homebrew",
    "postman",
    "insomnia",
    "tableplus",
    "sequel pro",
    "pgadmin",
    "dbeaver",
    "transmit",
    "cyberduck",
    "tower",
    "gitkraken",
    "sourcetree",
    "textedit",
    "sublime text",
    "bbedit",
    "myna",
    "mynaportal",
    "thunderbird",
    "onedrive",
    "surfshark",
    "vpn",
    "tor browser",
    "mongodb",
    "compass",
    "unarchiver",
    "vencord",
    "nativeyoutube",
    "youtube",
    "google one",
    "google play",
    "pokemon",
    "pokémon",
    "pocket",
    "game",
    "kimi",
    "ollama",
    "lm studio",
    "gpt4all",
    "flowith",
    "google ai",
    "webcatalog",
    "wine",
    "wireshark",
    "zipang",
  ],

  // Short exact names that should be excluded (substring match would be too broad).
  // E.g. "line" would match "cline" with indexOf, so use exact match.
  notAiToolExactNames: [
    "line",
    "dia",
    "bear",
    "paw",
    "tot",
    "hex",
    "ftp",
    "collect",
  ],

  // Keywords that definitively identify an app as an AI coding tool.
  // Apps whose names contain any of these skip DDG verification.
  knownAiToolKeywords: [
    "copilot",
    "cursor",
    "windsurf",
    "codeium",
    "tabnine",
    "supermaven",
    "cody",
    "sourcegraph",
    "jetbrains",
    "replit",
    "augment",
    "devin",
    "amazonq",
    "codewhisperer",
    "openai",
    "chatgpt",
    "claude",
    "anthropic",
    "github",
    "codex",
    "aider",
    "continue",
    "cline",
    "roo code",
    "roo-code",
    "opencode",
    "codegeex",
    "glm code",
    "qwen coder",
    "gemini code",
    "bolt.new",
    "lovable",
    "trae",
    "v0",
  ],

  // Max DDG verification searches for AI tool filtering
  aiFilterMaxSearches: 10,
  aiFilterSearchDelay: 2500, // ms between verification searches
  aiFilterMaxVisitsPerSearch: 1, // pages to visit per verification

  // Subscription category overlaps — AI coding tools only -------------------
  categoryGroups: {
    "AI Code Completion": [
      "copilot",
      "cursor",
      "windsurf",
      "codeium",
      "tabnine",
      "supermaven",
      "continue",
      "amazonq",
      "codewhisperer",
      "augment",
    ],
    "AI Code Agent": [
      "devin",
      "cursor",
      "copilot",
      "claude code",
      "aider",
      "augment",
    ],
    "AI Code Search": ["sourcegraph", "cody", "github copilot"],
    "AI Chat / API (for coding)": [
      "chatgpt",
      "claude",
      "openai",
      "anthropic",
      "z.ai",
      "zai",
      "gemini",
    ],
    "IDE / Editor AI": ["jetbrains ai", "jetbrains", "replit", "v0", "trae"],
    "AI App Builder": ["bolt.new", "lovable", "v0"],
    "Open Source AI Code": [
      "continue",
      "aider",
      "opencode",
      "cline",
      "roo code",
      "codegeex",
      "qwen coder",
    ],
  },

  // Safety ------------------------------------------------------------------
  prohibitedActions: [
    "Cancel any subscription",
    "Upgrade or downgrade any plan",
    "Submit any purchase",
    "Modify payment methods or billing info",
    "Delete any user data",
    "Send emails or messages on the user's behalf",
  ],
};

// ============================================================================
// Runtime State
// ============================================================================

var AUDIT_LOG = [];
var EXCHANGE_RATES = {};

// ============================================================================
// Utility Functions
// ============================================================================

function debugLog(msg) {
  if (DEBUG) console.log("[DEBUG] " + msg);
}

function timestamp() {
  return new Date().toISOString();
}

function auditLog(action, detail) {
  var entry = { timestamp: timestamp(), action: action, detail: detail };
  AUDIT_LOG.push(entry);
  debugLog("[AUDIT] " + action + ": " + detail);
}

function safeSleep(ms) {
  if (typeof sleep === "function") sleep(ms);
}

function safeJsonParse(text) {
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch (e) {
    return null;
  }
}

function extractJsonArray(text) {
  if (!text) return "";
  var m = text.match(/\[[\s\S]*\]/);
  return m ? m[0] : "";
}

function extractJsonObject(text) {
  if (!text) return "";
  var m = text.match(/\{[\s\S]*\}/);
  return m ? m[0] : "";
}

function sanitizeLlmOutput(text) {
  if (!text) return "";
  var c = String(text);
  c = c.replace(/```[\s\S]*?```/g, function (block) {
    return block.replace(/```[a-zA-Z]*\n?/, "").replace(/```$/, "");
  });
  return c.trim();
}

function shellEscape(path) {
  return "'" + String(path || "").replace(/'/g, "'\\''") + "'";
}

function safeMd(value) {
  return String(value || "")
    .replace(/\|/g, "\\|")
    .replace(/\n/g, " ");
}

function execSafe(cmd) {
  try {
    var result = app.sapphillon.core.exec.exec(cmd);
    return result ? String(result).trim() : "";
  } catch (e) {
    debugLog("exec error: " + cmd + " → " + e);
    return "";
  }
}

function writeFile(path, content) {
  try {
    app.sapphillon.core.filesystem.write(path, content);
    auditLog("write_file", path);
    return true;
  } catch (e) {
    debugLog("filesystem.write failed for " + path + ", falling back to exec");
    try {
      var delimiter = "__SAPPHILLON_EOF__";
      var safe = String(content || "").replace(
        new RegExp(delimiter, "g"),
        delimiter + "_",
      );
      var cmd =
        "cat <<'" +
        delimiter +
        "' > " +
        shellEscape(path) +
        "\n" +
        safe +
        "\n" +
        delimiter;
      app.sapphillon.core.exec.exec(cmd);
      auditLog("write_file_exec", path);
      return true;
    } catch (e2) {
      console.log("[ERROR] Failed to write " + path + ": " + e2);
      return false;
    }
  }
}

function ensureDirExists(dirPath) {
  execSafe("mkdir -p " + shellEscape(dirPath));
}

function getHomeDir() {
  try {
    var result = app.sapphillon.core.exec.exec("echo $HOME");
    if (result) return String(result).trim();
  } catch (e) {}
  return "/Users/user";
}

function getDateString() {
  var d = new Date();
  var y = d.getFullYear();
  var m = String(d.getMonth() + 1).padStart(2, "0");
  var day = String(d.getDate()).padStart(2, "0");
  return y + "-" + m + "-" + day;
}

function lineHasKeyword(line, keywords) {
  if (!line || !keywords || !keywords.length) return false;
  var lower = line.toLowerCase();
  for (var i = 0; i < keywords.length; i++) {
    if (lower.indexOf(String(keywords[i]).toLowerCase()) !== -1) return true;
  }
  return false;
}

function llmChat(systemPrompt, userPrompt, retries) {
  retries = retries || CONFIG.maxLlmRetries;
  var attempt = 0;
  while (attempt <= retries) {
    try {
      var raw = llm_chat.chat(systemPrompt, userPrompt);
      return sanitizeLlmOutput(raw);
    } catch (e) {
      attempt++;
      if (attempt > retries) {
        console.log(
          "[WARN] LLM call failed after " + retries + " retries: " + e,
        );
        return "";
      }
      safeSleep(1000 * attempt);
    }
  }
  return "";
}

// ============================================================================
// Prerequisites Check
// ============================================================================

function ensurePrerequisites() {
  var errors = [];
  if (typeof floorp === "undefined") errors.push("floorp plugin");
  if (typeof llm_chat === "undefined" || !llm_chat.chat)
    errors.push("llm_chat plugin");
  if (
    !app ||
    !app.sapphillon ||
    !app.sapphillon.core ||
    !app.sapphillon.core.exec
  )
    errors.push("exec plugin");
  if (!app.sapphillon.core.filesystem) errors.push("filesystem plugin");
  if (errors.length) {
    throw new Error("Missing required plugins: " + errors.join(", "));
  }
}

// ============================================================================
// 0. Pre-Execution Audit
// ============================================================================

function resolveConfig() {
  var home = getHomeDir();
  var dateStr = getDateString();
  CONFIG.outputBaseDir =
    home + "/Documents/FloorpOS/SubsOptimization/" + dateStr;
  CONFIG.userAppDir = home + "/Applications";
  CONFIG.invoiceSearchPaths = [home + "/Documents", home + "/Downloads"];
}

function buildPreExecutionAudit() {
  var readSources = [];

  // Local paths
  readSources.push({
    type: "local_dir",
    path: "/Applications",
    purpose: "app inventory",
  });
  readSources.push({
    type: "local_dir",
    path: CONFIG.userAppDir,
    purpose: "user app inventory",
  });
  if (CONFIG.useHomebrew) {
    readSources.push({
      type: "command",
      command: "brew list --cask",
      purpose: "Homebrew cask apps",
    });
    readSources.push({
      type: "command",
      command: "brew list --formula",
      purpose: "Homebrew formula apps",
    });
  }
  if (CONFIG.useUsageSignals) {
    readSources.push({
      type: "command",
      command: "mdls (per app)",
      purpose: "app usage signals",
    });
  }

  // Browser tabs (will be discovered at runtime)
  readSources.push({
    type: "browser_auto_open",
    urls: CONFIG.autoOpenSources.map(function (s) {
      return s.label + " (" + s.url + ")";
    }),
    purpose:
      "auto-open known billing pages, extract subscription data, then close",
  });
  readSources.push({
    type: "browser_tabs",
    filter: "billing/subscription keyword match on already-open tabs",
    purpose: "subscription data extraction from logged-in billing pages",
  });

  // Optional invoices
  for (var i = 0; i < CONFIG.invoiceSearchPaths.length; i++) {
    readSources.push({
      type: "local_dir",
      path: CONFIG.invoiceSearchPaths[i],
      purpose: "search for PDF invoices",
    });
  }

  // Web exploration via DuckDuckGo
  if (CONFIG.webExplore.enabled) {
    readSources.push({
      type: "web_search",
      engine: "DuckDuckGo",
      purpose:
        "search for pricing/plan info on subscriptions with unknown prices, " +
        "and check if installed apps are subscription-based",
      max_searches: CONFIG.webExplore.maxSearches,
      max_page_visits: CONFIG.webExplore.maxTotalVisits,
    });
  }

  var writeTargets = [
    CONFIG.outputBaseDir + "/subscriptions_inventory.md",
    CONFIG.outputBaseDir + "/optimization_plan.md",
    CONFIG.outputBaseDir + "/actions_queue.json",
    CONFIG.outputBaseDir + "/audit_log.json",
  ];

  // Discover open billing tabs
  var detectedTabs = [];
  try {
    var tabsRaw = floorp.browserTabs();
    var tabs = safeJsonParse(tabsRaw);
    if (Array.isArray(tabs)) {
      for (var t = 0; t < tabs.length; t++) {
        var tab = tabs[t];
        var url = String(tab.url || "").toLowerCase();
        var title = String(tab.title || "").toLowerCase();
        if (isBillingTab(url, title)) {
          detectedTabs.push({
            url: tab.url,
            title: tab.title,
            id: tab.id || tab.tabId,
          });
        }
      }
    }
  } catch (e) {
    debugLog("Could not list browser tabs for audit: " + e);
  }

  var networkAccess = [];
  // Auto-open sources
  for (var a = 0; a < CONFIG.autoOpenSources.length; a++) {
    networkAccess.push(CONFIG.autoOpenSources[a].url);
  }
  for (var d = 0; d < detectedTabs.length; d++) {
    networkAccess.push(detectedTabs[d].url);
  }
  // Exchange rate API
  networkAccess.push(
    "https://api.exchangerate-api.com (currency conversion, optional)",
  );
  // Web exploration (DuckDuckGo)
  if (CONFIG.webExplore.enabled) {
    networkAccess.push(
      "https://duckduckgo.com (pricing/plan verification searches, up to " +
        CONFIG.webExplore.maxSearches +
        " queries)",
    );
    networkAccess.push(
      "Various pricing pages (up to " +
        CONFIG.webExplore.maxTotalVisits +
        " page visits via DuckDuckGo search results)",
    );
  }

  var audit = {
    workflow: "Subscription Optimization Deep Research",
    generated_at: timestamp(),
    read_sources: readSources,
    detected_billing_tabs: detectedTabs,
    write_targets: writeTargets,
    network_access: networkAccess,
    confirmations_needed: [
      "User runs this workflow (implicit approval)",
      "All detected billing tabs are listed above — review before proceeding",
    ],
    prohibited_actions: CONFIG.prohibitedActions,
    notes: [
      "This workflow is READ-ONLY for all subscription/billing pages.",
      "No subscriptions will be cancelled, upgraded, or downgraded.",
      "The actions_queue.json contains human-executable instructions only.",
      CONFIG.autoOpenSources.length +
        " billing page(s) will be auto-opened and then closed.",
      detectedTabs.length === 0
        ? "No additional billing tabs detected among already-open tabs."
        : detectedTabs.length +
          " additional billing tab(s) detected among already-open tabs.",
      CONFIG.webExplore.enabled
        ? "DuckDuckGo web exploration ENABLED — will search for unknown pricing/plan info."
        : "DuckDuckGo web exploration DISABLED.",
    ],
  };

  return audit;
}

function isBillingTab(url, title) {
  var combined = url + " " + title;
  // Exclude pricing/comparison pages (e.g. /subscribe?plan=Pro, /pricing)
  var excludePatterns = CONFIG.pricingPageExcludePatterns || [];
  for (var ep = 0; ep < excludePatterns.length; ep++) {
    if (url.indexOf(excludePatterns[ep]) !== -1) {
      debugLog("isBillingTab: excluding pricing page: " + url);
      return false;
    }
  }
  // Check known billing domains
  for (var i = 0; i < CONFIG.knownBillingDomains.length; i++) {
    if (url.indexOf(CONFIG.knownBillingDomains[i]) !== -1) {
      if (lineHasKeyword(combined, CONFIG.billingTabKeywords)) return true;
    }
  }
  // Generic keyword match on URL path
  if (lineHasKeyword(url, CONFIG.billingTabKeywords)) return true;
  // Title-based match
  if (
    lineHasKeyword(title, [
      "billing",
      "subscription",
      "plan",
      "payment",
      "invoice",
      "サブスクリプション",
      "お支払い",
      "プラン",
    ])
  )
    return true;
  return false;
}

// ============================================================================
// 1. Local App Inventory
// ============================================================================

function buildLocalAppInventory() {
  auditLog("start_local_inventory", "Building macOS local app inventory");
  var apps = [];

  // 1a. /Applications
  apps = apps.concat(listAppsInDir("/Applications", "system"));
  auditLog("read_local_apps", "/Applications (" + apps.length + " apps)");

  // 1b. ~/Applications
  var userApps = listAppsInDir(CONFIG.userAppDir, "user");
  apps = apps.concat(userApps);
  if (userApps.length) {
    auditLog(
      "read_local_apps",
      CONFIG.userAppDir + " (" + userApps.length + " apps)",
    );
  }

  // 1c. Homebrew casks
  if (CONFIG.useHomebrew) {
    var brewApps = listHomebrewApps();
    apps = apps.concat(brewApps);
    auditLog("read_homebrew", "Homebrew (" + brewApps.length + " cask apps)");
  }

  // 1d. Usage signals
  if (CONFIG.useUsageSignals) {
    apps = enrichWithUsageSignals(apps);
    auditLog(
      "read_usage_signals",
      "Applied mdls usage signals to " + apps.length + " apps",
    );
  }

  console.log("[Step 1] Local app inventory: " + apps.length + " apps found");
  return apps;
}

function listAppsInDir(dirPath, source) {
  var raw = execSafe(
    "ls -1 " + shellEscape(dirPath) + " 2>/dev/null | grep '\\.app$'",
  );
  if (!raw) return [];
  var lines = raw.split("\n");
  var apps = [];
  for (var i = 0; i < lines.length; i++) {
    var name = lines[i].trim();
    if (!name) continue;
    var displayName = name.replace(/\.app$/, "");
    var fullPath = dirPath + "/" + name;

    // Get bundle ID
    var bundleId = "";
    try {
      bundleId = execSafe(
        "mdls -name kMDItemCFBundleIdentifier -raw " +
          shellEscape(fullPath) +
          " 2>/dev/null",
      );
      if (bundleId === "(null)") bundleId = "";
    } catch (e) {}

    apps.push({
      name: displayName,
      bundleId: bundleId,
      path: fullPath,
      source: source,
      lastUsed: "",
      usageBucket: "unknown",
    });
  }
  return apps;
}

function listHomebrewApps() {
  var raw = execSafe("brew list --cask 2>/dev/null");
  if (!raw) return [];
  var lines = raw.split("\n");
  var apps = [];
  for (var i = 0; i < lines.length; i++) {
    var name = lines[i].trim();
    if (!name) continue;
    apps.push({
      name: name,
      bundleId: "",
      path: "(homebrew cask)",
      source: "homebrew",
      lastUsed: "",
      usageBucket: "unknown",
    });
  }
  return apps;
}

function enrichWithUsageSignals(apps) {
  for (var i = 0; i < apps.length; i++) {
    var a = apps[i];
    if (!a.path || a.path.indexOf(".app") === -1) continue;

    try {
      var mdls = execSafe(
        "mdls -name kMDItemLastUsedDate -raw " +
          shellEscape(a.path) +
          " 2>/dev/null",
      );
      if (mdls && mdls !== "(null)") {
        a.lastUsed = mdls.trim();
        a.usageBucket = categoriseUsage(mdls.trim());
      } else {
        a.usageBucket = "never_or_unknown";
      }
    } catch (e) {
      a.usageBucket = "unknown";
    }
  }
  return apps;
}

function categoriseUsage(dateString) {
  try {
    var ms = Date.parse(dateString);
    if (isNaN(ms)) return "unknown";
    var now = Date.now();
    var daysAgo = (now - ms) / (1000 * 60 * 60 * 24);
    if (daysAgo <= 7) return "daily_weekly";
    if (daysAgo <= 30) return "monthly";
    if (daysAgo <= 90) return "quarterly";
    if (daysAgo <= 365) return "rare";
    return "dormant";
  } catch (e) {
    return "unknown";
  }
}

// ============================================================================
// 2. Subscription Inventory — Browser Tab Detection
// ============================================================================

function buildSubscriptionInventory() {
  auditLog(
    "start_subscription_inventory",
    "Auto-opening billing pages and scanning browser tabs",
  );
  var subscriptions = [];
  var visitedUrls = {}; // track visited URLs to avoid duplicates

  // 2a. Auto-open & explore pre-configured billing sources
  var allSources = CONFIG.autoOpenSources.concat(CONFIG.additionalSources);
  console.log(
    "[Step 2a] Auto-opening " + allSources.length + " billing source(s)...",
  );

  for (var i = 0; i < allSources.length; i++) {
    var src = allSources[i];
    console.log("\n  [Collect] " + src.label + " → " + src.url);
    var tabId = null;
    try {
      tabId = floorp.createTab(src.url, false);
      if (src.waitSelector) {
        floorp.tabWaitForElement(tabId, src.waitSelector, 15000);
      }
      floorp.tabWaitForNetworkIdle(tabId);
      safeSleep(1500);

      var pageText = collectSourceText(tabId, src, CONFIG.maxPageChars);
      auditLog(
        "read_auto_source",
        src.url + " (" + pageText.length + " chars)",
      );

      if (pageText.length < 50) {
        debugLog("Page text too short for: " + src.url);
      } else {
        var extracted = llmExtractSubscriptions(pageText, src.url, src.label);
        // Fallback: if LLM failed for GitHub Copilot, extract manually from known plan patterns
        if ((!extracted || extracted.length === 0) && src.id === "github_copilot") {
          debugLog("GitHub Copilot LLM failed, trying manual extraction...");
          extracted = manualGitHubCopilotFallback(pageText);
          if (extracted && extracted.length) {
            debugLog("GitHub Copilot manual fallback: extracted " + extracted.length + " entries");
          }
        }
        if (extracted && extracted.length) {
          for (var j = 0; j < extracted.length; j++) {
            extracted[j].source_url = src.url;
            extracted[j].source_title = src.label;
            extracted[j].source_type = "auto_opened_source";
          }
          subscriptions = subscriptions.concat(extracted);
        }
      }
      visitedUrls[src.url.toLowerCase()] = true;
    } catch (e) {
      console.log("  [WARN] Failed to collect from " + src.label + ": " + e);
    } finally {
      if (tabId) {
        try {
          floorp.closeTab(tabId);
        } catch (e2) {}
      }
    }
  }

  // 2b. Also scan any already-open billing tabs not yet visited
  var billingTabs = detectBillingTabs();
  var newTabs = [];
  for (var b = 0; b < billingTabs.length; b++) {
    if (!visitedUrls[billingTabs[b].url.toLowerCase()]) {
      newTabs.push(billingTabs[b]);
    }
  }
  if (newTabs.length > 0) {
    console.log(
      "\n[Step 2b] Found " + newTabs.length + " additional open billing tab(s)",
    );
  }

  for (var k = 0; k < newTabs.length; k++) {
    var tab = newTabs[k];
    console.log("  [Tab] " + tab.title + " → " + tab.url);
    var instanceId = null;
    try {
      instanceId = floorp.attachToTab(String(tab.id));
      if (!instanceId) {
        debugLog("Could not attach to tab " + tab.id);
        continue;
      }
      safeSleep(500);

      var tabText = extractPageText(instanceId, CONFIG.maxPageChars);
      auditLog(
        "read_existing_tab",
        tab.url + " (" + tabText.length + " chars)",
      );

      if (tabText.length < 50) {
        debugLog("Page text too short for tab: " + tab.url);
        continue;
      }

      var tabEntries = llmExtractSubscriptions(tabText, tab.url, tab.title);
      if (tabEntries && tabEntries.length) {
        for (var m = 0; m < tabEntries.length; m++) {
          tabEntries[m].source_url = tab.url;
          tabEntries[m].source_title = tab.title;
          tabEntries[m].source_type = "existing_browser_tab";
        }
        subscriptions = subscriptions.concat(tabEntries);
      }
    } catch (e) {
      console.log("  [WARN] Failed to process tab: " + tab.url + " — " + e);
    }
    // Do NOT close attached tabs — they are the user's tabs
  }

  // 2c. Optional PDF invoice processing
  var pdfSubs = processInvoicePDFs();
  subscriptions = subscriptions.concat(pdfSubs);

  console.log(
    "\n[Step 2] Total raw subscription entries: " + subscriptions.length,
  );
  return subscriptions;
}

function detectBillingTabs() {
  var results = [];
  try {
    var tabsRaw = floorp.browserTabs();
    var tabs = safeJsonParse(tabsRaw);
    if (!Array.isArray(tabs)) {
      debugLog("browserTabs() did not return array");
      return results;
    }
    for (var i = 0; i < tabs.length; i++) {
      var t = tabs[i];
      var url = String(t.url || "").toLowerCase();
      var title = String(t.title || "").toLowerCase();
      if (isBillingTab(url, title)) {
        results.push({
          id: t.id || t.tabId || t.browserId || "",
          url: t.url || "",
          title: t.title || "",
        });
      }
    }
  } catch (e) {
    console.log("[WARN] Could not list browser tabs: " + e);
  }
  return results;
}

function extractPageText(instanceId, maxChars) {
  var text = "";
  var selectors = ["main", "[role='main']", "article", "#content", "body"];
  for (var i = 0; i < selectors.length; i++) {
    try {
      var json = floorp.tabElementText(instanceId, selectors[i]);
      var obj = safeJsonParse(json);
      var part = obj && obj.text ? obj.text : String(json || "");
      // Skip text that looks like unrendered JavaScript source code
      if (part.length > 50 && looksLikeJavaScript(part)) {
        debugLog(
          "extractPageText: skipped JS-like content from selector '" +
            selectors[i] +
            "' (" +
            part.length +
            " chars)",
        );
        continue;
      }
      if (part.length > text.length) text = part;
    } catch (e) {}
  }
  return text.length > maxChars ? text.slice(0, maxChars) : text;
}

/**
 * Detect if extracted text looks like raw JavaScript source code rather than
 * rendered page content. Used to skip unhydrated SPA body text.
 */
function looksLikeJavaScript(text) {
  if (!text || text.length < 50) return false;
  var prefix = text.substring(0, 300);
  // Common JS patterns at the start of text
  var jsPatterns = [
    /^\s*[\(\{\[]\s*[\(\{a-z]/, // starts with ((, ({, [( etc
    /=>\s*\{/, // arrow function
    /\bfunction\s*\(/, // function keyword
    /\bvar\s+\w+\s*=/, // var declaration
    /\blet\s+\w+\s*=/, // let declaration
    /\bconst\s+\w+\s*=/, // const declaration
    /\bdocument\.documentElement/, // DOM access
    /\bwindow\.__/, // webpack/bundler globals
    /\b__webpack/, // webpack internals
    /\bself\.__next/, // Next.js internals
  ];
  for (var i = 0; i < jsPatterns.length; i++) {
    if (jsPatterns[i].test(prefix)) return true;
  }
  return false;
}

/**
 * Source-specific text collection — uses custom selectors and page interactions
 * to extract richer data from known billing pages (like the original workflow).
 */
function collectSourceText(tabId, source, maxChars) {
  var selectors = source.selectors || {};
  var sourceId = source.id || "";

  debugLog("collectSourceText sourceId=" + sourceId);

  // Link.com / Stripe — click "show all inactive", then iterate list items
  if (sourceId === "link") {
    if (selectors.inactiveToggle) {
      try {
        debugLog("click inactiveToggle=" + selectors.inactiveToggle);
        floorp.tabClick(tabId, selectors.inactiveToggle);
        safeSleep(800);
      } catch (e) {}
    }
    // Collect list + detail pane text
    var listText = readSelectorText(tabId, selectors.list, "link.list");
    var detailText = readSelectorText(tabId, selectors.detail, "link.detail");
    var combined = listText + "\n---\n" + detailText;
    if (combined.length > 100) {
      return combined.length > maxChars
        ? combined.slice(0, maxChars)
        : combined;
    }
    // fallback to generic
  }

  // GitHub Copilot — combine plan card AND billing container for complete context
  if (sourceId === "github_copilot") {
    var planText = readSelectorText(
      tabId,
      selectors.planCard,
      "github.planCard",
    );
    var billingText = readSelectorText(
      tabId,
      selectors.billingContainer,
      "github.billingContainer",
    );
    // Truncate billingContainer — the full page (9000+ chars) can overwhelm small LLMs
    // Plan-relevant info (plan name, next billing, payment method) is in the first ~2000 chars
    var maxBillingChars = 2000;
    if (billingText && billingText.length > maxBillingChars) {
      debugLog("github: truncating billingContainer from " + billingText.length + " to " + maxBillingChars + " chars");
      billingText = billingText.slice(0, maxBillingChars);
    }
    // Always combine both: planCard has the plan name, billingContainer has details
    var combined = "";
    if (planText) combined += "[Copilot Plan Card]\n" + planText + "\n";
    if (billingText) combined += "[Billing Details]\n" + billingText;
    if (combined.length > 50) {
      return combined.length > maxChars
        ? combined.slice(0, maxChars)
        : combined;
    }
  }

  // Google Subscriptions — target main role
  if (sourceId === "google_subscriptions") {
    var mainText = readSelectorText(tabId, selectors.main, "google.main");
    if (mainText && mainText.length > 50) {
      return mainText.length > maxChars
        ? mainText.slice(0, maxChars)
        : mainText;
    }
  }

  // Z.ai — use tabpanel selector
  if (sourceId === "zai") {
    var zaiPanel = readSelectorText(
      tabId,
      "div[role='tabpanel']",
      "zai.tabpanel",
    );
    if (zaiPanel && zaiPanel.length > 50) {
      debugLog("Z.ai: tabpanel OK (" + zaiPanel.length + " chars)");
      // Prefix with source hint so LLM knows the service name
      var zaiPrefixed =
        "[Z.ai / GLM Coding subscription management page]\n" + zaiPanel;
      return zaiPrefixed.length > maxChars
        ? zaiPrefixed.slice(0, maxChars)
        : zaiPrefixed;
    }
  }

  // Cursor — SPA needs extra wait; try specific selectors, avoid JS source
  if (sourceId === "cursor") {
    // Extra wait for SPA hydration
    safeSleep(3000);
    debugLog("Cursor: extra SPA wait done, trying selectors...");
    // Try specific selectors
    var cursorMain = readSelectorText(
      tabId,
      selectors.settingsMain,
      "cursor.main",
    );
    if (
      cursorMain &&
      cursorMain.length > 100 &&
      !looksLikeJavaScript(cursorMain)
    ) {
      debugLog("Cursor: main selector OK (" + cursorMain.length + " chars)");
      return cursorMain.length > maxChars
        ? cursorMain.slice(0, maxChars)
        : cursorMain;
    }
    var cursorPlan = readSelectorText(
      tabId,
      selectors.planSection,
      "cursor.planSection",
    );
    if (
      cursorPlan &&
      cursorPlan.length > 50 &&
      !looksLikeJavaScript(cursorPlan)
    ) {
      debugLog("Cursor: plan selector OK (" + cursorPlan.length + " chars)");
      return cursorPlan.length > maxChars
        ? cursorPlan.slice(0, maxChars)
        : cursorPlan;
    }
    // Try generic selectors but skip JS-like content
    var cursorFallback = extractPageText(tabId, maxChars);
    if (cursorFallback.length > 100 && !looksLikeJavaScript(cursorFallback)) {
      return cursorFallback;
    }
    // Last resort: wait even longer
    debugLog("Cursor: content still JS-like, extra 5s wait...");
    safeSleep(5000);
    cursorFallback = extractPageText(tabId, maxChars);
    debugLog(
      "Cursor: after extra wait, " +
        cursorFallback.length +
        " chars, JS=" +
        looksLikeJavaScript(cursorFallback),
    );
    return cursorFallback;
  }

  // Generic fallback — extract best text from common selectors
  // For SPA sites (cursor, etc.), if first extraction looks like JS, wait and retry
  var fallbackText = extractPageText(tabId, maxChars);
  if (fallbackText.length < 100 || looksLikeJavaScript(fallbackText)) {
    debugLog(
      "Fallback text is short or JS-like (" +
        fallbackText.length +
        " chars), waiting 4s for SPA render...",
    );
    safeSleep(4000);
    fallbackText = extractPageText(tabId, maxChars);
    debugLog(
      "After SPA wait: " +
        fallbackText.length +
        " chars, JS-like=" +
        looksLikeJavaScript(fallbackText),
    );
  }
  return fallbackText;
}

function readSelectorText(tabId, selector, label) {
  if (!selector) {
    debugLog("missing selector" + (label ? " for " + label : ""));
    return "";
  }
  try {
    var json = floorp.tabElementText(tabId, selector);
    var obj = safeJsonParse(json);
    var text = obj && obj.text ? obj.text : String(json || "");
    debugLog(
      "selector ok" +
        (label ? " " + label : "") +
        " length=" +
        text.length +
        " selector=" +
        selector,
    );
    return text;
  } catch (e) {
    debugLog(
      "selector error" +
        (label ? " " + label : "") +
        " selector=" +
        selector +
        " error=" +
        e,
    );
    return "";
  }
}

/**
 * Manual fallback for GitHub Copilot when LLM completely fails.
 * Extracts plan info from known patterns in the planCard text.
 */
function manualGitHubCopilotFallback(pageText) {
  var lower = String(pageText || "").toLowerCase();

  // Known GitHub Copilot plans and their monthly USD prices (as of 2025/2026)
  var plans = [
    { pattern: "copilot pro+",  plan: "Pro+",       price: "$39" },
    { pattern: "pro+",          plan: "Pro+",       price: "$39" },
    { pattern: "copilot enterprise", plan: "Enterprise", price: "$39" },
    { pattern: "copilot business",   plan: "Business",   price: "$19" },
    { pattern: "copilot pro",   plan: "Pro",        price: "$10" },
    { pattern: "copilot free",  plan: "Free",       price: "$0" },
  ];

  var matched = null;
  for (var i = 0; i < plans.length; i++) {
    if (lower.indexOf(plans[i].pattern) !== -1) {
      matched = plans[i];
      break;
    }
  }
  if (!matched) return [];

  var notes = "";
  if (lower.indexOf("downgrade pending") !== -1) notes = "Downgrade Pending";
  if (lower.indexOf("upgrade pending") !== -1) notes = "Upgrade Pending";

  debugLog("manualGitHubCopilotFallback matched: " + matched.plan + " " + matched.price);
  return [{
    service: "GitHub Copilot",
    plan: matched.plan,
    price: matched.price,
    currency: "USD",
    billing_period: "monthly",
    next_billing_date: "",
    status: "active",
    notes: notes ? notes + " (manual extraction)" : "(manual extraction — LLM failed)",
  }];
}

function llmExtractSubscriptions(pageText, sourceUrl, sourceTitle) {
  var systemPrompt =
    "You extract the user's ACTUAL subscription/service entries from a billing or account page. " +
    "Return ONLY a JSON array. Each item MUST include these fields: " +
    "service (string: actual service name, NOT merchant/payment processor), " +
    "plan (string: plan tier name), " +
    "price (string: raw price including currency symbol), " +
    "currency (string: USD|EUR|JPY|GBP or symbol), " +
    "billing_period (string: monthly|yearly|one-time|unknown), " +
    "next_billing_date (string: YYYY-MM-DD or empty), " +
    "status (string: active|inactive|cancelled|trial|unknown), " +
    "notes (string: any extra detail). " +
    "CRITICAL RULES:\n" +
    "1. Only extract subscriptions the user ACTUALLY HAS — NOT plans listed for comparison or marketing.\n" +
    "2. If the page shows a plan comparison table (Free / Pro / Team / Enterprise), " +
    "   extract ONLY the plan that has indicators like 'Current plan', 'Your plan', " +
    "   'Active', a checkmark, or billing details (next bill date, payment method).\n" +
    "3. Do NOT extract model names (GPT-4, Claude Opus, Sonnet, Haiku) as separate subscriptions.\n" +
    "4. Do NOT extract usage/consumption line items (e.g. 'Actions: $0.62', 'Packages: $0.01') " +
    "   as subscriptions — these are metered usage, not separate plans.\n" +
    "5. Distinguish MERCHANT names (BUILDJET OÜ, OpenAI OpCo) from SERVICE names (Cursor, ChatGPT Plus).\n" +
    "6. If you cannot determine the user's actual plan from the page, return an empty array [].\n" +
    "7. Preserve currency symbols in price. If unknown, use empty string.\n" +
    "8. A settings/pricing page often shows MULTIPLE plan tiers the user COULD choose. " +
    "   This is NOT the same as the user having multiple subscriptions! " +
    "   Extract only ONE entry — the plan the user is currently on. " +
    "   Look for indicators: 'Current plan', 'Your plan', billing date, 'Manage', amount charged. " +
    "   Plans labelled 'Upgrade', 'Get started', or with a 'Subscribe' button are NOT the user's plan.\n" +
    "9. For Cursor settings pages: the user can only have ONE active plan (Hobby/Pro/Pro+/UltraPro/Business). " +
    "   Look for 'Current Plan' or which tier shows a billing period like 'Jan 15 - Feb 13'. Return only that one.\n" +
    "10. Use the EXACT plan name shown on the page. If the page says 'Pro Plan', the plan is 'Pro', NOT 'UltraPro'. " +
    "    Do NOT guess or upgrade the plan name. Copy it verbatim from the displayed text.";

  debugLog(
    "LLM extract input preview (" +
      sourceUrl +
      "): " +
      pageText.substring(0, 300).replace(/\n/g, " "),
  );

  // Helper: attempt LLM extraction with given prompt text
  function tryLlmExtract(promptText, attempt) {
    var up = "Page URL: " + sourceUrl + "\n" +
      "Page title: " + sourceTitle + "\n" +
      "Page text:\n" + promptText;
    var r = llmChat(systemPrompt, up);
    debugLog(
      "LLM extract raw response" + (attempt > 1 ? " (retry " + attempt + ")" : "") +
        " (" + sourceUrl + "): " + String(r || "").substring(0, 500),
    );
    var j = extractJsonArray(r);
    if (!j) return null;
    var p = safeJsonParse(j);
    if (!Array.isArray(p)) return null;
    return p;
  }

  // First attempt with full text
  var parsed = tryLlmExtract(pageText, 1);

  // If first attempt failed and input was long, retry with truncated text
  if ((!parsed || parsed.length === 0) && pageText.length > 2500) {
    var shortLen = Math.min(2000, Math.floor(pageText.length / 2));
    debugLog("LLM retry with truncated input (" + shortLen + " chars from " + pageText.length + ")...");
    safeSleep(1200);
    parsed = tryLlmExtract(pageText.substring(0, shortLen), 2);
  }

  if (!parsed) {
    console.log("  [WARN] LLM returned no valid JSON for " + sourceUrl);
    return [];
  }
  console.log("  Extracted " + parsed.length + " entries from " + sourceUrl);
  if (parsed.length === 0) {
    debugLog(
      "LLM returned empty array [] for " +
        sourceUrl +
        " — page had " +
        pageText.length +
        " chars",
    );
  }
  return parsed;
}

// ============================================================================
// 2c. Optional PDF Invoice Processing
// ============================================================================

function processInvoicePDFs() {
  var hasOcr =
    typeof ocr !== "undefined" && typeof ocr.extract_text === "function";
  if (!hasOcr) {
    debugLog("OCR plugin not available, skipping PDF invoices");
    return [];
  }

  var subscriptions = [];
  var pdfCount = 0;

  for (var i = 0; i < CONFIG.invoiceSearchPaths.length; i++) {
    var searchPath = CONFIG.invoiceSearchPaths[i];
    try {
      var foundRaw = app.sapphillon.core.search.file(
        searchPath,
        "invoice OR receipt OR subscription",
      );
      var found = safeJsonParse(foundRaw);
      if (!Array.isArray(found)) continue;

      for (var j = 0; j < found.length && pdfCount < CONFIG.maxInvoices; j++) {
        var filePath = found[j];
        if (!filePath || !String(filePath).toLowerCase().endsWith(".pdf"))
          continue;

        try {
          var text = ocr.extract_text(filePath);
          if (!text || text.length < 30) continue;

          pdfCount++;
          auditLog("read_pdf_invoice", filePath);

          var entries = llmExtractFromInvoice(text, filePath);
          if (entries && entries.length) {
            for (var k = 0; k < entries.length; k++) {
              entries[k].source_url = "file://" + filePath;
              entries[k].source_type = "pdf_invoice";
            }
            subscriptions = subscriptions.concat(entries);
          }
        } catch (e) {
          debugLog("PDF extraction failed: " + filePath + " — " + e);
        }
      }
    } catch (e) {
      debugLog("Invoice search failed in " + searchPath + ": " + e);
    }
  }

  if (pdfCount > 0) {
    console.log(
      "[Step 2c] Processed " +
        pdfCount +
        " PDF invoice(s), found " +
        subscriptions.length +
        " entries",
    );
  }
  return subscriptions;
}

function llmExtractFromInvoice(text, filePath) {
  var systemPrompt =
    "Extract subscription/payment entries from this invoice/receipt text. " +
    "Return ONLY a JSON array with fields: service, plan, price, currency, " +
    "billing_period, next_billing_date, status, notes. " +
    "Note this is from an invoice/receipt so confidence may be lower.";

  var userPrompt = "File: " + filePath + "\nText:\n" + text.slice(0, 8000);

  var raw = llmChat(systemPrompt, userPrompt);
  var json = extractJsonArray(raw);
  var parsed = safeJsonParse(json);
  return Array.isArray(parsed) ? parsed : [];
}

// ============================================================================
// 3. Normalisation, Currency Conversion, Deduplication
// ============================================================================

function normaliseSubscriptions(entries) {
  var normalised = [];
  for (var i = 0; i < entries.length; i++) {
    var it = entries[i] || {};
    var priceInfo = parsePrice(String(it.price || ""));
    var currency =
      priceInfo.currency || normaliseCurrencyCode(String(it.currency || ""));
    var period = normalisePeriod(
      String(it.billing_period || priceInfo.period || ""),
    );

    // Convert to JPY if possible
    var priceJpy = convertToJpy(priceInfo.amount, currency);
    var monthlyJpy = estimateMonthlyJpy(priceJpy, period);

    normalised.push({
      service: String(it.service || "").trim(),
      plan: String(it.plan || "").trim(),
      price_original: priceInfo.amount,
      currency_original: currency,
      price_jpy: priceJpy,
      monthly_jpy: monthlyJpy,
      billing_period: period,
      next_billing_date: String(it.next_billing_date || "").trim(),
      status: normaliseStatus(String(it.status || "")),
      source_url: String(it.source_url || ""),
      source_type: String(it.source_type || ""),
      source_title: String(it.source_title || ""),
      notes: String(it.notes || "").trim(),
      confidence: calculateConfidence(it),
      raw_price: String(it.price || "").trim(),
    });
  }
  return normalised;
}

function parsePrice(text) {
  var cleaned = text.trim();
  var match = cleaned.match(/([\$€¥£￥])?\s*([0-9,]+(?:\.[0-9]+)?)/);
  if (!match) return { amount: 0, currency: "", period: "" };

  var symbol = match[1] || "";
  var amount = parseFloat(String(match[2] || "0").replace(/,/g, "")) || 0;
  var currency = "";

  if (symbol === "$" || cleaned.indexOf("USD") !== -1) currency = "USD";
  else if (symbol === "€" || cleaned.indexOf("EUR") !== -1) currency = "EUR";
  else if (
    symbol === "¥" ||
    symbol === "￥" ||
    cleaned.indexOf("JPY") !== -1 ||
    cleaned.indexOf("円") !== -1
  )
    currency = "JPY";
  else if (symbol === "£" || cleaned.indexOf("GBP") !== -1) currency = "GBP";

  var period = "";
  if (/年|year|annual|yr/i.test(cleaned)) period = "yearly";
  else if (/月|month|mo/i.test(cleaned)) period = "monthly";

  return { amount: amount, currency: currency, period: period };
}

function normaliseCurrencyCode(code) {
  var c = code.toUpperCase().trim();
  if (c === "$" || c === "US$") return "USD";
  if (c === "€") return "EUR";
  if (c === "¥" || c === "￥" || c === "円") return "JPY";
  if (c === "£") return "GBP";
  if (["USD", "EUR", "JPY", "GBP"].indexOf(c) !== -1) return c;
  return c || "USD";
}

function normalisePeriod(period) {
  var p = String(period || "").toLowerCase();
  if (/年|year|annual/.test(p)) return "yearly";
  if (/月|month|mo/.test(p)) return "monthly";
  if (/一回|one.?time|lifetime/.test(p)) return "one-time";
  if (/日/.test(p)) return ""; // likely parse error
  return p || "unknown";
}

function normaliseStatus(status) {
  var s = String(status || "").toLowerCase();
  if (/active|アクティブ/.test(s)) return "active";
  if (/inactive|cancel|キャンセル|解約|停止|非アクティブ/.test(s))
    return "inactive";
  if (/trial|トライアル|お試し/.test(s)) return "trial";
  return s || "unknown";
}

function fetchExchangeRates() {
  // Try live rate
  try {
    var raw = app.sapphillon.core.fetch.fetch(
      "https://api.exchangerate-api.com/v4/latest/USD",
    );
    var data = safeJsonParse(raw);
    if (data && data.rates && data.rates.JPY) {
      EXCHANGE_RATES = data.rates;
      auditLog(
        "fetch_exchange_rates",
        "Live rates fetched (USD/JPY=" + data.rates.JPY + ")",
      );
      return;
    }
  } catch (e) {
    debugLog("Live exchange rate fetch failed: " + e);
  }

  // Fallback to hardcoded approximate rates (same structure as live API: base=USD)
  EXCHANGE_RATES = {
    JPY: CONFIG.fallbackRates.USD || 150, // 1 USD = 150 JPY
    EUR: CONFIG.fallbackRates.EUR
      ? CONFIG.fallbackRates.USD / CONFIG.fallbackRates.EUR
      : 0.92,
    GBP: CONFIG.fallbackRates.GBP
      ? CONFIG.fallbackRates.USD / CONFIG.fallbackRates.GBP
      : 0.79,
    USD: 1,
  };
  auditLog(
    "exchange_rates_fallback",
    "Using approximate rates (USD/JPY=" + CONFIG.fallbackRates.USD + ")",
  );
}

function convertToJpy(amount, currency) {
  if (!amount || amount === 0) return 0;
  if (currency === "JPY") return Math.round(amount);

  // EXCHANGE_RATES may come from two different structures:
  //   Live API (base=USD): { JPY: 150, EUR: 0.92, GBP: 0.79, ... }
  //   Fallback:            same structure, from CONFIG.fallbackRates
  // In both cases, the value for each currency is "1 USD = X units".

  var jpyPerUsd = EXCHANGE_RATES.JPY || CONFIG.fallbackRates.USD || 150;

  if (currency === "USD") {
    return Math.round(amount * jpyPerUsd);
  }

  // For other currencies, convert to USD first, then to JPY
  var unitsPerUsd = EXCHANGE_RATES[currency];
  if (unitsPerUsd && unitsPerUsd > 0) {
    // amount is in <currency>; 1 USD = unitsPerUsd <currency>
    var amountInUsd = amount / unitsPerUsd;
    return Math.round(amountInUsd * jpyPerUsd);
  }

  // Last resort: use fallback rates (stored as JPY per 1 unit of currency)
  var fallback = CONFIG.fallbackRates[currency];
  if (fallback && fallback > 0) {
    return Math.round(amount * fallback);
  }

  // Completely unknown currency — treat as USD
  debugLog("Unknown currency '" + currency + "', treating as USD");
  return Math.round(amount * jpyPerUsd);
}

function estimateMonthlyJpy(jpyAmount, period) {
  if (!jpyAmount || jpyAmount === 0) return 0;
  if (period === "yearly") return Math.round(jpyAmount / 12);
  if (period === "monthly") return jpyAmount;
  return jpyAmount; // unknown or one-time → treat as-is
}

function calculateConfidence(entry) {
  var score = 0.3; // base
  if (entry.service && entry.service.length > 1) score += 0.15;
  if (entry.price && String(entry.price).match(/[0-9]/)) score += 0.2;
  if (entry.billing_period && entry.billing_period !== "unknown") score += 0.1;
  if (entry.next_billing_date) score += 0.1;
  if (entry.source_type === "browser_billing_page") score += 0.1;
  if (entry.source_type === "pdf_invoice") score -= 0.05;
  if (entry.status === "active") score += 0.05;
  return Math.min(1.0, Math.round(score * 100) / 100);
}

function deduplicateSubscriptions(subs) {
  if (!Array.isArray(subs) || !subs.length) return [];
  var seen = {};
  var result = [];

  // Helper: normalise service name for dedup key
  // "Cursor Pro" → "cursor", "GitHub Copilot" → "githubcopilot"
  function normService(name) {
    return String(name || "")
      .toLowerCase()
      .replace(
        /\s*(pro\+?|ultra\s*pro|business|enterprise|team|plus|free|hobby)\s*/gi,
        "",
      )
      .replace(/\s+/g, "")
      .trim();
  }

  for (var i = 0; i < subs.length; i++) {
    var s = subs[i] || {};
    // Primary key: normalised service + plan + price + currency + period
    var key =
      normService(s.service) +
      "|" +
      String(s.plan || "")
        .toLowerCase()
        .replace(/\s+/g, "") +
      "|" +
      String(s.price_original || "") +
      "|" +
      String(s.currency_original || "").toLowerCase() +
      "|" +
      String(s.billing_period || "").toLowerCase();

    // Secondary key: same service + same monthly JPY price (catches plan name mismatches)
    var priceKey = normService(s.service) + "|" + (s.price_jpy || 0);

    if (!seen[key] && !seen["price:" + priceKey]) {
      seen[key] = s;
      if (s.price_jpy > 0) seen["price:" + priceKey] = s;
      result.push(s);
    } else {
      // Keep the one with higher confidence
      var existing = seen[key] || seen["price:" + priceKey];
      debugLog(
        "Dedup: merging duplicate " +
          s.service +
          " (" +
          s.plan +
          ") with existing " +
          existing.service +
          " (" +
          existing.plan +
          ")",
      );
      if ((s.confidence || 0) > (existing.confidence || 0)) {
        for (var j = 0; j < result.length; j++) {
          if (result[j] === existing) {
            result[j] = s;
            break;
          }
        }
        seen[key] = s;
        if (s.price_jpy > 0) seen["price:" + priceKey] = s;
      }
    }
  }
  return result;
}

// ============================================================================
// 3.5  Web Exploration — DuckDuckGo Search + Page Visits
// ============================================================================

/**
 * Search DuckDuckGo and collect result URLs.
 * Pattern adapted from deep_research_ddg.js.
 */
function ddgSearch(query, maxResults) {
  maxResults = maxResults || CONFIG.webExplore.resultsPerSearch || 5;
  var results = [];
  var ddgTab = null;

  try {
    var ddgUrl = "https://duckduckgo.com/?q=" + encodeURIComponent(query);
    ddgTab = floorp.createTab(ddgUrl, false);
    floorp.tabWaitForElement(ddgTab, "article[data-testid='result']", 15000);
    safeSleep(3000);

    // Click "More Results" once to load extra results
    try {
      floorp.tabClick(ddgTab, "#more-results");
      safeSleep(2000);
    } catch (e) {}

    // Collect results from the result list
    for (var i = 1; i <= maxResults + 10; i++) {
      if (results.length >= maxResults) break;

      var baseSel =
        "ol.react-results--main > li:nth-child(" +
        i +
        ") article[data-testid='result']";
      try {
        var titleSel = baseSel + " a[data-testid='result-title-a']";
        var titleJson = floorp.tabElementText(ddgTab, titleSel);
        var titleObj = safeJsonParse(titleJson);
        var title = titleObj && titleObj.text ? titleObj.text : "";
        if (!title) continue;

        var linkJson = floorp.tabAttribute(ddgTab, titleSel, "href");
        var linkObj = safeJsonParse(linkJson);
        var url = linkObj && linkObj.value ? linkObj.value : "";
        if (!url) continue;

        // Skip social media, video sites
        if (
          url.indexOf("youtube.com") !== -1 ||
          url.indexOf("twitter.com") !== -1 ||
          url.indexOf("facebook.com") !== -1 ||
          url.indexOf("reddit.com") !== -1
        )
          continue;

        // Get snippet
        var snippet = "";
        try {
          var snipJson = floorp.tabElementText(ddgTab, baseSel);
          var snipObj = safeJsonParse(snipJson);
          snippet = snipObj && snipObj.text ? snipObj.text : "";
        } catch (e) {}

        results.push({
          title: String(title).trim(),
          url: url,
          snippet: String(snippet).substring(0, 300),
          domain: extractUrlDomain(url),
        });
      } catch (e) {}
    }
  } catch (e) {
    debugLog("DuckDuckGo search failed for '" + query + "': " + e);
  } finally {
    if (ddgTab) {
      try {
        floorp.closeTab(ddgTab);
      } catch (e2) {}
    }
  }

  return results;
}

function extractUrlDomain(url) {
  try {
    var m = String(url).match(/https?:\/\/([^\/]+)/);
    return m ? m[1] : "";
  } catch (e) {
    return "";
  }
}

/**
 * Visit a URL in a new tab, extract text, close tab.
 * Returns extracted text or empty string.
 */
function visitAndExtractContent(url, maxChars) {
  maxChars = maxChars || CONFIG.webExplore.maxPageChars || 6000;
  var pageTab = null;
  var text = "";

  try {
    pageTab = floorp.createTab(url, false);
    try {
      floorp.tabWaitForNetworkIdle(pageTab);
    } catch (e) {}
    safeSleep(CONFIG.webExplore.visitDelay || 1500);

    // Try several selectors to get the best content
    var selectors = [
      "main",
      "[role='main']",
      "article",
      "#content",
      ".pricing",
      ".plans",
      "body",
    ];
    for (var i = 0; i < selectors.length; i++) {
      try {
        var json = floorp.tabElementText(pageTab, selectors[i]);
        var obj = safeJsonParse(json);
        var part = obj && obj.text ? obj.text : "";
        if (part.length > text.length) text = part;
      } catch (e) {}
    }
  } catch (e) {
    debugLog("visitAndExtractContent failed for " + url + ": " + e);
  } finally {
    if (pageTab) {
      try {
        floorp.closeTab(pageTab);
      } catch (e2) {}
    }
  }

  return text.length > maxChars ? text.slice(0, maxChars) : text;
}

/**
 * Identify subscriptions & apps that need web research.
 * Returns { pricingQueries: [...], appQueries: [...] }.
 */
function identifyWebResearchCandidates(subscriptions, apps) {
  var pricingQueries = [];
  var appQueries = [];
  var year = new Date().getFullYear();
  var seenServiceKeys = {};

  // --- Subscriptions needing price/plan verification ---
  for (var i = 0; i < subscriptions.length; i++) {
    var sub = subscriptions[i];
    var svcKey = String(sub.service || "")
      .toLowerCase()
      .replace(/\s+/g, "");
    if (seenServiceKeys[svcKey]) continue; // one search per service
    seenServiceKeys[svcKey] = true;

    var needsResearch = false;
    var reason = "";

    if (sub.price_jpy === 0 && sub.status !== "inactive") {
      needsResearch = true;
      reason = "price unknown/zero";
    } else if (sub.confidence < 0.7) {
      needsResearch = true;
      reason = "low confidence (" + sub.confidence + ")";
    } else if (
      String(sub.plan || "").toLowerCase() === "unknown" ||
      !sub.plan
    ) {
      needsResearch = true;
      reason = "plan unknown";
    }

    if (needsResearch) {
      pricingQueries.push({
        type: "pricing",
        service: sub.service,
        plan: sub.plan || "",
        query: sub.service + " pricing plans " + year,
        reason: reason,
        source_url: sub.source_url,
      });
    }
  }

  // --- Apps that may be subscription-based but not detected ---
  // AI coding tools that may be subscription-based
  var knownSubApps = [
    "copilot",
    "cursor",
    "windsurf",
    "codeium",
    "tabnine",
    "supermaven",
    "continue",
    "aider",
    "sourcegraph",
    "cody",
    "jetbrains",
    "replit",
    "augment",
    "devin",
    "amazonq",
    "codewhisperer",
    "v0",
    "github",
    "openai",
    "chatgpt",
    "claude",
    "anthropic",
  ];

  var detectedServiceNames = {};
  for (var j = 0; j < subscriptions.length; j++) {
    detectedServiceNames[
      String(subscriptions[j].service || "")
        .toLowerCase()
        .replace(/\s+/g, "")
    ] = true;
  }

  for (var k = 0; k < apps.length; k++) {
    var appName = String(apps[k].name || "").toLowerCase();
    if (!appName) continue;

    // Check if this app matches known subscription apps
    var matches = false;
    for (var m = 0; m < knownSubApps.length; m++) {
      if (appName.indexOf(knownSubApps[m]) !== -1) {
        matches = true;
        break;
      }
    }
    if (!matches) continue;

    // Check if already detected in subscriptions
    var alreadyDetected = false;
    var appKey = appName.replace(/\s+/g, "");
    var dKeys = Object.keys(detectedServiceNames);
    for (var d = 0; d < dKeys.length; d++) {
      if (dKeys[d].indexOf(appKey) !== -1 || appKey.indexOf(dKeys[d]) !== -1) {
        alreadyDetected = true;
        break;
      }
    }

    if (!alreadyDetected) {
      appQueries.push({
        type: "app_subscription_check",
        appName: apps[k].name,
        usage: apps[k].usageBucket,
        query: apps[k].name + " subscription pricing free vs paid " + year,
        reason: "installed app, possible subscription not detected",
      });
    }
  }

  // --- Dedup app queries by normalised core name ---
  // "ChatGPT Atlas" → "chatgpt", "GitHub Copilot for Xcode" → "githubcopilot"
  // "github-copilot-for-xcode" → "githubcopilot"
  function coreAppName(name) {
    var n = String(name || "")
      .toLowerCase()
      .replace(/-/g, " ") // kebab → spaces
      .replace(/\.(app|exe)$/i, "") // remove extension
      .replace(/\s+/g, " ")
      .trim();
    // Strip common suffixes: "for xcode", "for macos", "for mac", "atlas", "desktop"
    n = n.replace(
      /\s+(for\s+\w+|atlas|desktop|preview|beta|alpha|lite|pro|plus)$/i,
      "",
    );
    // Remove all remaining spaces for comparison key
    return n.replace(/\s+/g, "");
  }

  var seenCoreNames = {};
  var dedupedAppQueries = [];
  for (var q = 0; q < appQueries.length; q++) {
    var core = coreAppName(appQueries[q].appName);
    if (seenCoreNames[core]) {
      debugLog(
        "Web explore dedup: skipped '" +
          appQueries[q].appName +
          "' (core='" +
          core +
          "' already queued)",
      );
      continue;
    }
    seenCoreNames[core] = true;
    dedupedAppQueries.push(appQueries[q]);
  }

  debugLog(
    "Web research candidates: " +
      pricingQueries.length +
      " pricing, " +
      dedupedAppQueries.length +
      " app checks (deduped from " +
      appQueries.length +
      ")",
  );
  return { pricingQueries: pricingQueries, appQueries: dedupedAppQueries };
}

/**
 * Use LLM to extract structured pricing data from a web page.
 * Returns { plans: [...], verified_price: ..., ... } or null.
 */
function llmExtractPricingFromPage(pageText, serviceName, contextInfo) {
  if (!pageText || pageText.length < 50) return null;

  var systemPrompt =
    "You are a pricing data extractor. Given text from a pricing/plans page, " +
    "extract ALL available plans with their prices. " +
    "Return ONLY a JSON object with these fields:\n" +
    "{\n" +
    '  "service_name": "string",\n' +
    '  "plans": [\n' +
    "    {\n" +
    '      "name": "plan tier name",\n' +
    '      "price_monthly": "monthly price string with currency",\n' +
    '      "price_yearly": "yearly price string with currency (if available)",\n' +
    '      "is_free": boolean,\n' +
    '      "features_summary": "key features in ≤50 chars"\n' +
    "    }\n" +
    "  ],\n" +
    '  "currency": "USD|EUR|JPY|GBP",\n' +
    '  "is_subscription_service": boolean,\n' +
    '  "notes": "any relevant details"\n' +
    "}\n" +
    'If the page does not contain pricing info, return {"plans":[], "is_subscription_service": false}.';

  var userPrompt =
    "Service we're looking up: " +
    serviceName +
    "\n" +
    (contextInfo ? "Context: " + contextInfo + "\n" : "") +
    "Page text:\n" +
    pageText;

  var raw = llmChat(systemPrompt, userPrompt);
  var json = extractJsonObject(raw);
  return safeJsonParse(json);
}

/**
 * Determine whether an app is subscription-based from web results.
 */
function llmCheckAppSubscription(pageText, appName) {
  if (!pageText || pageText.length < 50) return null;

  var systemPrompt =
    "Determine whether the software '" +
    appName +
    "' is a paid subscription service, " +
    "a one-time purchase, or free. Return ONLY a JSON object:\n" +
    "{\n" +
    '  "app_name": "string",\n' +
    '  "is_subscription": boolean,\n' +
    '  "is_free": boolean,\n' +
    '  "price_monthly": "string or null",\n' +
    '  "price_yearly": "string or null",\n' +
    '  "free_tier_available": boolean,\n' +
    '  "plan_name": "string or null (the likely paid plan)",\n' +
    '  "notes": "brief summary"\n' +
    "}";

  var raw = llmChat(
    systemPrompt,
    "App: " + appName + "\nPage text:\n" + pageText,
  );
  var json = extractJsonObject(raw);
  return safeJsonParse(json);
}

// ============================================================================
// AI Coding Tool Verification via DDG (Filtering)
// ============================================================================

/**
 * Check if an app name is obviously an AI coding tool based on known keywords.
 * Returns true if the name matches any keyword in CONFIG.knownAiToolKeywords.
 */
function isDefinitelyNotAiTool(name) {
  var lower = String(name || "").toLowerCase();
  // Check exact-match short names first (prevents "line" matching "cline")
  var exactNames = CONFIG.notAiToolExactNames || [];
  for (var e = 0; e < exactNames.length; e++) {
    if (lower === exactNames[e]) return true;
  }
  // Check substring negatives
  var negatives = CONFIG.notAiToolKeywords || [];
  for (var i = 0; i < negatives.length; i++) {
    if (lower.indexOf(negatives[i]) !== -1) return true;
  }
  return false;
}

function isObviouslyAiTool(name) {
  if (isDefinitelyNotAiTool(name)) return false;
  var lower = String(name || "").toLowerCase();
  var keywords = CONFIG.knownAiToolKeywords || [];
  for (var i = 0; i < keywords.length; i++) {
    if (lower.indexOf(keywords[i]) !== -1) return true;
  }
  return false;
}

/**
 * DDG search + LLM to determine if an app/service is an AI coding tool.
 * Returns { is_ai_coding_tool: boolean, confidence: number, reason: string }
 */
function ddgVerifyIsAiCodingTool(appName) {
  var query = appName + " AI coding tool software what is";
  debugLog("AI-filter DDG search: " + query);

  var searchResults = ddgSearch(query, 3);
  if (!searchResults.length) {
    debugLog("  No DDG results for '" + appName + "', assuming NOT AI tool");
    return {
      is_ai_coding_tool: false,
      confidence: 0.3,
      reason: "no search results",
    };
  }

  // Visit the top result to get more context
  var pageText = "";
  var maxVisits = CONFIG.aiFilterMaxVisitsPerSearch || 1;
  for (var i = 0; i < searchResults.length && i < maxVisits; i++) {
    var text = visitAndExtractContent(searchResults[i].url, 3000);
    if (text.length > pageText.length) pageText = text;
  }

  // Combine snippets + page text for LLM
  var snippets = "";
  for (var s = 0; s < searchResults.length; s++) {
    snippets += searchResults[s].title + ": " + searchResults[s].snippet + "\n";
  }

  var context = "Search snippets:\n" + snippets;
  if (pageText.length > 100) {
    context += "\nPage text:\n" + pageText.substring(0, 2000);
  }

  var systemPrompt =
    "You are classifying software applications. Given information about an app, " +
    "determine whether it is an AI-powered coding assistant, AI code editor, " +
    "AI code completion tool, AI code generation tool, or AI developer tool.\n" +
    "Return ONLY a JSON object:\n" +
    "{\n" +
    '  "app_name": "string",\n' +
    '  "is_ai_coding_tool": boolean,\n' +
    '  "confidence": number (0.0-1.0),\n' +
    '  "category": "string (e.g. AI code completion, AI code editor, AI chat for coding, not AI, general app)",\n' +
    '  "reason": "brief explanation in 1-2 sentences"\n' +
    "}\n" +
    "CRITICAL CLASSIFICATION RULES:\n" +
    "1. Only return is_ai_coding_tool=true if the app's PRIMARY PURPOSE is AI-assisted coding/programming.\n" +
    "2. An app that merely CONTAINS AI features (like Copilot in Edge, AI in OneNote, AI in Sheets) " +
    "   is NOT an AI coding tool. The PRIMARY function must be writing/generating code.\n" +
    "3. Web browsers (Edge, Chrome, Brave, Arc) are NOT AI coding tools even if they have AI sidebars.\n" +
    "4. Note-taking apps, email clients, spreadsheets, presentations, music/video software are NEVER AI coding tools.\n" +
    "5. General-purpose AI assistants (ChatGPT, Claude) count as AI coding tools ONLY if accessed via a " +
    "   dedicated coding interface (e.g. Claude Code CLI). ChatGPT.app and Claude.app are AI chat apps, not coding tools.\n" +
    "6. Developer utilities (Git clients, database tools, API testers, terminal emulators) are NOT AI coding tools " +
    "   even though developers use them.\n" +
    "7. IDEs/code editors WITHOUT built-in AI code generation (e.g. vanilla VS Code, Sublime Text, Vim) are NOT AI coding tools.\n" +
    "Examples of TRUE AI coding tools: GitHub Copilot, Cursor, Windsurf/Codeium, Tabnine, Supermaven, Cline, Aider, Devin.\n" +
    "Examples of NOT AI coding tools: Microsoft Edge, OneNote, Docker, Postman, Xcode, VS Code (without Copilot), Notion, Thunderbird, Ollama.\n" +
    "8. Email clients (Thunderbird, Outlook, Mail) that integrate with AI chatbots are NOT AI coding tools.\n" +
    "9. Local LLM runners (Ollama, LM Studio, GPT4All) are infrastructure tools, NOT AI coding tools " +
    "   unless they specifically provide code completion or editor integration as their primary feature.";

  var userPrompt = "App name: " + appName + "\n" + context;

  var raw = llmChat(systemPrompt, userPrompt);
  var json = extractJsonObject(raw);
  var result = safeJsonParse(json);

  if (result && typeof result.is_ai_coding_tool === "boolean") {
    return result;
  }

  // Fallback: if LLM fails, check snippets for AI keywords heuristically
  var combinedText = (snippets + " " + pageText).toLowerCase();
  var aiHits = 0;
  var aiKeywords = [
    "ai cod",
    "code completion",
    "code generation",
    "ai assistant",
    "copilot",
    "llm",
    "machine learning",
    "ai-powered",
    "code editor ai",
  ];
  for (var k = 0; k < aiKeywords.length; k++) {
    if (combinedText.indexOf(aiKeywords[k]) !== -1) aiHits++;
  }
  var isAi = aiHits >= 2;
  return {
    is_ai_coding_tool: isAi,
    confidence: isAi ? 0.5 : 0.4,
    reason: "heuristic fallback (" + aiHits + " AI keyword hits)",
  };
}

/**
 * Filter local apps: keep only those that are AI coding tools.
 * - Apps matching knownAiToolKeywords pass immediately.
 * - Ambiguous apps are verified via DDG search + LLM.
 * Returns filtered apps array.
 */
function filterLocalAppsToAiTools(apps) {
  if (!apps || !apps.length) return [];

  var passed = [];
  var needVerification = [];

  for (var i = 0; i < apps.length; i++) {
    if (isDefinitelyNotAiTool(apps[i].name)) {
      // Excluded by negative keyword — skip entirely, don't even DDG-verify
      continue;
    } else if (isObviouslyAiTool(apps[i].name)) {
      passed.push(apps[i]);
    } else {
      needVerification.push(apps[i]);
    }
  }

  console.log(
    "  [AI Filter] " +
      passed.length +
      " obvious AI tools, " +
      needVerification.length +
      " ambiguous apps to verify",
  );

  // DDG-verify ambiguous apps (up to aiFilterMaxSearches)
  var maxSearches = CONFIG.aiFilterMaxSearches || 10;
  var verified = 0;
  for (var j = 0; j < needVerification.length && verified < maxSearches; j++) {
    var app = needVerification[j];
    console.log("    [Verify] " + app.name + "...");

    var result = ddgVerifyIsAiCodingTool(app.name);
    verified++;

    if (result.is_ai_coding_tool) {
      console.log("      → YES: " + (result.reason || ""));
      passed.push(app);
    } else {
      console.log("      → NO (excluded): " + (result.reason || ""));
      auditLog(
        "ai_filter_excluded_app",
        app.name + " — " + (result.reason || "not AI coding tool"),
      );
    }

    safeSleep(CONFIG.aiFilterSearchDelay || 2500);
  }

  // Apps beyond the max search limit are excluded by default
  if (needVerification.length > maxSearches) {
    var skipped = needVerification.length - maxSearches;
    console.log(
      "    [AI Filter] " + skipped + " apps skipped (search limit), excluded",
    );
    auditLog("ai_filter_skipped", skipped + " apps beyond search limit");
  }

  return passed;
}

/**
 * Filter subscriptions: keep only those that are AI coding tools.
 * Same logic as filterLocalAppsToAiTools but for subscription entries.
 * Returns filtered subscriptions array.
 */
function filterSubscriptionsToAiTools(subscriptions) {
  if (!subscriptions || !subscriptions.length) return [];

  var passed = [];
  var needVerification = [];

  for (var i = 0; i < subscriptions.length; i++) {
    var svc = subscriptions[i].service || "";
    if (isDefinitelyNotAiTool(svc)) {
      auditLog("ai_filter_excluded_sub", svc + " — excluded by notAiTool list");
      continue;
    }
    if (isObviouslyAiTool(svc)) {
      passed.push(subscriptions[i]);
    } else {
      needVerification.push(subscriptions[i]);
    }
  }

  console.log(
    "  [AI Filter] " +
      passed.length +
      " obvious AI subscriptions, " +
      needVerification.length +
      " ambiguous subscriptions to verify",
  );

  var maxSearches = CONFIG.aiFilterMaxSearches || 10;
  var verified = 0;
  for (var j = 0; j < needVerification.length && verified < maxSearches; j++) {
    var sub = needVerification[j];
    console.log("    [Verify] " + sub.service + "...");

    var result = ddgVerifyIsAiCodingTool(sub.service);
    verified++;

    if (result.is_ai_coding_tool) {
      console.log("      → YES: " + (result.reason || ""));
      passed.push(sub);
    } else {
      console.log("      → NO (excluded): " + (result.reason || ""));
      auditLog(
        "ai_filter_excluded_sub",
        sub.service + " — " + (result.reason || "not AI coding tool"),
      );
    }

    safeSleep(CONFIG.aiFilterSearchDelay || 2500);
  }

  if (needVerification.length > maxSearches) {
    var skipped = needVerification.length - maxSearches;
    console.log(
      "    [AI Filter] " +
        skipped +
        " subscriptions skipped (search limit), excluded",
    );
    auditLog(
      "ai_filter_skipped_subs",
      skipped + " subscriptions beyond search limit",
    );
  }

  return passed;
}

/**
 * Main web exploration phase:
 * 1. Identify candidates needing research
 * 2. DuckDuckGo search for each
 * 3. Visit top results, extract pricing
 * 4. Merge findings back into subscriptions
 *
 * Returns { subscriptions, newCandidates, findings }
 */
function webExploreAndVerify(subscriptions, apps) {
  if (!CONFIG.webExplore.enabled) {
    debugLog("Web exploration disabled in config");
    return { subscriptions: subscriptions, newCandidates: [], findings: [] };
  }

  auditLog("start_web_exploration", "Identifying candidates for web research");
  var candidates = identifyWebResearchCandidates(subscriptions, apps);
  var allQueries = candidates.pricingQueries.concat(candidates.appQueries);

  // Limit total queries
  var maxSearches = CONFIG.webExplore.maxSearches || 10;
  if (allQueries.length > maxSearches) {
    // Prioritise: pricing queries (unknowns) first, then app queries
    allQueries = allQueries.slice(0, maxSearches);
  }

  if (allQueries.length === 0) {
    console.log("[Step 3.5] No candidates need web research — skipping.");
    return { subscriptions: subscriptions, newCandidates: [], findings: [] };
  }

  console.log(
    "[Step 3.5] Web-exploring " +
      allQueries.length +
      " queries (" +
      candidates.pricingQueries.length +
      " pricing + " +
      candidates.appQueries.length +
      " app checks)...",
  );

  var totalVisits = 0;
  var maxTotalVisits = CONFIG.webExplore.maxTotalVisits || 30;
  var maxVisitsPerSearch = CONFIG.webExplore.maxVisitsPerSearch || 3;
  var findings = [];
  var newCandidates = [];

  for (var q = 0; q < allQueries.length; q++) {
    if (totalVisits >= maxTotalVisits) {
      console.log(
        "  [LIMIT] Reached max total visits (" +
          maxTotalVisits +
          "), stopping.",
      );
      break;
    }

    var query = allQueries[q];
    console.log(
      "\n  [Search " +
        (q + 1) +
        "/" +
        allQueries.length +
        "] " +
        query.query +
        "  (" +
        query.reason +
        ")",
    );

    var searchResults = ddgSearch(
      query.query,
      CONFIG.webExplore.resultsPerSearch,
    );
    console.log("    → " + searchResults.length + " results");

    if (!searchResults.length) {
      safeSleep(CONFIG.webExplore.searchDelay || 2000);
      continue;
    }

    // Prioritise official domains: look for the service's own domain first
    var serviceLower = String(query.service || query.appName || "")
      .toLowerCase()
      .replace(/\s+/g, "");
    searchResults.sort(function (a, b) {
      var aDomain = String(a.domain || "").toLowerCase();
      var bDomain = String(b.domain || "").toLowerCase();
      var aOfficial = aDomain.indexOf(serviceLower) !== -1 ? 0 : 1;
      var bOfficial = bDomain.indexOf(serviceLower) !== -1 ? 0 : 1;
      return aOfficial - bOfficial;
    });

    var visitCount = 0;
    for (
      var r = 0;
      r < searchResults.length &&
      visitCount < maxVisitsPerSearch &&
      totalVisits < maxTotalVisits;
      r++
    ) {
      var result = searchResults[r];
      console.log(
        "    [Visit] " + result.domain + " — " + result.title.substring(0, 50),
      );

      var pageText = visitAndExtractContent(
        result.url,
        CONFIG.webExplore.maxPageChars,
      );
      totalVisits++;
      visitCount++;

      if (pageText.length < 80) {
        console.log("      → too short (" + pageText.length + " chars), skip");
        continue;
      }

      auditLog(
        "web_explore_visit",
        result.url + " (" + pageText.length + " chars)",
      );

      var finding = null;
      if (query.type === "pricing") {
        finding = llmExtractPricingFromPage(
          pageText,
          query.service,
          "Current plan: " +
            (query.plan || "unknown") +
            ". Source: " +
            (query.source_url || ""),
        );
        if (finding && finding.plans && finding.plans.length > 0) {
          finding._queryType = "pricing";
          finding._service = query.service;
          finding._sourceUrl = result.url;
          findings.push(finding);
          console.log("      → found " + finding.plans.length + " plan(s)");
          break; // got what we need for this service
        }
      } else if (query.type === "app_subscription_check") {
        finding = llmCheckAppSubscription(pageText, query.appName);
        if (finding) {
          finding._queryType = "app_check";
          finding._appName = query.appName;
          finding._sourceUrl = result.url;
          findings.push(finding);

          if (finding.is_subscription && !finding.is_free) {
            console.log(
              "      → " +
                query.appName +
                " is subscription-based: " +
                (finding.price_monthly || "price unknown"),
            );
            // Validate that price_monthly looks like a real price (has digit), not "Unknown" / "unknown"
            var rawPrice = finding.price_monthly || "";
            var hasRealPrice = /\d/.test(rawPrice);
            newCandidates.push({
              service: query.appName,
              plan: finding.plan_name || "unknown",
              price: hasRealPrice ? rawPrice : "",
              currency: "USD",
              billing_period: "monthly",
              next_billing_date: "",
              status: "active",
              notes: "Detected via web exploration — verify manually" + (hasRealPrice ? "" : " (price unknown)"),
              source_url: result.url,
              source_type: "web_exploration",
              source_title: "DuckDuckGo → " + result.domain,
            });
          } else {
            console.log(
              "      → " +
                query.appName +
                ": " +
                (finding.is_free
                  ? "free"
                  : finding.is_subscription
                    ? "subscription"
                    : "one-time/free"),
            );
          }
          break; // got answer for this app
        }
      }
    }

    safeSleep(CONFIG.webExplore.searchDelay || 2000);
  }

  // --- Merge pricing findings back into subscriptions ---
  for (var f = 0; f < findings.length; f++) {
    var fd = findings[f];
    if (fd._queryType !== "pricing" || !fd.plans || !fd.plans.length) continue;

    var svcName = String(fd._service || "").toLowerCase();
    for (var s = 0; s < subscriptions.length; s++) {
      if (String(subscriptions[s].service || "").toLowerCase() !== svcName)
        continue;

      var sub = subscriptions[s];
      if (sub.price_jpy > 0 && sub.confidence >= 0.8) continue; // already has good data

      // Find the plan that best matches or the most common paid plan
      var bestPlan = findBestMatchingPlan(fd.plans, sub.plan);
      if (bestPlan) {
        var priceInfo = parsePrice(
          bestPlan.price_monthly || bestPlan.price_yearly || "",
        );
        if (priceInfo.amount > 0) {
          var currency = priceInfo.currency || fd.currency || "USD";
          var jpyPrice = convertToJpy(priceInfo.amount, currency);
          var period =
            bestPlan.price_yearly && !bestPlan.price_monthly
              ? "yearly"
              : "monthly";
          var monthlyJpy = estimateMonthlyJpy(jpyPrice, period);

          if (monthlyJpy > sub.monthly_jpy) {
            debugLog(
              "Web-verified price for " +
                sub.service +
                ": " +
                formatJpy(sub.monthly_jpy) +
                " → " +
                formatJpy(monthlyJpy) +
                "/月",
            );
            sub.monthly_jpy = monthlyJpy;
            sub.price_jpy = jpyPrice;
            sub.price_original = priceInfo.amount;
            sub.currency_original = currency;
            sub.billing_period = period;
            sub.plan =
              sub.plan && sub.plan !== "unknown" ? sub.plan : bestPlan.name;
            sub.confidence = Math.min(1.0, sub.confidence + 0.15);
            sub.notes =
              (sub.notes ? sub.notes + " | " : "") +
              "Web-verified: " +
              (bestPlan.price_monthly || bestPlan.price_yearly) +
              " (source: " +
              fd._sourceUrl +
              ")";
            sub.web_verified = true;
          }
        }
      }
    }
  }

  console.log(
    "\n[Step 3.5] Web exploration complete: " +
      totalVisits +
      " pages visited, " +
      findings.length +
      " findings, " +
      newCandidates.length +
      " new candidate(s)",
  );

  auditLog(
    "web_exploration_complete",
    totalVisits +
      " visits, " +
      findings.length +
      " findings, " +
      newCandidates.length +
      " new candidates",
  );

  return {
    subscriptions: subscriptions,
    newCandidates: newCandidates,
    findings: findings,
  };
}

/**
 * Find the plan from a pricing page that best matches the user's current plan.
 */
function findBestMatchingPlan(plans, currentPlanName) {
  if (!plans || !plans.length) return null;
  if (
    !currentPlanName ||
    currentPlanName === "unknown" ||
    currentPlanName === "Unknown"
  ) {
    // Return the first non-free paid plan
    for (var i = 0; i < plans.length; i++) {
      if (!plans[i].is_free) return plans[i];
    }
    return plans[0];
  }

  // Try to match by name
  var cpLower = String(currentPlanName).toLowerCase();
  for (var j = 0; j < plans.length; j++) {
    var pName = String(plans[j].name || "").toLowerCase();
    if (pName.indexOf(cpLower) !== -1 || cpLower.indexOf(pName) !== -1) {
      return plans[j];
    }
  }

  // Fallback: return the first non-free plan
  for (var k = 0; k < plans.length; k++) {
    if (!plans[k].is_free) return plans[k];
  }
  return plans[0];
}

// ============================================================================
// 4. Join & Analyse — Apps × Subscriptions
// ============================================================================

function joinAndAnalyse(apps, subscriptions) {
  auditLog(
    "start_analysis",
    "Joining " +
      apps.length +
      " apps with " +
      subscriptions.length +
      " subscriptions",
  );

  // 4a. Map subscriptions → installed apps
  var enriched = mapSubscriptionsToApps(subscriptions, apps);

  // 4b. Detect category overlaps
  var overlaps = detectCategoryOverlaps(enriched);

  // 4c. Identify unused subscriptions (no matching app or dormant usage)
  var unused = identifyUnused(enriched, apps);

  // 4d. Identify downgrade opportunities via LLM
  var downgrades = identifyDowngradeOpportunities(enriched);

  return {
    subscriptions: enriched,
    overlaps: overlaps,
    unused: unused,
    downgrades: downgrades,
    totalMonthlyJpy: sumMonthly(enriched),
  };
}

function mapSubscriptionsToApps(subscriptions, apps) {
  var appNames = [];
  for (var i = 0; i < apps.length; i++) {
    appNames.push(String(apps[i].name || "").toLowerCase());
  }

  for (var j = 0; j < subscriptions.length; j++) {
    var sub = subscriptions[j];
    var svcLower = String(sub.service || "").toLowerCase();
    sub.matched_app = "";
    sub.app_usage = "no_match";

    for (var k = 0; k < apps.length; k++) {
      var appLower = appNames[k];
      if (!appLower) continue;

      // Direct name match or substring match
      if (
        svcLower.indexOf(appLower) !== -1 ||
        appLower.indexOf(svcLower) !== -1
      ) {
        sub.matched_app = apps[k].name;
        sub.app_usage = apps[k].usageBucket || "unknown";
        break;
      }
      // Bundle ID match
      var bid = String(apps[k].bundleId || "").toLowerCase();
      if (bid && svcLower && bid.indexOf(svcLower) !== -1) {
        sub.matched_app = apps[k].name;
        sub.app_usage = apps[k].usageBucket || "unknown";
        break;
      }
    }

    // Assign categories
    sub.categories = assignCategories(
      svcLower + " " + String(sub.plan || "").toLowerCase(),
    );
  }
  return subscriptions;
}

function assignCategories(text) {
  var result = [];
  var groups = CONFIG.categoryGroups;
  var keys = Object.keys(groups);
  for (var i = 0; i < keys.length; i++) {
    var keywords = groups[keys[i]];
    for (var j = 0; j < keywords.length; j++) {
      if (text.indexOf(keywords[j]) !== -1) {
        result.push(keys[i]);
        break;
      }
    }
  }
  return result;
}

function detectCategoryOverlaps(subscriptions) {
  var categoryMap = {}; // category → [subscription indices]
  for (var i = 0; i < subscriptions.length; i++) {
    var cats = subscriptions[i].categories || [];
    for (var j = 0; j < cats.length; j++) {
      if (!categoryMap[cats[j]]) categoryMap[cats[j]] = [];
      categoryMap[cats[j]].push(i);
    }
  }

  var overlaps = [];
  var keys = Object.keys(categoryMap);
  for (var k = 0; k < keys.length; k++) {
    var indices = categoryMap[keys[k]];
    if (indices.length >= 2) {
      var services = [];
      var totalMonthly = 0;
      for (var m = 0; m < indices.length; m++) {
        var sub = subscriptions[indices[m]];
        if (sub.status !== "inactive") {
          services.push(
            sub.service + " (" + formatJpy(sub.monthly_jpy) + "/月)",
          );
          totalMonthly += sub.monthly_jpy || 0;
        }
      }
      if (services.length >= 2) {
        overlaps.push({
          category: keys[k],
          services: services,
          total_monthly_jpy: totalMonthly,
          potential_savings: Math.round(totalMonthly * 0.4), // conservative estimate
        });
      }
    }
  }
  return overlaps;
}

function identifyUnused(subscriptions, apps) {
  var unused = [];
  for (var i = 0; i < subscriptions.length; i++) {
    var sub = subscriptions[i];
    if (sub.status === "inactive") continue;

    var isUnused = false;
    var reason = "";

    if (!sub.matched_app && sub.monthly_jpy > 0) {
      isUnused = true;
      reason = "No matching installed application found";
    } else if (sub.app_usage === "dormant" || sub.app_usage === "rare") {
      isUnused = true;
      reason =
        "Matched app '" + sub.matched_app + "' usage is " + sub.app_usage;
    } else if (sub.app_usage === "never_or_unknown" && sub.monthly_jpy > 0) {
      isUnused = true;
      reason = "Matched app has no recorded usage";
    }

    if (isUnused) {
      unused.push({
        service: sub.service,
        plan: sub.plan,
        monthly_jpy: sub.monthly_jpy,
        reason: reason,
        matched_app: sub.matched_app,
        app_usage: sub.app_usage,
      });
    }
  }
  return unused;
}

function identifyDowngradeOpportunities(subscriptions) {
  // Filter active subscriptions w/ meaningful price
  var candidates = [];
  for (var i = 0; i < subscriptions.length; i++) {
    var s = subscriptions[i];
    if (s.status !== "inactive" && s.monthly_jpy > 0) {
      candidates.push(s);
    }
  }
  if (!candidates.length) return [];

  var systemPrompt =
    "You are a subscription optimisation advisor. Given a list of active subscriptions, " +
    "identify downgrade opportunities. For each, suggest a cheaper plan or alternative. " +
    "Consider: annual vs monthly savings, lower tiers, free alternatives. " +
    "Return ONLY a JSON array. Each item: " +
    "{service, current_plan, suggested_action, suggested_plan, estimated_monthly_savings_jpy, rationale, risk}. " +
    "Only include items where a real saving is plausible. Be conservative.";

  var userPrompt =
    "Active subscriptions:\n" + JSON.stringify(candidates, null, 2);

  var raw = llmChat(systemPrompt, userPrompt);
  var json = extractJsonArray(raw);
  var parsed = safeJsonParse(json);
  return Array.isArray(parsed) ? parsed : [];
}

function sumMonthly(subscriptions) {
  var total = 0;
  for (var i = 0; i < subscriptions.length; i++) {
    if (subscriptions[i].status !== "inactive") {
      total += subscriptions[i].monthly_jpy || 0;
    }
  }
  return total;
}

function formatJpy(amount) {
  if (!amount) return "¥0";
  return "¥" + String(Math.round(amount)).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

// ============================================================================
// 5. Generate 3-Category Recommendations
// ============================================================================

function generateRecommendations(analysis) {
  auditLog("start_recommendations", "Generating 3-category recommendations");

  var recs = {
    downgrades: [],
    duplicates: [],
    unused: [],
  };

  // A. Downgrade opportunities
  var llmDowngrades = analysis.downgrades || [];
  for (var i = 0; i < llmDowngrades.length; i++) {
    var d = llmDowngrades[i];
    recs.downgrades.push({
      service: d.service || "",
      current_plan: d.current_plan || "",
      suggested_action: d.suggested_action || "Review plan",
      suggested_plan: d.suggested_plan || "",
      estimated_monthly_savings_jpy: d.estimated_monthly_savings_jpy || 0,
      rationale: d.rationale || "",
      risk: d.risk || "low",
      evidence: "LLM analysis of subscription data",
      user_must_confirm: true,
    });
  }

  // B. Duplicate/overlap reduction
  var overlaps = analysis.overlaps || [];
  for (var j = 0; j < overlaps.length; j++) {
    var o = overlaps[j];
    recs.duplicates.push({
      category: o.category,
      services: o.services,
      total_monthly_jpy: o.total_monthly_jpy,
      potential_savings_jpy: o.potential_savings,
      rationale:
        "Multiple services in '" +
        o.category +
        "' category. Consider consolidating.",
      risk: "medium",
      evidence: "Category overlap detection",
      user_must_confirm: true,
    });
  }

  // C. Likely-unused subscriptions
  var unusedList = analysis.unused || [];
  for (var k = 0; k < unusedList.length; k++) {
    var u = unusedList[k];
    recs.unused.push({
      service: u.service,
      plan: u.plan,
      monthly_jpy: u.monthly_jpy,
      reason: u.reason,
      matched_app: u.matched_app,
      app_usage: u.app_usage,
      risk: "low",
      evidence: "App inventory vs subscription cross-reference",
      user_must_confirm: true,
    });
  }

  var totalPotentialSavings = 0;
  for (var a = 0; a < recs.downgrades.length; a++)
    totalPotentialSavings +=
      recs.downgrades[a].estimated_monthly_savings_jpy || 0;
  for (var b = 0; b < recs.duplicates.length; b++)
    totalPotentialSavings += recs.duplicates[b].potential_savings_jpy || 0;
  for (var c = 0; c < recs.unused.length; c++)
    totalPotentialSavings += recs.unused[c].monthly_jpy || 0;

  recs.total_potential_monthly_savings_jpy = totalPotentialSavings;

  console.log(
    "[Step 5] Recommendations: " +
      recs.downgrades.length +
      " downgrades, " +
      recs.duplicates.length +
      " overlaps, " +
      recs.unused.length +
      " unused. " +
      "Potential savings: " +
      formatJpy(totalPotentialSavings) +
      "/月",
  );

  return recs;
}

// ============================================================================
// 6. Build Actions Queue
// ============================================================================

function buildActionsQueue(analysis, recs) {
  var actions = [];

  // From downgrade recommendations
  for (var i = 0; i < recs.downgrades.length; i++) {
    var d = recs.downgrades[i];
    var sub = findSubscriptionByService(analysis.subscriptions, d.service);
    actions.push({
      type: "navigate_steps",
      title:
        "Review " +
        d.service +
        " plan — " +
        (d.suggested_action || "consider downgrade"),
      url: sub ? sub.source_url : "",
      steps: [
        "Open " + d.service + " billing/plan page",
        "Review current plan: " + (d.current_plan || "unknown"),
        d.suggested_action || "Consider downgrading",
        d.suggested_plan
          ? "Target plan: " + d.suggested_plan
          : "Review available plans",
        "Confirm any changes only after careful review",
      ],
      prerequisites: [
        "Logged into " + d.service + " account",
        "Review rationale: " + (d.rationale || ""),
      ],
      estimated_savings:
        formatJpy(d.estimated_monthly_savings_jpy || 0) + "/月",
      risk_level: d.risk || "low",
    });
  }

  // From overlap recommendations
  for (var j = 0; j < recs.duplicates.length; j++) {
    var o = recs.duplicates[j];
    actions.push({
      type: "navigate_steps",
      title: "Review overlapping " + o.category + " subscriptions",
      url: "",
      steps: [
        "Compare services: " + (o.services || []).join(", "),
        "Determine which service best fits your needs",
        "Consider cancelling redundant service(s)",
        "Before cancelling, verify no unique features would be lost",
      ],
      prerequisites: ["Access to all listed service accounts"],
      estimated_savings: formatJpy(o.potential_savings_jpy || 0) + "/月",
      risk_level: "medium",
    });
  }

  // From unused recommendations
  for (var k = 0; k < recs.unused.length; k++) {
    var u = recs.unused[k];
    var uSub = findSubscriptionByService(analysis.subscriptions, u.service);
    actions.push({
      type: "navigate_steps",
      title: "Review unused subscription: " + u.service,
      url: uSub ? uSub.source_url : "",
      steps: [
        "Open " + u.service + " account/billing page",
        "Confirm you are no longer using this service",
        "Reason flagged: " + (u.reason || "low/no usage detected"),
        "If confirmed unused, navigate to cancellation page",
        "Follow the service's cancellation process",
      ],
      prerequisites: [
        "Logged into " + u.service + " account",
        "Verified that you no longer need this service",
      ],
      estimated_savings: formatJpy(u.monthly_jpy || 0) + "/月",
      risk_level: "low",
    });
  }

  // Calendar reminder for next billing review
  actions.push({
    type: "calendar_reminder",
    title: "Schedule next subscription review",
    url: "",
    steps: [
      "Set a calendar reminder for 30 days from now",
      "Review subscription spending again after implementing changes",
      "Verify savings have taken effect",
    ],
    prerequisites: [],
    estimated_savings: "N/A",
    risk_level: "low",
  });

  return actions;
}

function findSubscriptionByService(subscriptions, serviceName) {
  if (!serviceName) return null;
  var lower = serviceName.toLowerCase();
  for (var i = 0; i < subscriptions.length; i++) {
    if (
      String(subscriptions[i].service || "")
        .toLowerCase()
        .indexOf(lower) !== -1
    ) {
      return subscriptions[i];
    }
  }
  return null;
}

// ============================================================================
// 6.5. LLM-Based Comprehensive Report Generation
// ============================================================================

/**
 * Generate a single report section via LLM.
 * Returns the generated text, or a fallback message on failure.
 */
function generateReportSection(sectionTitle, systemPrompt, userPrompt) {
  debugLog("Generating report section: " + sectionTitle);
  var text = llmChat(systemPrompt, userPrompt);
  if (!text || text.length < 30) {
    debugLog("Report section too short or empty: " + sectionTitle);
    return "(この節の生成に失敗しました。データは他の artifact ファイルを参照してください。)";
  }
  return text;
}

/**
 * Build a JSON-like summary string of subscriptions for LLM context.
 */
function buildSubsSummaryForLlm(subs) {
  var parts = [];
  for (var i = 0; i < subs.length; i++) {
    var s = subs[i];
    parts.push(
      (i + 1) + ". " + (s.service || "?") +
        " — plan: " + (s.plan || "?") +
        ", price: " + (s.raw_price || formatJpy(s.monthly_jpy) + "/月") +
        ", billing: " + (s.billing_period || "?") +
        ", status: " + (s.status || "?") +
        (s.notes ? ", notes: " + s.notes : "")
    );
  }
  return parts.join("\n");
}

/**
 * Build a summary of local apps for LLM context.
 */
function buildAppsSummaryForLlm(apps) {
  var parts = [];
  for (var i = 0; i < apps.length; i++) {
    var a = apps[i];
    parts.push(
      (i + 1) + ". " + (a.name || "?") +
        (a.lastUsed ? " (last used: " + a.lastUsed + ")" : "") +
        (a.source ? " [" + a.source + "]" : "")
    );
  }
  return parts.join("\n");
}

/**
 * Build a summary of recommendations for LLM context.
 */
function buildRecsSummaryForLlm(recs) {
  var parts = [];
  if (recs.downgrades && recs.downgrades.length) {
    parts.push("■ ダウングレード候補:");
    for (var i = 0; i < recs.downgrades.length; i++) {
      var d = recs.downgrades[i];
      parts.push(
        "  - " + d.service + ": " + d.current_plan + " → " + (d.suggested_plan || d.suggested_action) +
          " (savings: " + formatJpy(d.estimated_monthly_savings_jpy || 0) + "/月)"
      );
    }
  }
  if (recs.duplicates && recs.duplicates.length) {
    parts.push("■ 重複/オーバーラップ:");
    for (var j = 0; j < recs.duplicates.length; j++) {
      var o = recs.duplicates[j];
      parts.push(
        "  - " + o.category + ": " + (o.services || []).join(", ") +
          " (savings: " + formatJpy(o.potential_savings_jpy || 0) + "/月)"
      );
    }
  }
  if (recs.unused && recs.unused.length) {
    parts.push("■ 未使用の可能性:");
    for (var k = 0; k < recs.unused.length; k++) {
      var u = recs.unused[k];
      parts.push(
        "  - " + u.service + " (" + formatJpy(u.monthly_jpy) + "/月) — " + (u.reason || "低使用率")
      );
    }
  }
  return parts.join("\n");
}

/**
 * Main report builder: generates a comprehensive Markdown report using LLM
 * for each section, following the DDG Deep Research report pattern.
 */
function buildComprehensiveReport(analysis, recs, localApps, auditObj) {
  var subs = analysis.subscriptions || [];
  var today = new Date().toISOString().split("T")[0];

  // Pre-build context strings for LLM
  var subsSummary = buildSubsSummaryForLlm(subs);
  var appsSummary = buildAppsSummaryForLlm(localApps);
  var recsSummary = buildRecsSummaryForLlm(recs);

  var sysBase =
    "あなたはサブスクリプション最適化の専門アナリストです。日本語でレポートを書いてください。" +
    "マークダウン形式で出力してください。見出し(##, ###)は使わないでください（親セクションが管理します）。" +
    "具体的な数値・サービス名を必ず含めてください。";

  var report = "";

  // ─── Title ───
  report += "# AI コーディングツール サブスクリプション最適化レポート\n\n";
  report += "**Subscription Optimization Deep Research** | Generated: " + today + "\n\n";
  report += "---\n\n";

  // ─── Quality Metrics Box ───
  report += "> **データ収集サマリー**\n>\n";
  report += "> - 検出サブスクリプション数: " + subs.length + "\n>\n";
  report += "> - ローカルAIツール数: " + localApps.length + "\n>\n";
  report += "> - 月額合計 (推定): " + formatJpy(analysis.totalMonthlyJpy) + "\n>\n";
  report += "> - 潜在的月間節約額: " + formatJpy(recs.total_potential_monthly_savings_jpy || 0) + "\n>\n";
  report += "> - 重複カテゴリ: " + (analysis.overlaps ? analysis.overlaps.length : 0) + "\n>\n";
  report += "> - 未使用の可能性: " + (analysis.unused ? analysis.unused.length : 0) + "\n>\n";
  report += "> - データソース: ブラウザ課金ページ, ローカルアプリ, DuckDuckGo Web探索\n>\n";
  report += ">\n";
  report += "> ⚠ このレポートは読み取り専用の分析結果です。サブスクリプションの変更は一切行いません。\n\n";
  report += "---\n\n";

  // ─── Table of Contents ───
  report += "## 目次\n\n";
  report += "1. [エグゼクティブサマリー](#1-エグゼクティブサマリー)\n";
  report += "2. [調査方法](#2-調査方法)\n";
  report += "3. [サブスクリプション詳細分析](#3-サブスクリプション詳細分析)\n";
  report += "4. [ローカルアプリ相関分析](#4-ローカルアプリ相関分析)\n";
  report += "5. [コスト最適化戦略](#5-コスト最適化戦略)\n";
  report += "6. [ツール比較・競合分析](#6-ツール比較競合分析)\n";
  report += "7. [推奨アクションプラン](#7-推奨アクションプラン)\n";
  report += "8. [結論](#8-結論)\n";
  report += "9. [データ付録](#9-データ付録)\n\n";
  report += "---\n\n";

  // ─── 1. Executive Summary (LLM) ───
  console.log("  → エグゼクティブサマリー...");
  var execSummary = generateReportSection(
    "Executive Summary",
    sysBase,
    "以下のデータに基づき、AIコーディングツールのサブスクリプション最適化調査のエグゼクティブサマリーを200〜400語で作成してください。" +
      "現在の月額総コスト、検出した主要サブスクリプション、主要な最適化機会、推定節約額を含めてください。\n\n" +
      "■ サブスクリプション一覧:\n" + subsSummary + "\n\n" +
      "■ 月額合計: " + formatJpy(analysis.totalMonthlyJpy) + "\n" +
      "■ 推定節約可能額: " + formatJpy(recs.total_potential_monthly_savings_jpy || 0) + "/月\n\n" +
      "■ 推奨事項概要:\n" + recsSummary
  );
  report += "## 1. エグゼクティブサマリー\n\n";
  report += execSummary + "\n\n";
  report += "---\n\n";

  // ─── 2. Methodology ───
  console.log("  → 調査方法...");
  report += "## 2. 調査方法\n\n";
  report += "### 2.1 データ収集プロセス\n\n";
  report += "本調査では、以下の多層的アプローチでデータを収集しました：\n\n";
  report += "1. **ローカルアプリ走査**: `/Applications`、`~/Applications`、Homebrew Cask から AI コーディングツールを検出\n";
  report += "2. **使用シグナル取得**: `mdls` コマンドで各アプリの最終使用日を取得\n";
  report += "3. **AI ツールフィルタリング**: 既知のキーワードリスト + DuckDuckGo 検索による AI ツール判定\n";
  report += "4. **課金ページ自動収集**: " + (auditObj.read_sources ? auditObj.read_sources.length : 4) + " のソースからサブスクリプション情報を自動抽出\n";
  report += "5. **LLM 抽出**: 各課金ページのテキストから構造化データを LLM で抽出\n";
  report += "6. **Web 探索**: DuckDuckGo で未検出サブスクリプションの料金情報を補完\n\n";

  report += "### 2.2 分析パイプライン\n\n";
  report += "```\n";
  report += "ローカルアプリ走査 → AI ツールフィルタ → 課金ページ収集 → LLM 抽出\n";
  report += "  → 正規化・重複除去 → AI ツールフィルタ(サブスク) → Web 探索\n";
  report += "  → アプリ-サブスク結合 → 最適化分析 → レポート生成\n";
  report += "```\n\n";

  report += "### 2.3 統計\n\n";
  report += "| 項目 | 数値 |\n";
  report += "|------|------|\n";
  report += "| ローカルアプリ検出数 | " + localApps.length + " |\n";
  report += "| サブスクリプション検出数 | " + subs.length + " |\n";
  report += "| 月額合計 (推定) | " + formatJpy(analysis.totalMonthlyJpy) + " |\n";
  report += "| 重複カテゴリ | " + (analysis.overlaps ? analysis.overlaps.length : 0) + " |\n";
  report += "| 未使用の可能性 | " + (analysis.unused ? analysis.unused.length : 0) + " |\n\n";
  report += "---\n\n";

  // ─── 3. Per-Subscription Analysis (LLM) ───
  console.log("  → サブスクリプション詳細分析...");
  report += "## 3. サブスクリプション詳細分析\n\n";

  if (subs.length === 0) {
    report += "検出されたサブスクリプションはありません。\n\n";
  } else {
    // Generate a combined deep-dive analysis for all subscriptions
    var subsAnalysis = generateReportSection(
      "Subscription Analysis",
      sysBase + " 各サブスクリプションについて、サービスの概要・プラン詳細・コスト評価・使用状況の推定を含めてください。",
      "以下の各AIコーディングツールサブスクリプションについて、それぞれ100〜200語で詳細分析を行ってください。" +
        "各サービスについて: (1)サービス概要 (2)検出プラン・価格 (3)主要機能 (4)コスト対価値評価 を含めてください。" +
        "各サービスの前に「**サービス名**」を太字で付けてください。\n\n" +
        subsSummary
    );
    report += subsAnalysis + "\n\n";
  }
  report += "---\n\n";

  // ─── 4. Local App Correlation (LLM) ───
  console.log("  → ローカルアプリ相関分析...");
  report += "## 4. ローカルアプリ相関分析\n\n";

  var correlationText = generateReportSection(
    "App Correlation",
    sysBase + " ローカルにインストールされたAIコーディングツールと有料サブスクリプションの関連性を分析してください。",
    "以下のローカルAIコーディングツールと検出サブスクリプションの相関を300〜500語で分析してください。" +
      "どのアプリがどのサブスクリプションに対応するか、サブスクなしで使えるアプリ、" +
      "サブスクがあるのにローカルにアプリがないケースを特定してください。\n\n" +
      "■ ローカルアプリ:\n" + appsSummary + "\n\n" +
      "■ サブスクリプション:\n" + subsSummary
  );
  report += correlationText + "\n\n";

  // Correlation table
  report += "### アプリ-サブスクリプション対応表\n\n";
  report += "| サブスクリプション | マッチしたアプリ | アプリ使用状況 |\n";
  report += "|---|---|---|\n";
  for (var ci = 0; ci < subs.length; ci++) {
    var cs = subs[ci];
    report += "| " + safeMd(cs.service) +
      " | " + safeMd(cs.matched_app || "(なし)") +
      " | " + safeMd(cs.app_usage || "N/A") + " |\n";
  }
  report += "\n";
  report += "---\n\n";

  // ─── 5. Cost Optimization Strategy (LLM) ───
  console.log("  → コスト最適化戦略...");
  var costStrategy = generateReportSection(
    "Cost Strategy",
    sysBase + " 具体的な数値を使い、コスト削減の戦略を提案してください。リスクと注意点も含めてください。",
    "以下のデータに基づき、AIコーディングツールのコスト最適化戦略を400〜600語で作成してください。" +
      "現状の支出分析、削減優先順位、具体的な節約額の計算、実行リスクを含めてください。\n\n" +
      "■ 月額合計: " + formatJpy(analysis.totalMonthlyJpy) + "\n" +
      "■ 推定節約可能額: " + formatJpy(recs.total_potential_monthly_savings_jpy || 0) + "/月\n\n" +
      "■ サブスクリプション:\n" + subsSummary + "\n\n" +
      "■ 推奨事項:\n" + recsSummary
  );
  report += "## 5. コスト最適化戦略\n\n";
  report += costStrategy + "\n\n";
  report += "---\n\n";

  // ─── 6. Competitive Comparison (LLM) ───
  console.log("  → ツール比較・競合分析...");
  var compAnalysis = generateReportSection(
    "Competitive Analysis",
    sysBase + " AIコーディングツール市場の知識を活用し、検出されたツールの比較分析を行ってください。",
    "以下のAIコーディングツールについて、機能比較・市場ポジション・価格対価値を300〜500語で分析してください。" +
      "重複する機能、各ツールの独自の強み、統合の可能性を評価してください。\n\n" +
      "■ 検出ツール (サブスクリプション):\n" + subsSummary + "\n\n" +
      "■ ローカルインストール済み:\n" + appsSummary
  );
  report += "## 6. ツール比較・競合分析\n\n";
  report += compAnalysis + "\n\n";
  report += "---\n\n";

  // ─── 7. Action Plan (LLM) ───
  console.log("  → 推奨アクションプラン...");
  var actionPlan = generateReportSection(
    "Action Plan",
    sysBase + " 優先度順に具体的なアクションアイテムを提案してください。各アクションのリスクレベルも示してください。",
    "以下の最適化推奨事項に基づき、優先度付きの具体的アクションプランを300〜500語で作成してください。" +
      "各アクションに (1)概要 (2)期待効果 (3)リスク (4)優先度(高/中/低) を含めてください。\n\n" +
      "■ 推奨事項:\n" + recsSummary + "\n\n" +
      "■ 現在の月額合計: " + formatJpy(analysis.totalMonthlyJpy) + "\n" +
      "■ 推定節約額: " + formatJpy(recs.total_potential_monthly_savings_jpy || 0) + "/月"
  );
  report += "## 7. 推奨アクションプラン\n\n";
  report += actionPlan + "\n\n";
  report += "---\n\n";

  // ─── 8. Conclusions (LLM) ───
  console.log("  → 結論...");
  var conclusions = generateReportSection(
    "Conclusions",
    sysBase,
    "以下のデータに基づき、AIコーディングツールのサブスクリプション最適化調査の結論を200〜300語でまとめてください。" +
      "主要な発見、推奨アクションの要約、今後のレビュー計画を含めてください。\n\n" +
      "■ サブスクリプション数: " + subs.length + "\n" +
      "■ ローカルAIツール数: " + localApps.length + "\n" +
      "■ 月額合計: " + formatJpy(analysis.totalMonthlyJpy) + "\n" +
      "■ 推定節約可能額: " + formatJpy(recs.total_potential_monthly_savings_jpy || 0) + "/月\n" +
      "■ 推奨事項数: ダウングレード " + (recs.downgrades ? recs.downgrades.length : 0) +
        ", 重複 " + (recs.duplicates ? recs.duplicates.length : 0) +
        ", 未使用 " + (recs.unused ? recs.unused.length : 0)
  );
  report += "## 8. 結論\n\n";
  report += conclusions + "\n\n";
  report += "---\n\n";

  // ─── 9. Data Appendix (static) ───
  report += "## 9. データ付録\n\n";
  report += "### 9.1 サブスクリプション一覧\n\n";
  report += "| # | サービス | プラン | 価格 | 月額(¥) | 請求周期 | ステータス | ソース |\n";
  report += "|---|---|---|---|---:|---|---|---|\n";
  for (var di = 0; di < subs.length; di++) {
    var ds = subs[di];
    report += "| " + (di + 1) +
      " | " + safeMd(ds.service) +
      " | " + safeMd(ds.plan) +
      " | " + safeMd(ds.raw_price || "") +
      " | " + safeMd(formatJpy(ds.monthly_jpy)) +
      " | " + safeMd(ds.billing_period) +
      " | " + safeMd(ds.status) +
      " | " + safeMd(ds.source_type || "") + " |\n";
  }
  report += "\n";

  report += "### 9.2 ローカル AI ツール一覧\n\n";
  report += "| # | アプリ名 | 最終使用日 | ソース |\n";
  report += "|---|---|---|---|\n";
  for (var ai = 0; ai < localApps.length; ai++) {
    var la = localApps[ai];
    report += "| " + (ai + 1) +
      " | " + safeMd(la.name) +
      " | " + safeMd(la.lastUsed || "不明") +
      " | " + safeMd(la.source || "") + " |\n";
  }
  report += "\n";

  report += "---\n\n";
  report += "*本レポートは Subscription Optimization Deep Research ワークフローにより自動生成されました。*\n";
  report += "*AI 分析によるコンテンツ抽出・合成を使用しています。*\n";
  report += "*サブスクリプションの変更は一切行っていません。すべての推奨事項は手動での確認・実行が必要です。*\n";
  report += "*Generated: " + new Date().toISOString() + "*\n";

  return report;
}

// ============================================================================
// 7. Write Artifacts
// ============================================================================

function writeArtifacts(outputDir, analysis, recs, actionsQueue, auditObj, comprehensiveReport) {
  ensureDirExists(outputDir);
  auditLog("create_output_dir", outputDir);

  // 7a. subscriptions_inventory.md
  var invMd = buildInventoryMarkdown(analysis);
  writeFile(outputDir + "/subscriptions_inventory.md", invMd);

  // 7b. optimization_plan.md
  var planMd = buildOptimizationPlanMarkdown(analysis, recs);
  writeFile(outputDir + "/optimization_plan.md", planMd);

  // 7c. actions_queue.json
  writeFile(
    outputDir + "/actions_queue.json",
    JSON.stringify(actionsQueue, null, 2),
  );

  // 7d. audit_log.json
  var auditLogObj = {
    workflow_id: "subscription_optimization_deep_research",
    started_at: auditObj.generated_at,
    completed_at: timestamp(),
    pre_execution_audit: auditObj,
    entries: AUDIT_LOG,
  };
  writeFile(
    outputDir + "/audit_log.json",
    JSON.stringify(auditLogObj, null, 2),
  );

  // 7e. analysis_report.md (LLM-generated comprehensive report)
  if (comprehensiveReport) {
    writeFile(outputDir + "/analysis_report.md", comprehensiveReport);
    console.log("  Report size: " + (comprehensiveReport.length / 1000).toFixed(1) + " KB");
  }

  console.log("[Step 7] Artifacts written to " + outputDir);
}

function buildInventoryMarkdown(analysis) {
  var subs = analysis.subscriptions || [];
  var lines = [];

  lines.push("# Subscription Inventory");
  lines.push("");
  lines.push("Generated: " + timestamp());
  lines.push("");
  lines.push(
    "Coverage: " +
      (subs.length > 0
        ? "partial — only detected from open browser tabs and local signals"
        : "no subscriptions detected"),
  );
  lines.push("");
  lines.push("## Summary");
  lines.push("");
  lines.push("- Total subscriptions detected: " + subs.length);

  var activeCount = 0;
  var inactiveCount = 0;
  for (var i = 0; i < subs.length; i++) {
    if (subs[i].status === "inactive") inactiveCount++;
    else activeCount++;
  }
  lines.push("- Active: " + activeCount);
  lines.push("- Inactive/Cancelled: " + inactiveCount);
  lines.push(
    "- Total estimated monthly cost: " + formatJpy(analysis.totalMonthlyJpy),
  );
  lines.push(
    "- Currency conversion: " +
      (EXCHANGE_RATES.JPY ? "live rates" : "approximate fallback rates"),
  );
  lines.push("");
  lines.push("## Subscription Table");
  lines.push("");
  lines.push(
    "| ServiceName | Plan | Price (original) | Price (¥/月) | BillingCycle | NextBillingDate | Status | SourceEvidence | Confidence | Notes |",
  );
  lines.push("|---|---|---:|---:|---|---|---|---|---|---|");

  for (var j = 0; j < subs.length; j++) {
    var s = subs[j];
    lines.push(
      "| " +
        safeMd(s.service) +
        " | " +
        safeMd(s.plan) +
        " | " +
        safeMd(s.raw_price || String(s.price_original || "")) +
        " | " +
        safeMd(formatJpy(s.monthly_jpy)) +
        " | " +
        safeMd(s.billing_period) +
        " | " +
        safeMd(s.next_billing_date) +
        " | " +
        safeMd(s.status) +
        " | " +
        safeMd(s.source_url) +
        " | " +
        safeMd(String(s.confidence || "")) +
        " | " +
        safeMd(s.notes) +
        " |",
    );
  }

  lines.push("");
  lines.push("## Local App Cross-Reference");
  lines.push("");
  lines.push("| Subscription | MatchedApp | AppUsage |");
  lines.push("|---|---|---|");
  for (var k = 0; k < subs.length; k++) {
    var sub = subs[k];
    lines.push(
      "| " +
        safeMd(sub.service) +
        " | " +
        safeMd(sub.matched_app || "(none)") +
        " | " +
        safeMd(sub.app_usage || "N/A") +
        " |",
    );
  }

  return lines.join("\n");
}

function buildOptimizationPlanMarkdown(analysis, recs) {
  var lines = [];

  lines.push("# Subscription Optimization Plan");
  lines.push("");
  lines.push("Generated: " + timestamp());
  lines.push("");
  lines.push(
    "**Total current monthly spending (estimate):** " +
      formatJpy(analysis.totalMonthlyJpy),
  );
  lines.push("");
  lines.push(
    "**Total potential monthly savings (estimate):** " +
      formatJpy(recs.total_potential_monthly_savings_jpy || 0),
  );
  lines.push("");
  lines.push(
    "> **IMPORTANT:** This plan contains recommendations only. No subscriptions have been modified.",
  );
  lines.push("> Review each item carefully before taking any action.");
  lines.push("");

  // A. Downgrade
  lines.push("## A. Downgrade Opportunities");
  lines.push("");
  if (!recs.downgrades.length) {
    lines.push("No downgrade opportunities identified.");
  } else {
    for (var i = 0; i < recs.downgrades.length; i++) {
      var d = recs.downgrades[i];
      lines.push("### " + (i + 1) + ". " + safeMd(d.service));
      lines.push("");
      lines.push("- **Current plan:** " + safeMd(d.current_plan));
      lines.push("- **Suggested action:** " + safeMd(d.suggested_action));
      lines.push("- **Suggested plan:** " + safeMd(d.suggested_plan));
      lines.push(
        "- **Expected monthly savings:** " +
          formatJpy(d.estimated_monthly_savings_jpy || 0),
      );
      lines.push("- **Rationale:** " + safeMd(d.rationale));
      lines.push("- **Risk/Tradeoff:** " + safeMd(d.risk));
      lines.push("- **Evidence:** " + safeMd(d.evidence));
      lines.push("- **⚠ User must confirm** before making changes");
      lines.push("");
    }
  }

  // B. Duplicates/Overlaps
  lines.push("## B. Duplicate/Overlap Reduction");
  lines.push("");
  if (!recs.duplicates.length) {
    lines.push("No duplicate/overlap issues identified.");
  } else {
    for (var j = 0; j < recs.duplicates.length; j++) {
      var o = recs.duplicates[j];
      lines.push("### " + (j + 1) + ". " + safeMd(o.category) + " overlap");
      lines.push("");
      lines.push("- **Services:** " + (o.services || []).join(", "));
      lines.push(
        "- **Combined monthly cost:** " + formatJpy(o.total_monthly_jpy),
      );
      lines.push(
        "- **Potential monthly savings:** " +
          formatJpy(o.potential_savings_jpy || 0),
      );
      lines.push("- **Rationale:** " + safeMd(o.rationale));
      lines.push("- **Risk/Tradeoff:** " + safeMd(o.risk));
      lines.push("- **Evidence:** " + safeMd(o.evidence));
      lines.push("- **⚠ User must confirm** before making changes");
      lines.push("");
    }
  }

  // C. Unused
  lines.push("## C. Likely-Unused Subscriptions");
  lines.push("");
  if (!recs.unused.length) {
    lines.push("No likely-unused subscriptions identified.");
  } else {
    for (var k = 0; k < recs.unused.length; k++) {
      var u = recs.unused[k];
      lines.push("### " + (k + 1) + ". " + safeMd(u.service));
      lines.push("");
      lines.push("- **Plan:** " + safeMd(u.plan));
      lines.push("- **Monthly cost:** " + formatJpy(u.monthly_jpy));
      lines.push("- **Reason flagged:** " + safeMd(u.reason));
      lines.push("- **Matched app:** " + safeMd(u.matched_app || "(none)"));
      lines.push("- **App usage:** " + safeMd(u.app_usage));
      lines.push(
        "- **Risk/Tradeoff:** " +
          safeMd(u.risk) +
          " — verify before cancelling",
      );
      lines.push("- **Evidence:** " + safeMd(u.evidence));
      lines.push(
        "- **⚠ User must confirm** that this service is no longer needed",
      );
      lines.push("");
    }
  }

  lines.push("---");
  lines.push("");
  lines.push("## Review Checklist");
  lines.push("");
  lines.push("- [ ] Verify all subscription prices and billing cycles");
  lines.push("- [ ] Cross-check usage signals — some apps may be used via web");
  lines.push(
    "- [ ] Review overlap categories — some services may complement each other",
  );
  lines.push("- [ ] Confirm savings estimates with official pricing pages");
  lines.push("- [ ] Set a follow-up review date (30 days recommended)");

  return lines.join("\n");
}

// ============================================================================
// Main Workflow
// ============================================================================

function workflow() {
  console.log("=== Subscription Optimization Deep Research ===\n");

  // --- Prerequisites ---
  ensurePrerequisites();
  resolveConfig();

  // --- Step 0: Pre-Execution Audit ---
  console.log("[Step 0] Building pre-execution audit...");
  var auditObj = buildPreExecutionAudit();
  auditLog("pre_execution_audit", "Generated");

  console.log("\n========================================");
  console.log("  PRE-EXECUTION AUDIT");
  console.log("========================================");
  console.log(JSON.stringify(auditObj, null, 2));
  console.log("========================================\n");

  if (auditObj.detected_billing_tabs.length === 0) {
    console.log("[WARN] No billing tabs detected in browser.");
    console.log(
      "  Open billing/subscription pages in Floorp, then re-run this workflow.",
    );
    console.log("  Proceeding with local-only analysis...\n");
  }

  if (AUDIT_ONLY) {
    console.log(
      "[AUDIT_ONLY] Stopping after audit. Set AUDIT_ONLY=false to proceed.\n",
    );
    return { ok: true, audit_only: true, audit: auditObj };
  }

  // --- Step 1: Local App Inventory ---
  console.log("\n[Step 1] Building local app inventory...");
  var localApps = buildLocalAppInventory();

  // --- Step 1.5: Filter local apps to AI coding tools only ---
  console.log(
    "\n[Step 1.5] Filtering local apps — keeping AI coding tools only...",
  );
  var preFilterCount = localApps.length;
  localApps = filterLocalAppsToAiTools(localApps);
  console.log(
    "[Step 1.5] " +
      preFilterCount +
      " apps → " +
      localApps.length +
      " AI coding tools (" +
      (preFilterCount - localApps.length) +
      " excluded)",
  );
  auditLog(
    "ai_filter_apps",
    preFilterCount + " → " + localApps.length + " after AI tool filter",
  );

  // --- Currency setup ---
  console.log("\n[Currency] Fetching exchange rates...");
  fetchExchangeRates();

  // --- Step 2: Subscription Inventory ---
  console.log("\n[Step 2] Building subscription inventory from browser...");
  var rawSubscriptions = buildSubscriptionInventory();

  // --- Step 3: Normalise & deduplicate ---
  console.log("\n[Step 3] Normalising and deduplicating...");
  var normalised = normaliseSubscriptions(rawSubscriptions);
  var deduped = deduplicateSubscriptions(normalised);
  console.log(
    "[Step 3] " +
      deduped.length +
      " unique subscriptions after dedup (from " +
      normalised.length +
      " raw)",
  );
  auditLog(
    "normalise_dedup",
    normalised.length + " → " + deduped.length + " entries",
  );

  // --- Step 3.2: Filter subscriptions to AI coding tools only ---
  console.log(
    "\n[Step 3.2] Filtering subscriptions — keeping AI coding tools only...",
  );
  var preSubFilterCount = deduped.length;
  deduped = filterSubscriptionsToAiTools(deduped);
  console.log(
    "[Step 3.2] " +
      preSubFilterCount +
      " subscriptions → " +
      deduped.length +
      " AI coding tools (" +
      (preSubFilterCount - deduped.length) +
      " excluded)",
  );
  auditLog(
    "ai_filter_subs",
    preSubFilterCount + " → " + deduped.length + " after AI tool filter",
  );

  // --- Step 3.5: Web Exploration (DuckDuckGo search for pricing & plan verification) ---
  console.log(
    "\n[Step 3.5] Web exploration for pricing & plan verification...",
  );
  var webResult = webExploreAndVerify(deduped, localApps);
  deduped = webResult.subscriptions;

  // If web exploration found new subscription candidates, normalise and add them
  if (webResult.newCandidates.length > 0) {
    console.log(
      "[Step 3.5] Adding " +
        webResult.newCandidates.length +
        " newly discovered subscription candidate(s)...",
    );
    var newNorm = normaliseSubscriptions(webResult.newCandidates);
    deduped = deduplicateSubscriptions(deduped.concat(newNorm));
    console.log(
      "[Step 3.5] Total after merge: " + deduped.length + " subscriptions",
    );
  }

  // --- Step 4: Join & Analyse ---
  console.log("\n[Step 4] Joining apps with subscriptions...");
  var analysis = joinAndAnalyse(localApps, deduped);
  console.log(
    "[Step 4] Total monthly: " +
      formatJpy(analysis.totalMonthlyJpy) +
      " | Overlaps: " +
      analysis.overlaps.length +
      " | Unused: " +
      analysis.unused.length,
  );

  // --- Step 5: Recommendations ---
  console.log("\n[Step 5] Generating recommendations...");
  var recs = generateRecommendations(analysis);

  // --- Step 6: Actions Queue ---
  console.log("\n[Step 6] Building actions queue...");
  var actionsQueue = buildActionsQueue(analysis, recs);
  auditLog("build_actions_queue", actionsQueue.length + " actions");

  // --- Step 6.5: Comprehensive Report Generation (LLM) ---
  console.log("\n[Step 6.5] Generating comprehensive analysis report...");
  var comprehensiveReport = buildComprehensiveReport(analysis, recs, localApps, auditObj);
  console.log("[Step 6.5] Report generated: " + (comprehensiveReport.length / 1000).toFixed(1) + " KB");
  auditLog("generate_report", comprehensiveReport.length + " chars");

  // --- Step 7: Write Artifacts ---
  console.log("\n[Step 7] Writing artifacts...");
  writeArtifacts(CONFIG.outputBaseDir, analysis, recs, actionsQueue, auditObj, comprehensiveReport);

  // --- Done ---
  console.log("\n========================================");
  console.log("  COMPLETE — NO CHANGES WERE MADE");
  console.log("========================================");
  console.log("Output: " + CONFIG.outputBaseDir);
  console.log("Files:");
  console.log("  1. subscriptions_inventory.md");
  console.log("  2. optimization_plan.md");
  console.log("  3. actions_queue.json");
  console.log("  4. audit_log.json");
  console.log("  5. analysis_report.md  ← NEW: LLM comprehensive report");
  console.log("========================================\n");

  auditLog("workflow_complete", "All artifacts written successfully");

  return {
    ok: true,
    outputDir: CONFIG.outputBaseDir,
    subscriptions: deduped.length,
    localApps: localApps.length,
    totalMonthlyJpy: analysis.totalMonthlyJpy,
    recommendations: {
      downgrades: recs.downgrades.length,
      duplicates: recs.duplicates.length,
      unused: recs.unused.length,
    },
    totalPotentialSavingsJpy: recs.total_potential_monthly_savings_jpy || 0,
  };
}
