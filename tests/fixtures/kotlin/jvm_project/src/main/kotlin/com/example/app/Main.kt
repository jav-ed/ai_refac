package com.example.app

import com.example.util.Helper
import com.example.util.shout

fun main() {
    val helper = Helper("> ")
    // Same package as Greeter: no import is written for it.
    val greeter = Greeter(helper)
    println(shout(greeter.greet("world")))
    println(helper.counter)
}
