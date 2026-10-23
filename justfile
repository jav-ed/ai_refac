
default:
    @just --list

[doc('Delete all Scratch contents and recreate empty Agent_Tasks, Audit, Design, and Screenshots folders.')]
scratch-clean:
    bash Code/Just/Scripts/clean_Scratch.sh
