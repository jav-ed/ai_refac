Project_Manag/Docs/doc_Start.md can be used to get a quick understanding of this project inshallah

Before you build, add a test file or start a new background tool, read Project_Manag/Docs/Setup/resource_Use.md: the Cargo profile keeps debug information out of the build (a full one once wrote 19 GB and filled the sandbox disk), integration tests are grouped as tests/<group>/main.rs and a new .rs file directly in tests/ is not allowed, and every tool you start must be stopped again and have its memory measured.

The real-server Kotlin tests: put the ones you want into ONE `cargo test` command (the library-level tests share one server per fixture inside it, 40 seconds once; the dispatch, dry-run and server tests start their own at 1 to 2 minutes each), never the whole group (21 minutes) as a routine. Project_Manag/Docs/Setup/kotlin_Server.md lists which tests cover which change.
