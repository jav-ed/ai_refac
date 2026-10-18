# Social cards (OG images)

<!-- Copy to the site's docs as `og_Cards.md` (a site with several OG pages keeps a folder `Og_Images/` with a `linker_Og_Images.md` instead and splits these headings into pages), link it from the doc start, and replace every <...>. This page holds only facts of THIS site. The rules and the seven parts are in the `web-comp-tools` skill (Og_Image, Per_Page_Cards); do not repeat them here, link them. -->

One paragraph: what a card is on this site, which pages have one, and that the cards are generated and never committed (rule: `web-comp-tools`).

## What a card shows

The card and variant of the generator (`<variant>`, and why it fits this site), the brand name, the logo file, the fonts and the colors, and any option (such as the Bismillah). One line each.

## Where everything is

| File | Job |
| --- | --- |
| `<name rule file>` | The name rule |
| `<route or layout file>` | The head tags |
| `<manifest script>` | Writes `Cache/Og_Gen/og_Manifest.json`; holds the brand name, the logo path and the theme |
| `<step script>` | The pipeline step |
| `<guard script>` | The check |
| `<env file>` | The variable that points at the generator folder |
| `.gitignore` | The ignored folders |

## The flow

The recipes to run by hand, in order, and what each leaves on disk.

## Changing the look

| Change | Where |
| --- | --- |
| Brand name | every place that holds it |
| Logo | the path constant |
| Colors | the theme in the manifest script and the site's CSS, together |
| Font | the font file and its entry in the manifest |
| Card layout | the variant, and the generator's guide to choosing a card |
| Text of one card | the frontmatter of the page |

## What stops the build

One line per hard error of the manifest script and the guard.

## Choices and open decisions

Where this site deviates from the general rules and why, and what is still undecided.
