# Z.ai selectors memo

## Status

- Logged in and navigated to /manage-apikey/subscription
- Active plan confirmed in tabpanel
- Promo dialog may appear (Value Subscription)

## Navigation

- API Management sidebar: `main > div > div > h2` (API Management)
- Subscription link in sidebar: `a[href='/manage-apikey/subscription']`

## Subscription panel

- Panel heading: `main > div > div > h2` (contains "Subscription")
- Subscription tablist: `div[role='tablist']`
- Subscription tabpanel: `div[role='tabpanel']`
- Empty state text: `div[role='tabpanel']` (contains "You don't have any subscription")
- Active plan text sample (tabpanel):
  - GLM Coding Pro-Quarterly Plan
  - Status: Valid
  - Auto-renew: 2026.04.21
  - Price: $45 (1st Quarter)
  - Perks: Vision Analyze / Web Search / Web Reader / Zread MCP

## Promo dialog (optional)

- Dialog container: `div[role='dialog']`
- Close button: `button:has-text('Close')`
- Join Now button: `button:has-text('Join Now')`

## Notes

- If a subscription exists, scrape within `div[role='tabpanel']` for plan/price
- Dismiss promo dialog before scraping for stable content
