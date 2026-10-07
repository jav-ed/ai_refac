# CI/CD

Use the shared `ci-cd` skill for platform setup and secrets. Adapt its static-site flow to this artifact:

1. install dependencies from the lockfile;
2. run type checking;
3. build Fumadocs and Pagefind;
4. precompress `dist/client`;
5. copy its contents into a versioned release directory;
6. atomically switch the `main` symlink;
7. verify the public health URL.

Keep the repository checkout path, runtime path, and compressor path explicit CI variables. The trigger can differ by Git forge; the artifact and release contract must not.
