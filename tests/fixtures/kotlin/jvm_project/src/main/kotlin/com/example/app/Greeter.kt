package com.example.app

import com.example.util.DEFAULT_GREETING
import com.example.util.Helper

class Greeter(private val helper: Helper) {
    fun greet(name: String): String = helper.decorate("$DEFAULT_GREETING $name")
}
