package com.example.util

const val DEFAULT_GREETING = "hello"

class Helper(val prefix: String) {
    var counter: Int = 0

    fun decorate(text: String): String {
        counter++
        return "$prefix$text"
    }
}

fun shout(text: String): String = text.uppercase()

fun Helper.undecorate(text: String): String = text.removePrefix(prefix)
