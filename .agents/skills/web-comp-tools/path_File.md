# path_File

Every hard-coded path of this skill is in this file and nowhere else. The other pages write a name such as `Compressor_Home` and nothing more, so when a folder moves, one edit here fixes the whole skill.

**Why one file.** Folders move, and a path that is written in many files breaks in many places, each one found only when a command fails. A single list is checked once, edited once, and tested with the loop at the end of this page.

## How to use it

When a page names `Compressor_Home` or `Og_Image_Home` and you need the real folder, take it from the block below. Paste the block into the shell first and the names work as variables (`"$Compressor_Home"`), so the commands on the other pages run as written.

## The paths

```bash
export Web_Tools_Home=/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/01_Projects/07_Infrastruc/02_Web_Tools

export Compressor_Home=$Web_Tools_Home/02_Workfl_Comp/Compressor
export Og_Image_Home=$Web_Tools_Home/02_Workfl_Comp/Og_Image
export Index_Now_Home=$Web_Tools_Home/02_Workfl_Comp/Index_Now
export CI_CD_Home=$Web_Tools_Home/02_Workfl_Comp/CI_CD
export Web_Audit_Home=$Web_Tools_Home/01_Helper_Tools/01_Web_Audit

export Compressor_Bin=/home/jav/.local/bin/compressor
export Index_Now_Bin=/home/jav/.local/bin/indexnow
```

`Compressor_Bin` and `Index_Now_Bin` are symlinks in a folder that is on `PATH`, not the build output itself. Each points at the tool's own `target/release/` file, so every `cargo build --release` is available through the link at once, and a pipeline's `env.env` names the link, which does not change when the tool's repository moves. Create or repair them with:

```bash
ln -sfn "$Compressor_Home/target/release/compressor" "$Compressor_Bin"
ln -sfn "$Index_Now_Home/target/release/indexnow" "$Index_Now_Bin"
```

| Name | What it points to | Pipeline `env.env` key that copies it |
| --- | --- | --- |
| `Web_Tools_Home` | The folder that holds all the tool repositories | none |
| `Compressor_Home`, `Compressor_Bin` | The Compressor repository, and the link to its release binary | `CI_COMPRESSOR_BIN` takes `Compressor_Bin` |
| `Og_Image_Home` | The Og_Image repository (run with `bun src/main.js` inside it) | `CI_OG_IMAGE_DIR` |
| `Index_Now_Home`, `Index_Now_Bin` | The Index_Now repository, and the link to its release binary | `CI_INDEXNOW_LOCAL_BIN` takes `Index_Now_Bin` |
| `Web_Audit_Home` | The Web_Audit repository (installed once with `bun link`, run as `audit`) | none |
| `CI_CD_Home` | The Woodpecker and Forgejo runtime; the `ci-cd` skill owns it | none |

## When a folder moves or a tool is added

1. Edit the block above, and only that block. For a moved tool repository that is its `Home` line, then the two `ln -sfn` lines above to repair the link.
2. Run the check below. Every name must say `ok`.
3. Copy the changed values into the `env.env` of every project that uses them (the keys in the table). A moved repository does not change `Compressor_Bin` or `Index_Now_Bin`, so their projects need no edit. A project's pipeline has to work without this skill installed, so its `env.env` holds its own copy of the value; it cannot read this file. Search the projects for the key (`rg CI_COMPRESSOR_BIN`) to find them.
4. Adding a tool: one `export` line for its home, one for its binary if it has one, and one row in the table.

```bash
check() { [ "$2" "$3" ] && echo "ok       $1" || echo "MISSING  $1 $4"; }   # works in bash and zsh
check Web_Tools_Home   -d "$Web_Tools_Home"
check Compressor_Home  -d "$Compressor_Home"
check Og_Image_Home    -d "$Og_Image_Home"
check Index_Now_Home   -d "$Index_Now_Home"
check CI_CD_Home       -d "$CI_CD_Home"
check Web_Audit_Home   -d "$Web_Audit_Home"
check Compressor_Bin   -x "$Compressor_Bin" "(build it: cargo build --release in Compressor_Home, then link it as above)"
check Index_Now_Bin    -x "$Index_Now_Bin"  "(build it: just build in Index_Now_Home, then link it as above)"
```

These paths are for the development machine. On another machine this file needs that machine's values; check which machine you are on first (the `remote-helper` skill).
