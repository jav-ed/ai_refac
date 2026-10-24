import * as fs from "node:fs";
import { collectImports } from "./imports";
import { type ProjectConfig } from "./project";
import { makeResolver } from "./resolver";

/**
 * The component files (of `extensions`, such as "astro" and "mdx") that import
 * from one of the `defining` files and write `name` somewhere in their text. A symbol rename of the
 * TypeScript language server cannot read inside component files, so these are
 * the files it would leave behind. The text is searched, not only the code
 * blocks, because a template uses the names its frontmatter imports and a
 * property or method is used without being imported by that name.
 */
export function componentUsers(project: ProjectConfig, name: string, defining: string[], extensions: string[]): string[] {
    const targets = new Set(defining.map(file => fs.realpathSync.native(file)));
    const resolver = makeResolver(project);
    const mention = new RegExp(`(?<![\\p{ID_Continue}$])${name.replaceAll("$", "\\$")}(?![\\p{ID_Continue}$])`, "u");
    const users: string[] = [];
    for (const file of project.files.filter(file => extensions.some(extension => file.endsWith(`.${extension}`)))) {
        const text = fs.readFileSync(file, "utf8");
        if (!mention.test(text)) continue;
        const imports = collectImports(file, text).some(reference => {
            const resolved = resolver.resolve(file, reference.value, reference.mode);
            return resolved !== undefined && fs.existsSync(resolved) && targets.has(fs.realpathSync.native(resolved));
        });
        if (imports) users.push(file);
    }
    return users;
}
