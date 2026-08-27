# Copper reference and video analysis

Research date: 2026-08-27. Reference: [Copper landing page](https://shadcn.com/copper).

## Product model

Copper describes itself as useful parts of to-do list, clipboard, and scratchpad for AI-assisted work. It addresses thought interruption: while talking with ChatGPT, Claude, or Cursor, user notices context or a future prompt but does not want to leave current flow.

Advertised loop:

1. Select text in any application.
2. Double-tap Shift to capture it.
3. Add notes or prompts already in mind.
4. Organize items into sections.
5. Copy one or several items into chat/editor.
6. Mark work complete.

Page claims local file storage, no account, no sync, no collection, macOS 14+, Accessibility access, $39 one-time purchase, free updates, and 30-day refund. [Privacy policy](https://shadcn.com/copper/privacy) says app has no analytics, telemetry, crash reporting, or usage collection; license/payment use Lemon Squeezy, and site uses Vercel Analytics. [Terms](https://shadcn.com/copper/terms) describe personal use on own Macs, included updates, backup responsibility, and 30-day refund. Treat all as first-party claims and recheck before comparison publication.

## Video evidence

Landing video: 47.253 seconds, 1920×1080 source, embedded muted without visible controls or text tracks. Absence of captions and user controls is itself poor web-media accessibility, even though video is marketing rather than app UI.

### Timeline

| Time | On-screen behavior | Product requirement inferred |
| --- | --- | --- |
| 0–2 s | “Capture the ‘I’ll need this later’ with a quick shortcut.” | trigger must preserve focus and flow |
| 2–18 s | Text selected in AI chat; double Shift; “Captured” toast; cards append under sections | deliberate selection capture, deterministic target section, feedback |
| 18–28 s | “It works everywhere”; ChatGPT, Claude, Cursor, Chrome, apps/chats/tabs | cross-application compatibility is central promise |
| 28–34 s | User types two future prompts into bottom composer; Enter creates each | frictionless manual entry and input reset |
| 34–43 s | Multi-selection; context menu; Copy as List; numbered content pasted back; items become checked | batch formatting, copy-back, completion state |
| 43–47 s | Feature slate | Merge Notes, Sections, Markdown, Copy as List, Search, Custom Shortcuts, Local Files, No Tracking, No Account, Free Updates, Keyboard-First, Native Mac App |

Visible context actions include Copy (`⌘C`), Copy as List (`⇧⌘C`), Mark as Done (`Space`), Edit (`Return`), Edit in New Window (`⌘Return`), Merge Notes (`⇧⌘M`), and Move submenu. “Expand” appears disabled in demonstrated state.

## Interaction inventory

- Compact floating side panel remains adjacent to primary work, not a blocking modal.
- Header contains search and overflow/settings affordance.
- Items grouped under section headings such as Research and Configuration Formats.
- Cards have completion/select affordance, concise content, soft surface, and multi-select outline.
- Bottom composer creates prompt/note without leaving panel.
- Success feedback appears as transient toast.
- Batch copy offers numbered-list serialization and automatically changes demonstrated items to completed/struck state.

## Visual analysis

Landing page is restrained editorial design: Inter, near-black `#09090b`, white canvas, narrow left-aligned text, modest 16 px headline, large whitespace, and 16:9 demonstration. App panel uses quiet rounded cards, compact spacing, soft shadow/translucency, and small typography. This restraint supports focus, but Bronze must establish its own identity and accessible scale rather than reproduce exact measurements or look.

Bronze direction:

- Warm bronze accent only for focus, selection, and status; no color-only meaning.
- Text and hit areas larger than demonstrated where needed.
- Solid-background fallback for Reduce Transparency and Increase Contrast.
- Persistent focus rings, semantic status, visible shortcut alternatives.
- Source provenance and capture health visible without clutter.

See [original concept](../assets/bronze-ui-concept.png).

## Product strengths worth preserving

- One coherent loop, not feature collection.
- Deliberate capture avoids passive-surveillance feel.
- Manual prompts and captured context share same queue.
- Copy-back makes data actionable where user already works.
- Local/no-account pitch is simple and credible when network truly absent.
- Keyboard-first operation supports speed.

## Gaps Bronze should fix

- Double Shift cannot be only capture route; it is timing-sensitive, discoverability-poor, and difficult for some motor users.
- “Works everywhere” needs tested compatibility definition and failure UX.
- Copy should not silently complete unless user opts into that lifecycle.
- Capture should record optional source provenance and explain secure-field exclusions.
- Permissions need live pipeline test, not binary authorization status.
- No public accessibility conformance, i18n, keyboard-layout, or assistive-technology evidence was found.
- Marketing video itself lacks controls/text track.
- Storage needs backup, export/import, migrations, integrity, and recovery—not merely “one local file.”

## Independent implementation boundary

Use this analysis as behavioral reference. Do not use Copper name, logo, screenshots, video frames, marketing copy, icons, or pixel-identical layout. Cooper source was inspected during reliability research, so do not describe Bronze as a legal clean-room implementation. Bronze v1 copies no Cooper source. Bronze concept asset is independently generated. Any future side-by-side design review should compare workflows and measurable ergonomics, not reproduce trade dress.
