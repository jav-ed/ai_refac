# Og_Image

A shared page needs an `og:image`, or a shared link shows text only. There are two methods. Pick by the shape of the site.

| Site | Method | Read |
| --- | --- | --- |
| Many pages that each deserve their own card (blog posts, services, localized pages, docs pages) | The Og_Image tool: one card per page, rendered from a manifest the site writes | this page for the tool, [what every site does](Per_Page_Cards/site_Checklist.md) for the rules and parts |
| A small site where one image for every page is enough | One card made from HTML, rendered once, committed, no tool and no pipeline step | [One card for the whole site](Single_Card/single_Card.md) |

A site that follows the first method keeps its own page about its cards in its own repository ([template](Per_Page_Cards/site_Page_Template.md)); this skill holds only what is true for every site.

## Per-page cards: the tool

Og_Image is a Bun program with the Takumi renderer. It reads a manifest, loads fonts, theme tokens and project assets, and renders every card at 1200 by 630 as WebP (default) or JPEG.

The site writes the manifest, the tool only reads it. For an Astro site the manifest is built during `astro build` by an Astro integration; the `astro` skill describes that side (the config files, the output format constant, the template catalogue). A Fumadocs site writes it with a small script; the `fuma` skill has the templates. Any other site writes the manifest with its own script; the sections, the `meta` fields and the theme tokens are in `Architecture/zetun_Web_Manifest_Shape.md` (own SVG logo) and `Architecture/javed_Manifest_Shape.md` (`jav_*` cards) of `Og_Image_Home`. A manifest problem is fixed in the site, a rendering problem in the tool.

### Which card

A manifest has one family (set by its header) and each task a variant. The common choices:

| The site | Take |
| --- | --- |
| Its own logo and colors, plain text cards (docs, internal tools, company sites) | `zetun_Web_Page` of the Zetun_Web family: brand name, the site's own SVG mark, title and description; `content.bismillah: true` adds the Bismillah |
| The look of javedab.com (Bismillah and half-moon logo fixed, serif titles) | `jav_*` of the Javed family, `jav_Page_Title` for plain pages |
| A client site with photographs | `zet_*` of the premium_Clinic family |

The other Zetun_Web variants (landing, olive panel, photo split and background, legal), what each shows and what it needs are in `Project_Manag/Docs/Features/choosing_A_Card.md` in `Og_Image_Home`. Render a card with the site's longest real title and description before relying on it.

### Run

```bash
cd "$Og_Image_Home" && bun src/main.js --project_Root /abs/path/to/site
```

| Argument | Required | Meaning |
| --- | --- | --- |
| `--project_Root` | yes | Absolute path of the site, any existing folder (no Astro `src/content` needed). `ASTRO_PROJECT_ROOT` is read when the argument is missing. |
| `--manifest` | no | Default `<project_Root>/Cache/Og_Gen/og_Manifest.json`. |
| `--format` | no | `webp` or `jpeg`. Beats `meta.img_Format` of the manifest, which beats the `webp` default. |

`OG_IMG_TEST_MODE` is for the tool's own tests; do not set it in a pipeline.

### What it does

1. Loads the manifest and fails hard on any structural problem.
2. Checks that `meta.project_Root` equals `--project_Root` (a safety lock against rendering into the wrong site).
3. Detects the scripts the texts need (Latin, Han, Arabic and so on), loads the fonts once, and renders each task.
4. Writes one image per task into `<project_Root>/<meta.rel_Path_Og_Fold>/` and creates the folder when needed.

The manifest names a template family: `Javed` (the `jav_*` templates, no version), `Zetun_Web` (`zetun_Web_*`) or `premium_Clinic` (`zet_*`); the last two use the contract `og-manifest/v_0_0_1`. An unknown family, template or theme token is an error, not a fallback.

### Size limits

| Format | Soft target | Hard limit (fails the build) |
| --- | --- | --- |
| `webp` | 150 KB | 300 KB |
| `jpeg` | 250 KB | 500 KB |

### In a pipeline

The OG step runs after the site build has written the manifest and before the compress step, so the cards are compressed with everything else. The step reads the tool folder from `CI_OG_IMAGE_DIR` in the project's `env.env`; the template belongs to the `ci-cd` skill and the value comes from [path_File.md](../path_File.md).

## Deeper

For changing the tool: `Project_Manag/Docs/Features/og_Image_Generator_Manual.md` (full manual, local preview of the clinic templates), `Architecture/manifest_Contract.md` and `Architecture/template_Families_And_Naming.md` in `Og_Image_Home`.
