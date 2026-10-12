package com.example.shared

fun show(box: LibBox): String = libHelper("v" + box[1] + (box + box).value)

fun main() {
    println(show(LibBox(2)))
}
