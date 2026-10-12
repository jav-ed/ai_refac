package com.example.shared

// The package com.example.shared is split over two modules: this one declares
// it, the app module adds Screen.kt to it and uses all of this without an import.

fun libHelper(text: String): String = text.uppercase()

class LibBox(val value: Int)

operator fun LibBox.get(index: Int) = value + index

operator fun LibBox.plus(other: LibBox) = LibBox(value + other.value)
