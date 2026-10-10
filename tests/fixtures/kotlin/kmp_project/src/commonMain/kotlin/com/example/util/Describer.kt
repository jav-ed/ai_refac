package com.example.util

// Uses shout and the expect fun platformName of its own package without an
// import: the file that a move out of the package has to give them back to.
fun describe(text: String): String = shout(text) + platformName()
