# GitHub Copilot billing selectors memo

## Status

- Logged in and opened https://github.com/settings/billing
- Billing overview contains subscription cards

## Primary selectors

- Billing summary container: `[data-hpc]`
- Copilot plan card: `[data-testid='copilot-plan-card']`
- Copilot tab (sidebar): `[data-testid='copilot-tab']`

## Sample text (copilot plan card)

- Copilot Pro+
- Downgrade Pending
- $39.00 per month

## Notes

- Prefer `[data-testid='copilot-plan-card']` for plan/price
- `[data-hpc]` includes other subscriptions (GitHub Pro, etc.) if needed
