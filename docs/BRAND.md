# Brand and Logo Guidelines

This document is the reference for how the StellarForge name, logo, and visual
identity are used in the repository, documentation, releases, and community
material. It exists so contributions look consistent without anyone having to
ask.

## Name

- **Correct:** `StellarForge` — one word, capital `S`, capital `F`.
- **Also acceptable:** "the StellarForge protocol" or "StellarForge Protocol"
  when referring to the protocol as a whole.
- **Incorrect:** `Stellar Forge`, `Stellarforge`, `STELLARFORGE`, `SF` (except as
  a one-time abbreviation after first spelling out the name).
- Repositories, packages, and paths stay lowercase: `stellarforge`,
  `@stellarforge-protocol/sdk` (npm scope), `stellarforge-protocol` (crate
  prefix).
- The first mention in a document should use the full name with a link to
  <https://github.com/0xLizTech/stellarforge> where a link is appropriate.

## Logo

The primary logo is the wordmark lockup:
[`docs/assets/brand/stellarforge-wordmark.svg`](assets/brand/stellarforge-wordmark.svg).
It combines the orbital ring mark with the name.

### Clear space

Leave free space equal to the height of the `S` on every side of the lockup. Do
not crowd it with text, borders, or other logos.

### Minimum size

- Full lockup: 160 px wide minimum on screen; 32 mm wide in print.
- The ring mark alone may be used at 24 px or larger where the name appears
  adjacent in text (for example an avatar followed by a heading).

### Usage rules

- **Do not** stretch, rotate, recolor, or apply effects to the logo.
- **Do not** place the wordmark on a busy image without a solid or scrimmed
  background; use the dark or light surface colors below so contrast holds.
- **Do not** create modified versions or combine the mark with other symbols.
- On dark backgrounds, use the name in `#E2E8F0` with the ring in `#F59E0B`.

## Colors

| Name | Hex | Use |
| --- | --- | --- |
| Midnight | `#0B1220` | Primary text on light surfaces; dark surfaces |
| Ember | `#F59E0B` | Accent: the mark's core, highlights, callouts |
| Signal | `#38BDF8` | Links and informational states |
| Mist | `#E2E8F0` | Text on dark surfaces; light surfaces |
| Success | `#22C55E` | Positive states, passing checks |
| Danger | `#EF4444` | Destructive actions, failures |

Use Ember sparingly — it is an accent, not a background color. Status colors are
reserved for their status meaning.

## Typography

- **Text and UI:** [Inter](https://rsms.me/inter/) with a system fallback stack
  (`-apple-system, "Segoe UI", Roboto, Arial, sans-serif`).
- **Code, identifiers, and terminal output:** [JetBrains Mono](https://www.jetbrains.com/lp/mono/)
  with a monospace fallback (`ui-monospace, SFMono-Regular, Menlo, Consolas, monospace`).

## Voice and tone

- Precise and calm. State what the protocol does, then what it does not.
- Sentence case for headings and PR titles' subject lines (conventional-commit
  types stay lowercase: `fix(registry): ...`).
- Avoid hype words ("revolutionary", "guaranteed"). Security and financial
  claims must point to code, a test, or an audit — never a slogan.

## Use by third parties

You may use the name and unmodified logo to refer to this project (articles,
comparisons, integrations). You may not imply endorsement, sponsorship, or
official status without written permission from the maintainers, and you may not
use the marks in a way that suggests your fork or product *is* StellarForge.
