# Android Layer

The Kotlin language server moves and renames code but never touches Android XML, and a file that moves out of its module's namespace package loses the implicit `R` and `BuildConfig` it used. This layer repairs both after the server is done, for `refac move` and for `refac rename`. A project without an Android module (a manifest under `src/<set>/` or a `namespace` in the build script) passes through untouched.

## Which module is Android

A module is Android when its `build.gradle(.kts)` sets `namespace = "..."` (Kotlin DSL) or `namespace '...'` (Groovy), or when `src/<set>/AndroidManifest.xml` exists next to it. A module that has a manifest but no readable namespace is an error that names the build script, because relative class names and generated classes could not be handled correctly. A root script that merely mentions the Android plugin does not make a module. The deepest module around a file owns it.

## Class names in XML

When classes change their fully qualified name (a move to another package, a rename), refac rewrites them in `AndroidManifest.xml` and in every `src/<set>/res/**/*.xml` of the owning module:

- element names that are classes (`<com.example.widgets.BadgeView>`),
- attribute values that are fully qualified class names (navigation `android:name`, `app:argType`, `class`, and so on),
- names relative to the namespace in the attributes that allow them: `android:name`, `tools:context`, `android:targetActivity`, `android:parentActivityName`, `android:fragment`, `app:fragment`, `class`, `android:backupAgent`, `android:manageSpaceActivity`. In the manifest a bare `MainActivity` also means `<namespace>.MainActivity`.

A relative name stays relative while the class stays under the namespace (`.MainActivity` becomes `.ui.MainActivity`), and becomes fully qualified when it leaves. Only the changed names are replaced in place, so formatting, comments, and line endings survive, and commented-out XML is left alone. XML that cannot be read safely (an unterminated tag, an unquoted attribute) is an error and nothing is written.

## `R` and `BuildConfig`

Code in the namespace package uses `R` and `BuildConfig` without an import. A file that moves out of that package, and uses either unqualified, gets `import <namespace>.R` and `import <namespace>.BuildConfig` added, in sorted position when the existing imports are sorted and after the last import otherwise, and with the file's line endings. An existing import of the same name (for example `android.R`) is respected, and uses such as `android.R.id.home` or text in comments do not count.

## Old names refac cannot rewrite

After the edits, refac scans the files it does not edit for the old fully qualified class names, including the `FileKt` facade class of a moved file that holds top-level functions: ProGuard rules, build scripts (`mainClass`), service lists, configuration (`properties`, `json`, `yaml`, `toml`), XML it did not change, and string literals in sources (reflection, `Class.forName`). Each hit becomes a note with the file, the old name, and the new name. Real code references were already updated by the server, so in sources only a quoted name counts. The scan only reports; it changes nothing.

## Not covered

- Resource ids and resource files (`R.string.x`, layout names) are never renamed; the server refuses resource id renames.
- View binding classes follow layout names, not Kotlin classes, so a moved activity needs no binding change.
- Multi-module Android projects with several libraries were not tested; the logic is per module, but treat the first such move as something to review.
