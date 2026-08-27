# Product glossary

| Term | Meaning |
| --- | --- |
| Bronze | working product name; local macOS selection-to-action queue |
| capture | deliberate acquisition of currently selected text; never screenshot, screen recording, or ambient keystroke logging |
| trigger | user action requesting capture/show/new note, such as chord, modifier double tap, menu command |
| capture request | queued, ID-bearing attempt with provider stages and terminal result |
| selected text | source app’s current text selection exposed by AX or explicit clipboard path |
| queue | ordered transient work items intended for later copy/use/completion |
| section | named ordered queue container and active capture destination |
| context item | captured source text useful later |
| prompt item | user-authored instruction intended for later destination |
| note item | user-authored temporary text without implied task/AI semantics |
| lifecycle | queued, copied, active, done, skipped, trashed state model |
| output profile | deterministic format and post-copy behavior for one/multiple items |
| provenance | optional safe source app/title/URL/timestamp metadata |
| AX | macOS Accessibility API used to query focused element and selection |
| Input Monitoring | macOS privacy capability relevant to passive global event monitoring |
| Accessibility permission | macOS trust needed for selected-text and synthetic input paths |
| Secure Input | macOS mechanism suppressing keyboard observation in protected contexts; Bronze never bypasses it |
| manual clipboard capture | user explicitly copies selection, then asks Bronze to create from clipboard |
| synthetic fallback | experimental Bronze-generated Cmd-C path; bounded and off by default unless proven safe |
| local-only | no runtime content/network service, account, telemetry, or hosted AI; data on user Mac |
| conformance evidence | scoped criterion/build/platform/test record, not blanket “proof” |
