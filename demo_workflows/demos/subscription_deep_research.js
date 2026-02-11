/**
 * Subscription Deep Research (MVP)
 *
 * Flow:
 * 1) Open billing/subscription pages in Floorp and extract visible text
 * 2) Use LLM to extract subscription entries
 * 3) Open pricing pages and extract plan info with LLM
 * 4) Use LLM to generate savings/alternative suggestions
 * 5) Write Markdown report to demo_workflows/demos
 */

var DEBUG = true;

function debugLog(message) {
  if (DEBUG) {
    console.log("[DEBUG] " + message);
  }
}

function workflow() {
  var config = {
    outputPath:
      "/Users/user/dev-source/sapphillon-dev/Floorp-OS-Automator-Backend/demo_workflows/demos/subscription_deep_research.md",
    sources: [
      {
        id: "link",
        label: "Link.com",
        url: "https://app.link.com/subscriptions",
        waitSelector: "body",
      },
      {
        id: "zai",
        label: "Z.ai",
        url: "https://z.ai/manage-apikey/subscription",
        waitSelector: "div[role='tabpanel']",
      },
      {
        id: "github_copilot",
        label: "GitHub Copilot",
        url: "https://github.com/settings/billing",
        waitSelector: "body",
      },
      {
        id: "google_subscriptions",
        label: "Google Subscriptions",
        url: "https://myaccount.google.com/subscriptions",
        waitSelector: "body",
      },
    ],
    pricingUrls: {
      cursor: "https://www.cursor.com/pricing",
      copilot: "https://github.com/features/copilot#pricing",
      openai: "https://openai.com/pricing",
      claude: "https://www.anthropic.com/pricing",
      zai: "https://z.ai/pricing",
    },
    sourceKeywords: {
      link: [
        "subscription",
        "subscriptions",
        "plan",
        "billing",
        "payment",
        "invoice",
        "charged",
        "renew",
        "next",
        "amount",
        "price",
        "usd",
        "jpy",
        "copilot",
        "cursor",
        "openai",
        "claude",
        "anthropic",
        "z.ai",
        "zai",
      ],
      zai: ["billing", "usage", "plan", "price", "usd", "jpy", "token"],
      github_copilot: [
        "copilot",
        "billing",
        "plan",
        "price",
        "monthly",
        "annual",
      ],
      google_subscriptions: [
        "subscription",
        "plan",
        "renew",
        "next",
        "price",
        "monthly",
        "annual",
      ],
    },
    sourceSelectors: {
      link: {
        list: "main.RouteContent > div.ListAndDetailView > div.ListAndDetailView-list",
        detail:
          "main.RouteContent > div.ListAndDetailView > div.ListAndDetailView-detail",
        inactiveToggle: 'button:has-text("すべての非アクティブを表示")',
      },
      zai: {
        tabpanel: "div[role='tabpanel']",
        promoDialog: "div[role='dialog']",
        promoClose: 'button:has-text("Close")',
      },
      github_copilot: {
        planCard: "[data-testid='copilot-plan-card']",
        billingContainer: "[data-hpc]",
      },
      google_subscriptions: {
        main: "div[role='main']",
        listitem: "div[role='main'] [role='listitem']",
      },
    },
    serviceKeywords: {
      cursor: ["cursor", "pro", "hobby", "pricing", "monthly", "annual"],
      copilot: ["copilot", "github", "individual", "business", "pricing"],
      openai: ["openai", "chatgpt", "gpt", "pricing", "api"],
      claude: ["claude", "anthropic", "pricing", "api"],
      zai: ["z.ai", "zai", "pricing", "token"],
    },
    maxPageChars: 14000,
  };

  ensurePrerequisites();

  console.log("Starting subscription deep research...");
  console.log("Output path: " + config.outputPath);

  var subscriptionEntries = [];

  for (var i = 0; i < config.sources.length; i++) {
    var src = config.sources[i];
    console.log("\n[Collect] " + src.label + " -> " + src.url);

    var tabId = null;
    try {
      tabId = floorp.createTab(src.url, false);
      if (src.waitSelector) {
        floorp.tabWaitForElement(tabId, src.waitSelector, 15000);
      }
      floorp.tabWaitForNetworkIdle(tabId);
      safeSleep(1500);

      var pageText = collectSourceText(tabId, src.id, config);
      var extracted = llmExtractSubscriptions(pageText, src.id);
      if (extracted && extracted.length) {
        subscriptionEntries = subscriptionEntries.concat(extracted);
      }
    } catch (e) {
      console.log("  [WARN] Failed to collect from " + src.label + ": " + e);
    } finally {
      closeTabAndDestroy(tabId);
    }
  }

  if (!subscriptionEntries.length) {
    console.log("No subscriptions extracted. Aborting report generation.");
    return {
      ok: false,
      message: "No subscriptions extracted",
      outputPath: config.outputPath,
    };
  }

  var normalized = normalizeSubscriptions(subscriptionEntries);
  var pricingCatalog = buildPricingCatalog(normalized, config.pricingUrls);

  var recommendations = llmRecommend(normalized, pricingCatalog);

  var markdown = buildMarkdownReport(
    normalized,
    pricingCatalog,
    recommendations,
  );
  writeFileByExec(config.outputPath, markdown);

  console.log("Report written: " + config.outputPath);

  return {
    ok: true,
    outputPath: config.outputPath,
    subscriptions: normalized.length,
    pricing: pricingCatalog.length,
    recommendations: recommendations.length || 0,
  };
}

function ensurePrerequisites() {
  if (typeof floorp === "undefined") {
    throw new Error("Floorp plugin is required");
  }
  if (typeof llm_chat === "undefined" || !llm_chat.chat) {
    throw new Error("LLM plugin (llm_chat.chat) is required");
  }
  if (
    !app ||
    !app.sapphillon ||
    !app.sapphillon.core ||
    !app.sapphillon.core.exec ||
    typeof app.sapphillon.core.exec.exec !== "function"
  ) {
    throw new Error("Exec plugin is required for file output");
  }
}

function safeSleep(ms) {
  if (typeof sleep === "function") {
    sleep(ms);
  }
}

function getPageText(tabId, maxChars) {
  var text = "";
  var selectors = ["main", "[role='main']", "article", "body"];
  for (var i = 0; i < selectors.length; i++) {
    try {
      var json = floorp.tabElementText(tabId, selectors[i]);
      var obj = safeJsonParse(json);
      var part = obj && obj.text ? obj.text : String(json || "");
      if (part && part.length > text.length) {
        text = part;
      }
    } catch (e) {}
  }

  if (text.length > maxChars) {
    return text.slice(0, maxChars);
  }
  return text;
}

function collectFocusedText(tabId, sourceId, maxChars, sourceKeywords) {
  var text = getPageText(tabId, maxChars * 2);
  var keywords = (sourceKeywords && sourceKeywords[sourceId]) || [];
  return reduceTextForLlm(text, keywords, maxChars);
}

function collectSourceText(tabId, sourceId, config) {
  var selectors =
    (config.sourceSelectors && config.sourceSelectors[sourceId]) || {};
  var maxChars = config.maxPageChars;
  var fallback = collectFocusedText(
    tabId,
    sourceId,
    maxChars,
    config.sourceKeywords,
  );

  debugLog("collectSourceText sourceId=" + sourceId);
  debugLog("selectors=" + JSON.stringify(selectors));

  if (sourceId === "link") {
    if (selectors.inactiveToggle) {
      try {
        debugLog("click inactiveToggle=" + selectors.inactiveToggle);
        floorp.tabClick(tabId, selectors.inactiveToggle);
        safeSleep(800);
      } catch (e) {}
    }
    var listText = readSelectorText(tabId, selectors.list, "link.list");
    var detailText = readSelectorText(tabId, selectors.detail, "link.detail");
    var combined = [listText, detailText].filter(Boolean).join("\n");
    debugLog("link combined length=" + combined.length);
    return combined ? combined.slice(0, maxChars) : fallback;
  }

  if (sourceId === "zai") {
    if (selectors.promoClose) {
      try {
        debugLog("click promoClose=" + selectors.promoClose);
        floorp.tabClick(tabId, selectors.promoClose);
        safeSleep(500);
      } catch (e) {}
    }
    var panelText = readSelectorText(tabId, selectors.tabpanel, "zai.tabpanel");
    debugLog("zai tabpanel length=" + panelText.length);
    return panelText ? panelText.slice(0, maxChars) : fallback;
  }

  if (sourceId === "github_copilot") {
    var planText = readSelectorText(
      tabId,
      selectors.planCard,
      "github_copilot.planCard",
    );
    if (planText) {
      debugLog("github_copilot planCard length=" + planText.length);
      return planText.slice(0, maxChars);
    }
    var billingText = readSelectorText(
      tabId,
      selectors.billingContainer,
      "github_copilot.billingContainer",
    );
    debugLog("github_copilot billing length=" + billingText.length);
    return billingText ? billingText.slice(0, maxChars) : fallback;
  }

  if (sourceId === "google_subscriptions") {
    var mainText = readSelectorText(
      tabId,
      selectors.main,
      "google_subscriptions.main",
    );
    debugLog("google_subscriptions main length=" + mainText.length);
    return mainText ? mainText.slice(0, maxChars) : fallback;
  }

  return fallback;
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
function closeTabAndDestroy(tabId) {
  if (!tabId) return;
  debugLog("closeTab tabId=" + tabId);
  floorp.closeTab(tabId);
}

function collectPricingText(tabId, serviceKey, maxChars, serviceKeywords) {
  var text = getPageText(tabId, maxChars * 2);
  var keywords = (serviceKeywords && serviceKeywords[serviceKey]) || [];
  return reduceTextForLlm(text, keywords, maxChars);
}

function reduceTextForLlm(text, keywords, maxChars) {
  if (!text) return "";
  var lines = text.split(/\n+/);
  var matched = filterLinesWithContext(lines, keywords, 1);
  var joined = matched.join("\n");
  if (joined.length > maxChars) {
    return joined.slice(0, maxChars);
  }
  return joined;
}

function filterLinesWithContext(lines, keywords, context) {
  var keep = {};
  for (var i = 0; i < lines.length; i++) {
    var line = lines[i];
    if (matchesPriceLine(line) || lineHasKeyword(line, keywords)) {
      for (var j = Math.max(0, i - context); j <= i + context; j++) {
        keep[j] = true;
      }
    }
  }

  var result = [];
  var keys = Object.keys(keep).sort(function (a, b) {
    return Number(a) - Number(b);
  });
  for (var k = 0; k < keys.length; k++) {
    result.push(lines[Number(keys[k])]);
  }
  return result.length ? result : lines.slice(0, 200);
}

function matchesPriceLine(line) {
  if (!line) return false;
  return /\$|€|¥|£|\bUSD\b|\bJPY\b|\bEUR\b|\bGBP\b|\/mo|\/year|monthly|annual|per month|per year/i.test(
    line,
  );
}

function lineHasKeyword(line, keywords) {
  if (!line || !keywords || !keywords.length) return false;
  var lower = line.toLowerCase();
  for (var i = 0; i < keywords.length; i++) {
    if (lower.indexOf(String(keywords[i]).toLowerCase()) !== -1) {
      return true;
    }
  }
  return false;
}

function llmExtractSubscriptions(pageText, sourceId) {
  var systemPrompt =
    "You extract subscription entries from billing page text. " +
    "Return ONLY JSON array. Each item must include: " +
    "service, plan, price, currency, billing_period, next_billing_date, source, notes. " +
    "If unknown, use empty string. Do not add extra fields.";

  var userPrompt = "Source: " + sourceId + "\nText:\n" + pageText;

  var raw = sanitizeLlmOutput(llm_chat.chat(systemPrompt, userPrompt));
  var json = extractJsonArray(raw);
  if (!json) {
    console.log("  [WARN] LLM returned no JSON for " + sourceId);
    return [];
  }

  var parsed = safeJsonParse(json);
  if (!Array.isArray(parsed)) {
    console.log("  [WARN] LLM output not an array for " + sourceId);
    return [];
  }

  for (var i = 0; i < parsed.length; i++) {
    parsed[i].source = parsed[i].source || sourceId;
  }

  console.log("  Extracted " + parsed.length + " entries from " + sourceId);
  return parsed;
}

function normalizeSubscriptions(entries) {
  var normalized = [];
  for (var i = 0; i < entries.length; i++) {
    var it = entries[i] || {};
    var priceInfo = parsePrice(String(it.price || ""));

    normalized.push({
      service: String(it.service || "").trim(),
      plan: String(it.plan || "").trim(),
      price: priceInfo.amount,
      currency: priceInfo.currency || String(it.currency || "").trim(),
      billing_period: normalizePeriod(
        String(it.billing_period || priceInfo.period || ""),
      ),
      next_billing_date: String(it.next_billing_date || "").trim(),
      source: String(it.source || "").trim(),
      notes: String(it.notes || "").trim(),
      raw_price: String(it.price || "").trim(),
    });
  }
  return normalized;
}

function buildPricingCatalog(subscriptions, pricingUrls) {
  var uniqueServices = {};
  for (var i = 0; i < subscriptions.length; i++) {
    var key = normalizeServiceKey(subscriptions[i].service);
    if (key) {
      uniqueServices[key] = subscriptions[i].service;
    }
  }

  var catalog = [];
  var keys = Object.keys(uniqueServices);
  for (var k = 0; k < keys.length; k++) {
    var svcKey = keys[k];
    var svcName = uniqueServices[svcKey];
    var pricingUrl = pricingUrls[svcKey] || "";

    if (!pricingUrl) {
      catalog.push({
        service: svcName,
        pricing_url: "",
        plans: [],
        notes: "pricing url not found",
      });
      continue;
    }

    console.log("\n[Pricing] " + svcName + " -> " + pricingUrl);
    var tabId = null;
    try {
      tabId = floorp.createTab(pricingUrl, false);
      floorp.tabWaitForElement(tabId, "body", 15000);
      floorp.tabWaitForNetworkIdle(tabId);
      safeSleep(1200);

      var pageText = collectPricingText(
        tabId,
        svcKey,
        config.maxPageChars,
        config.serviceKeywords,
      );
      var pricing = llmExtractPricing(pageText, svcName, pricingUrl);
      catalog.push(pricing);
    } catch (e) {
      console.log("  [WARN] Pricing fetch failed for " + svcName + ": " + e);
      catalog.push({
        service: svcName,
        pricing_url: pricingUrl,
        plans: [],
        notes: "pricing fetch failed",
      });
    } finally {
      if (tabId) {
        floorp.destroyTabInstance(tabId);
      }
    }
  }

  return catalog;
}

function llmExtractPricing(pageText, serviceName, pricingUrl) {
  var systemPrompt =
    "You extract pricing plans from a pricing page. " +
    "Return ONLY JSON object with fields: service, pricing_url, plans, notes. " +
    "Each plan item: name, price, currency, billing_period, tokens, model_notes. " +
    "Use empty strings for unknown values.";

  var userPrompt =
    "Service: " +
    serviceName +
    "\nPricing URL: " +
    pricingUrl +
    "\nPage text:\n" +
    pageText;

  var raw = sanitizeLlmOutput(llm_chat.chat(systemPrompt, userPrompt));
  var json = extractJsonObject(raw);
  if (!json) {
    return {
      service: serviceName,
      pricing_url: pricingUrl,
      plans: [],
      notes: "llm parsing failed",
    };
  }

  var parsed = safeJsonParse(json);
  if (!parsed || typeof parsed !== "object") {
    return {
      service: serviceName,
      pricing_url: pricingUrl,
      plans: [],
      notes: "llm parsing failed",
    };
  }

  parsed.service = parsed.service || serviceName;
  parsed.pricing_url = parsed.pricing_url || pricingUrl;
  parsed.plans = Array.isArray(parsed.plans) ? parsed.plans : [];
  parsed.notes = parsed.notes || "";

  return parsed;
}

function llmRecommend(subscriptions, pricingCatalog) {
  var systemPrompt =
    "You recommend savings and alternatives for programming AI subscriptions. " +
    "Return ONLY JSON array. Each item fields: service, action, reason, alternatives.";

  var userPrompt =
    "Subscriptions:\n" +
    JSON.stringify(subscriptions) +
    "\n\nPricing Catalog:\n" +
    JSON.stringify(pricingCatalog);

  var raw = sanitizeLlmOutput(llm_chat.chat(systemPrompt, userPrompt));
  var json = extractJsonArray(raw);
  if (!json) {
    return [];
  }

  var parsed = safeJsonParse(json);
  return Array.isArray(parsed) ? parsed : [];
}

function buildMarkdownReport(subscriptions, pricingCatalog, recommendations) {
  var lines = [];
  lines.push("# Subscription Deep Research Report");
  lines.push("");
  lines.push("## Subscriptions");
  lines.push("");
  lines.push(
    "| Service | Plan | Price | Currency | Period | Next Billing | Source | Notes |",
  );
  lines.push("|---|---|---:|---|---|---|---|---|");

  for (var i = 0; i < subscriptions.length; i++) {
    var s = subscriptions[i];
    lines.push(
      "| " +
        safeMd(s.service) +
        " | " +
        safeMd(s.plan) +
        " | " +
        safeMd(formatPrice(s.price, s.raw_price)) +
        " | " +
        safeMd(s.currency) +
        " | " +
        safeMd(s.billing_period) +
        " | " +
        safeMd(s.next_billing_date) +
        " | " +
        safeMd(s.source) +
        " | " +
        safeMd(s.notes) +
        " |",
    );
  }

  lines.push("");
  lines.push("## Pricing Catalog");
  lines.push("");

  for (var p = 0; p < pricingCatalog.length; p++) {
    var c = pricingCatalog[p];
    lines.push("### " + safeMd(c.service));
    if (c.pricing_url) {
      lines.push("- Pricing URL: " + c.pricing_url);
    } else {
      lines.push("- Pricing URL: (not found)");
    }
    if (c.notes) {
      lines.push("- Notes: " + safeMd(c.notes));
    }
    lines.push("");
    lines.push("| Plan | Price | Currency | Period | Tokens | Model Notes |");
    lines.push("|---|---:|---|---|---|---|");

    for (var j = 0; j < c.plans.length; j++) {
      var plan = c.plans[j];
      lines.push(
        "| " +
          safeMd(plan.name) +
          " | " +
          safeMd(plan.price) +
          " | " +
          safeMd(plan.currency) +
          " | " +
          safeMd(plan.billing_period) +
          " | " +
          safeMd(plan.tokens) +
          " | " +
          safeMd(plan.model_notes) +
          " |",
      );
    }

    if (!c.plans.length) {
      lines.push("| (no plans extracted) | | | | | |");
    }

    lines.push("");
  }

  lines.push("## Recommendations");
  lines.push("");

  if (!recommendations.length) {
    lines.push("No recommendations generated.");
  } else {
    for (var r = 0; r < recommendations.length; r++) {
      var rec = recommendations[r];
      lines.push("- Service: " + safeMd(rec.service));
      lines.push("  - Action: " + safeMd(rec.action));
      lines.push("  - Reason: " + safeMd(rec.reason));
      lines.push("  - Alternatives: " + safeMd(rec.alternatives));
    }
  }

  lines.push("");
  lines.push("## Manual Review Checklist");
  lines.push("- Verify any missing pricing URLs");
  lines.push("- Confirm token limits and model access from official pages");
  lines.push("- Review recommendations before canceling subscriptions");

  return lines.join("\n");
}

function safeMd(value) {
  var str = String(value || "");
  return str.replace(/\|/g, "\\|").replace(/\n/g, " ");
}

function formatPrice(price, raw) {
  if (price === 0 && raw) {
    return raw;
  }
  if (price === 0) {
    return "";
  }
  return String(price);
}

function parsePrice(text) {
  var cleaned = text.trim();
  var match = cleaned.match(/([\$€¥£])?\s*([0-9]+(?:\.[0-9]+)?)/);
  if (!match) {
    return { amount: 0, currency: "", period: "" };
  }

  var currencySymbol = match[1] || "";
  var amount = parseFloat(match[2] || "0");
  var currency = "";
  if (currencySymbol === "$" || currencySymbol === "") {
    currency = "USD";
  } else if (currencySymbol === "€") {
    currency = "EUR";
  } else if (currencySymbol === "¥") {
    currency = "JPY";
  } else if (currencySymbol === "£") {
    currency = "GBP";
  }

  var period = "";
  if (/year|annual|yr/i.test(cleaned)) {
    period = "yearly";
  } else if (/month|mo/i.test(cleaned)) {
    period = "monthly";
  }

  return { amount: amount, currency: currency, period: period };
}

function normalizePeriod(period) {
  var p = String(period || "").toLowerCase();
  if (p.indexOf("year") !== -1 || p.indexOf("annual") !== -1) {
    return "yearly";
  }
  if (p.indexOf("month") !== -1 || p.indexOf("mo") !== -1) {
    return "monthly";
  }
  return p;
}

function normalizeServiceKey(serviceName) {
  var s = String(serviceName || "").toLowerCase();
  if (s.indexOf("cursor") !== -1) return "cursor";
  if (s.indexOf("copilot") !== -1 || s.indexOf("github") !== -1)
    return "copilot";
  if (s.indexOf("openai") !== -1 || s.indexOf("chatgpt") !== -1)
    return "openai";
  if (s.indexOf("claude") !== -1 || s.indexOf("anthropic") !== -1)
    return "claude";
  if (s.indexOf("z.ai") !== -1 || s.indexOf("zai") !== -1) return "zai";
  return "";
}

function extractJsonArray(text) {
  if (!text) return "";
  var match = text.match(/\[[\s\S]*\]/);
  return match ? match[0] : "";
}

function extractJsonObject(text) {
  if (!text) return "";
  var match = text.match(/\{[\s\S]*\}/);
  return match ? match[0] : "";
}

function sanitizeLlmOutput(text) {
  if (!text) return "";
  var cleaned = String(text);
  cleaned = cleaned.replace(/```[\s\S]*?```/g, function (block) {
    return block.replace(/```[a-zA-Z]*\n?/, "").replace(/```$/, "");
  });
  return cleaned.trim();
}

function safeJsonParse(text) {
  if (!text) return null;
  try {
    return JSON.parse(text);
  } catch (e) {
    return null;
  }
}

function shellEscape(path) {
  var safe = String(path || "").replace(/'/g, "'\\''");
  return "'" + safe + "'";
}

function writeFileByExec(path, content) {
  var delimiter = "__SAPPHILLON_EOF__";
  var safeContent = String(content || "").replace(
    new RegExp(delimiter, "g"),
    delimiter + "_",
  );
  var cmd =
    "cat <<'" +
    delimiter +
    "' > " +
    shellEscape(path) +
    "\n" +
    safeContent +
    "\n" +
    delimiter;
  return app.sapphillon.core.exec.exec(cmd);
}
