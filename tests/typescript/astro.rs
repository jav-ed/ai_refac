//! Astro projects through the CLI: the files that import a moved module are
//! often `.astro` and `.mdx`, and the moved file may be an `.astro` component.
//! The engine's own cases are in `scripts/Tests/TypeScript/components.test.ts`;
//! these prove the routing and the outcome a person sees.

use crate::common::project::Project;

mod components;
mod pages_and_pictures;
mod paths;
mod rename;

const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "baseUrl": ".",
    "paths": { "@components/*": ["src/components/*"] },
    "moduleResolution": "bundler",
    "module": "esnext"
  },
  "include": ["src/**/*"]
}"#;

fn astro_project() -> Project {
    let project = Project::empty();
    project.write("tsconfig.json", TSCONFIG);
    project.write(
        "src/lib/format.ts",
        "export const formatDate = (date: Date) => date.toISOString();\n",
    );
    project.write(
        "src/components/Card.astro",
        "---\nimport { formatDate } from \"../lib/format\";\n---\n<p>{formatDate(new Date())}</p>\n",
    );
    project.write(
        "src/pages/index.astro",
        "---\nimport Card from \"@components/Card.astro\";\nimport { formatDate } from \"../lib/format\";\n---\n<Card />{formatDate(new Date())}\n",
    );
    project.write(
        "src/content/post.mdx",
        "import Card from \"../components/Card.astro\";\nimport { formatDate } from \"../lib/format\";\n\n<Card />\n",
    );
    project
}
