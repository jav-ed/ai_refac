# Per-page social cards: what every site does

Rules and parts that hold on any stack. A site that renders one card per page follows this page, then reads its stack page (`astro`, `fuma`) for how its framework does parts 1 and 5. Tool facts (arguments, size limits, cards) are on [the Og_Image page](../og_Image.md).

Three layers, so nobody learns it twice:

| Layer | Holds | Lives in |
| --- | --- | --- |
| General | These rules and parts | this skill |
| Stack | Where the page list comes from, where the tags go, the templates | the `astro` or `fuma` skill |
| Site | Brand, colors, variant, deviations, open decisions | the site's own repository: one page `og_Cards.md` ([template](site_Page_Template.md)), or a folder `Og_Images/` with a `linker_Og_Images.md` once there are several pages (system, build, add to a page, theme, findings) |

Every site keeps that page, linked from its doc start. A rule here is not repeated in a stack skill or a site page; they point to it.

## The seven parts

| # | Part | Job |
| --- | --- | --- |
| 1 | Page list | The one list of pages that the prerender, the sitemap and the cards all read, so a page cannot lack one of them |
| 2 | Name rule | One function turns a page URL into the card name; the page head and the manifest script both call it |
| 3 | Manifest | A script writes `Cache/Og_Gen/og_Manifest.json` from the page list and the content |
| 4 | Step | After the build and before compression, the pipeline runs the generator on the site |
| 5 | Head tags | Each page head links its own card (table below) |
| 6 | Guard | The check step fails a release when a page links a missing card, a card has the wrong size, or a card has no page |
| 7 | Ignore | `Cache/` and the build folder are ignored by git; nothing generated is committed |

## Rules

- **Generated, not tracked.** The manifest and the cards are build output. The ordinary release copy of the build folder takes the cards to the host: no extra upload, no artifact store, no growth of the git history. A committed card could only drift from the content.
- **No borrowed card.** A page has its own card or none; the root head has no `og:image` that every page inherits. A site where every page needs a card (docs) fails its guard on a page without one. A site where cards are opt-in (a blog) emits no `og:image` tag for the pages without.
- **Text comes from the content.** Title and description are single-line frontmatter values; a missing one stops the manifest script and names the file.
- **Card copy is display copy.** Strip one final full stop from title and description (keep `?` and `!`), because a card reads as a title, not a sentence. The page's own `<meta description>` keeps the prose.
- **One path constant.** The folder the cards are written to, the URL prefix and the file extension each come from one constant that the manifest, the head and the guard all read. The folder name is the site's choice (`dist/Og_Gen` on the Astro sites, `dist/client/og` on the Fumadocs site); the tool writes to `<project_Root>/<dirname(rel_Path_Og_Fold)>/<out_Path>`.
- **Context in the title.** A card is seen without the sidebar, so a page inside a section carries the section title in front (`Kotlin / Android: Rename`). The tab title and `og:title` carry the same text, built by one function that the page head and the card script both use, and the guard compares the built `<title>` with the card title. A site may choose differently, for example the card only, and says so in its own page.
- **WebP.** Tested on a site with 141 cards against Facebook, LinkedIn, X and Mastodon: WebP was not the cause of any failure, and it averaged 30 KB against 53 KB for JPEG (4.2 MB against 7.3 MB in total). The tool fails a card above 300 KB, and WhatsApp is reported to drop previews of about that size (a secondary source). Avoid AVIF, which many ignore. Keep the format in one constant that the file name, the manifest and `og:image:type` all derive from, and test LinkedIn, X and Mastodon again after any change.
- **Concrete colors, real fonts.** The renderer cannot read CSS variables: write the theme as `oklch(...)` values and keep them in step with the site's CSS (two sites tried parsing the CSS, found it fragile and hardcode the values). Fonts are files the manifest names.
- **Keep the `<head>` tag.** HTML5 lets a page drop `<head>` and a minifier may do it. Browsers and Facebook still read the meta tags, but LinkedIn, X and Mastodon then ignore `og:image`, and the card fails only on the platforms that matter. Keep the tag (for `@swc/html`, `tagOmission: "keep-head-and-body"`), keep attribute values quoted, and let the guard test for the tag.
- **Look at the longest card.** Titles wrap; test the longest title and description of the site before relying on a layout.

## Head tags

| Tag | Value |
| --- | --- |
| `og:image` | Absolute URL of the card: the origin from one constant, the name from the name rule |
| `og:image:type` | The media type of the format the tool writes (`image/webp`), derived from the format constant |
| `og:image:width`, `og:image:height` | `1200`, `630` |
| `og:image:alt` | The text on the card: title and description |
| `twitter:card` | `summary_large_image`, in the root head. X falls back to the `og:` tags for title, description and image; a stack that emits `twitter:*` tags explicitly may keep doing so |

Crawlers reject relative URLs, and they read the prerendered HTML: the tags must be in the static page, not added by script.

## Verify

1. Run the build, the card step and the check by hand (the check includes `<head>` and the card sizes), and open two cards, the one with the longest description first.
2. Read one built page: the `og:image` URL must match a file in the card folder.
3. After a deploy, request one card from the live site: status 200, `image/webp`. A forgotten copy or a host rule shows only when someone shares a link otherwise.
4. Platforms cache cards for hours to days. Force a refresh in their debuggers (Facebook sharing debugger, LinkedIn post inspector); the card path stays the same.
