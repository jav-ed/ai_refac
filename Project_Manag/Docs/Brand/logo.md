# Logo

The mark is the doorway: a pointed arch, with a gold tile inside it, and below it an arrow and a faint tile that show where the tile came from. A piece moves through and the whole stays valid, which is what `refac` does. The arch is the Islamic architectural form (an equilateral pointed arch: two arcs, each as wide as the arch, each centred on the opposite foot); there is no calligraphy or figure in it on purpose, because a mark gets used on stickers, avatars, and screenshots.

On 2026-10-19 the doorway replaced the earlier mark, the moving-tile star (an eight-point star with a gold tile above it). The old files are kept in `Retired/`.

| File | Use |
|---|---|
| `refac-mark.svg` | The logo. Transparent, works on light and dark backgrounds (teal and gold only). README, docs header, sidebar and footer. Good from about 24 px up. |
| `refac-favicon.svg` | The arch with the gold tile only, heavier lines. No arrow and no faint tile: at tab size they would be a smear. Browser tab icon, and the mark on the social cards. |
| `refac-mark-512.png`, `refac-avatar-512.png` | Raster copies: transparent, and on a dark square (`#14181A`) for the GitHub avatar. |
| `refac-social-preview.png` | The GitHub social preview (1280 by 640): the docs site's home card on its own dark background, padded to the size GitHub asks for. GitHub has no API for it: upload it by hand under Settings, General, Social preview. Make it again from the docs repository's `dist/client/og/home.webp` when that card changes. |
| `refac-doorway.svg` | The same drawing with a finer arrow, for large empty spaces. The landing page draws it itself (`HeroArt` in the docs repository) and animates it: the arch draws itself, the tile rises through the arrow. |
| `Retired/` | The star mark, its favicon and rasters. The star lives on as the pattern behind the landing page. |

Use the logo wherever an icon is needed, and the favicon wherever it will be smaller than about 24 px. Do not draw anything new for a size in between: pick one of the two.

Colors: teal `#1D9E75` (arch, arrow, faint tile at 35 percent), gold `#EF9F27` (tile). Keep the faint tile faint and the gold tile gold: do not recolor one to match the other.

The geometry: the arch springs at y 40, is 40 wide and rises to y 5.36 in the 100 by 100 logo; the gold tile is 20 by 20 at (40, 28), the faint tile 20 by 20 at (40, 78). The favicon is the same arch at 19 wide in its 32 by 32 box. Edit these numbers and regenerate (`rsvg-convert` makes the PNGs) instead of nudging points by hand. The docs site copy of the favicon is `public/favicon.svg` in the docs repository, and its logo is `src/components/brand-mark.tsx`.
