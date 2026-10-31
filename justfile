
default:
    @just --list

[doc('Delete all Scratch contents and recreate empty Agent_Tasks, Audit, Design, and Screenshots folders.')]
scratch-clean:
    bash Code/Just/Scripts/clean_Scratch.sh

[doc('Install a build of the JetBrains Kotlin language server next to the one in use and check its license date; --clean also deletes older builds and their caches. Details: Project_Manag/Docs/Setup/kotlin_Server.md')]
kotlin-server-update build *flags:
    bash Code/Just/Scripts/update_Kotlin_Server.sh {{build}} {{flags}}
