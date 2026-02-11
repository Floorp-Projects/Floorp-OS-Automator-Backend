# Google Subscriptions selectors memo

## Status

- Logged in and opened https://myaccount.google.com/subscriptions
- Items are rendered under a role=main container

## Primary selectors

- Main container: `div[role='main']`
- Subscription items: `div[role='main'] [role='listitem']`
- Item link (if present): `div[role='main'] [role='listitem'] a`

## Sample item text

- Google One / Google AI Pro (2 TB) / 更新日: 2026/08/22
- YouTube Premium / ファミリーの定期購入 / 更新日: 2026年2月28日
- Google One / 100 GB / 有効期限: 2025/02/20

## Notes

- Some items include links to Play subscriptions; others are plain text
- Extract by iterating listitems and reading innerText
- Filter AI-related items by keywords ("AI", "Pro", "Google AI", "Gemini")
