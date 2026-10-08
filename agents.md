Project_Manag/Docs/doc_Start.md can be used to get a quick understanding of this project inshallah

Before you build, add a test file or start a new background tool, read Project_Manag/Docs/Setup/resource_Use.md: the Cargo profile keeps debug information out of the build (a full one once wrote 19 GB and filled the sandbox disk), integration tests are grouped as tests/<group>/main.rs and a new .rs file directly in tests/ is not allowed, and every tool you start must be stopped again and have its memory measured.
