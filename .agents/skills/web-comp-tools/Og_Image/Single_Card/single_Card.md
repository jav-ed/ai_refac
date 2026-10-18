# One card for the whole site

For a small site where one image for every page is enough: a card made from HTML, rendered once, committed as a PNG. No tool and no pipeline step. This is the only case where a card is committed, because here the PNG is a source file; generated per-page cards are build output and never committed ([per-page cards](../Per_Page_Cards/site_Checklist.md)).

The two files of this folder, `og-card.html` and `og_Image.sh`, are templates. A site copies them into its own repository and owns the copy, so its pipeline and its recipes do not need this skill to be installed.

1. Copy `og-card.html` to `Code/Brand/og-card.html` and replace the colors, the mark and the texts. Point the font path (relative to the file) at the font the site itself uses, so the card matches the site.
2. Copy `og_Image.sh` to `Code/Just/Scripts/og_Image.sh` and add a recipe that runs it. The script serves the repository on localhost (`playwright-cli` blocks `file:` URLs), waits until the card's font is usable (`OG_FONT`, default `700 40px Geist`: set it to the card's own font), renders at 1200 by 630 into `public/og.png`, and refuses any other size. It needs `playwright-cli`, ImageMagick and `python3`, and stops with a message when one is missing.
3. Open the PNG and look at it: the text must fit and nothing may be cut off.
4. In the head of the site's root layout add `og:image` (an absolute URL, crawlers do not resolve relative ones), `og:image:width` 1200, `og:image:height` 630 and `twitter:card` with `summary_large_image`. A page that sets its own `og:title` and `og:description` keeps these entries, because the heads merge.
5. Commit the PNG. It is remade only when the card changes, so the pipeline needs no step for it.

A mix is fine: the single card now, per-page cards later for the pages worth sharing. When the site moves to per-page cards, remove `public/og.png` and the root `og:image`, so a page without a card fails the guard instead of showing the generic one.
