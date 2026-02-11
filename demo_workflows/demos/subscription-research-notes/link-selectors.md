# Link.com selectors memo

## Status

- Logged in and navigated to Link subscriptions page
- Detail view was open while inspecting selectors

## List view (active + inactive)

- List container: `main.RouteContent > div.ListAndDetailView > div.ListAndDetailView-list`
- List items (clickable): `button.TableListItem.SubscriptionTableListItem`
- Inactive toggle: `button:has-text("すべての非アクティブを表示")`

## Detail view

- Detail panel: `main.RouteContent > div.ListAndDetailView > div.ListAndDetailView-detail`
- Merchant header: `div.SubscriptionDetailView > div.LayoutHeading.MerchantHeader`
- Plan name: `div.SubscriptionDetailView > div.LayoutHeading.MerchantHeader h2.LayoutHeading-subheading`
- Detail block (status/price): `div.SubscriptionDetailView`

## Notes

- Use list item clicks to load detail panel, then read detail text for plan/price
- AI-related filtering keywords: Cursor, Copilot, OpenAI, Claude, Anthropic, Z.ai
