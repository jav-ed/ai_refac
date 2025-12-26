# Languages where script variants matter (and where they don’t)

## The general rule

For **almost all languages**, using a single language key (e.g. `en`, `fr`, `de`) is sufficient:

* Same writing system
* Same glyph shapes
* Same font families
* Differences are spelling or vocabulary only
  → **content concern, not a rendering concern**

**Chinese is the exception**, not the norm.

---

## Why Chinese is special

Chinese is the **only major modern language** where:

* The **same language** (`zh`)
* Is written using **two different, actively used scripts**
* And **script choice materially affects readability and font rendering**

### The two scripts

* **Simplified Chinese** → `zh-Hans`
* **Traditional Chinese** → `zh-Hant`

Using the wrong one is:

* immediately visible
* perceived as careless
* often unacceptable to native readers

That is why Chinese requires explicit handling.

---

## Practical reality of Chinese usage today (important)

In modern products and websites, **script correctness is the key requirement** — not regional perfection.

In practice, the industry norm is:

* **One Simplified Chinese version** for all Simplified regions (China, Singapore, Malaysia, etc.)
* **One Traditional Chinese version** for all Traditional regions (Taiwan, Hong Kong, Macau)

Regional wording differences *within the same script* (e.g. Mainland vs Singapore Simplified, or Taiwan vs Hong Kong Traditional):

* are widely understood
* rarely cause confusion
* are generally accepted by users
* only matter for highly localized, market-specific content

For translated (non-primary) languages, this approach is considered **fully professional**.

---

## Languages where script variants exist (but usually don’t matter for us)

These languages *can* be written in multiple scripts, but are **not relevant for our current font or routing policy**.

### Serbian

* Scripts: Latin (`sr-Latn`), Cyrillic (`sr-Cyrl`)
* Regions: Serbia, Bosnia
* Why it’s different:

  * Script choice is explicit
  * Often treated as separate language variants
  * Rare in typical international websites

### Azerbaijani

* Scripts: Latin, Cyrillic, Arabic (historical)
* Region: Azerbaijan
* Modern web usage is overwhelmingly Latin

### Uzbek

* Scripts: Latin, Cyrillic
* Region: Uzbekistan
* Script switching exists, but rarely needed outside local contexts

### Kazakh

* Scripts: Cyrillic → transitioning to Latin
* Region: Kazakhstan
* Transitionary, but not a concern for general products

**Key point:**
These languages *have script subtags*, but they are:

* rare in global products
* usually modeled as separate locales
* not required for correct font fallback in our scope

---

## Languages people *think* are problematic (but aren’t)

### Japanese

* Mixes Kanji, Hiragana, Katakana
* **Single unified system**
* No script variants to select

### Korean

* Hangul vs historical Hanja
* Modern Korean uses Hangul exclusively

### Arabic

* One script across all regions
* Regional differences are vocabulary only

### European languages

Examples:

* English (US vs UK)
* German (ß vs ss)
* French accents
* Spanish regional vocabulary

All of these:

* use the same script
* use the same fonts
* do **not** require script-aware handling

---

## The practical policy (what we actually enforce)

### We actively care about:

* **Chinese**

  * `zh-Hans` (Simplified)
  * `zh-Hant` (Traditional)

These two variants ensure correct glyphs, fonts, and readability.

### We explicitly do *not* special-case (for now):

* Serbian
* Uzbek
* Azerbaijani
* Kazakh
* Any European language

This keeps the system:

* correct
* professional
* minimal
* easy to reason about

---

## One-sentence internal rule (recommended)

> **Chinese is the only language in our scope where script variants materially affect readability and font rendering; in practice, one Simplified (`zh-Hans`) and one Traditional (`zh-Hant`) variant are sufficient, while all other languages are handled using base language keys only.**

---

### Why this wording matters

This version:

* preserves the **technical justification**
* documents the **real-world norm**
* avoids over-engineering
* leaves a **clear upgrade path** for region-specific Chinese later
* gives future readers context for *why* the decision was made

If you want, next we can:

* align this 1:1 with the code comments your colleague wrote
* add a short **“Decision log / rationale”** block
* or trim it into a shorter executive summary

This doc is now solid, defensible, and accurate to how things are actually done.
