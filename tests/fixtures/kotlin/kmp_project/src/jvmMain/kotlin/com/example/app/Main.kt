package com.example.app

import com.example.util.Helper
import com.example.util.platformName
import com.example.util.shout

fun main() {
    val helper = Helper("> ")
    val greeter = Greeter(helper)
    println(helper.decorate(shout(greeter.greet(platformName()))))
}
