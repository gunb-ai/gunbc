## Lane H-2 browser typed-boundary follow-up

Status: scoped transport receipt rather than handle-threading patch.

- `browser.Page.*` remains singleton-page today. None of the 11 page operations carries a `Page` argv slot, and the shell runner contract exposed in `dsl/extdeps/browser.dag` addresses the current page implicitly (`goto`, `url`, `title`, `wait-for`, `query-all`, `click`, `fill`, `evaluate`, `upload`, `screenshot`, `wait`). Adding `page: Page` at the DSL boundary without a matching transport capability would be a fake typed boundary.
- `browser.Element.*` remains selector-addressed today. `IsVisible`, `InnerText`, and `EvaluateOn` take selectors, and no browser operation currently returns an `Element` handle that later operations could consume. Replacing `selector: String` with `element: Element` now would create an uninhabited carrier path unless the transport grows element-handle production/consumption end-to-end.
- `url` stays `String`, not `std.types.Url`. The shared `Url` carrier narrows to `^https?://`, but current browser callers intentionally use browser-native locations like `about:blank`; `browser.Page.Goto(url: "about:blank")` is live in `wip/chatgpt_reviewer.dag`.
- `selector` stays `String`. Selectors are user-authored query expressions, not transport-produced opaque handles; branding them would not currently add a disjoint producer/consumer boundary the runner can enforce.

Follow-up boundary if this lane is revisited:

- Add transport support for explicit `Page` handle threading, then wire `page: Page` across `browser.Page.*`.
- Add transport support for `Element`-returning queries (for example, `WaitForSelector` / `QueryAll`) plus `Element`-consuming commands, then migrate `browser.Element.*` from selectors to handles.
