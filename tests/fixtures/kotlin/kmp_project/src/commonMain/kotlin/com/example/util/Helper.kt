package com.example.util

const val DEFAULT_GREETING = "Hello"

class Helper(private val prefix: String) {
    fun decorate(text: String): String = prefix + text
}

fun shout(text: String): String = text.uppercase()

// Declared here, defined for the JVM in jvmMain/.../Platform.kt.
expect fun platformName(): String
