# floorp

A plugin connecting to Floorp browser. This plugin allows you to control Floorp browser ex(. open a new tab, close a tab, navigate to a URL, etc.) through commands.

## API仕様

このプラグインは `api-spec` サブモジュールの OpenAPI 仕様 (`openapi.yaml`) を参照して Floorp OS API にアクセスします。

- API仕様: [api-spec/openapi.yaml](api-spec/openapi.yaml)
- 型定義: [api-spec/types.ts](api-spec/types.ts)

OpenAPI仕様に基づき、RustやTypeScriptクライアントの自動生成が可能です。
