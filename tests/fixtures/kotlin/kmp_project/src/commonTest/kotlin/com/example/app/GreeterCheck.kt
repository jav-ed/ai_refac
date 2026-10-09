package com.example.app

import com.example.util.Helper

fun greeterSays(): String {
    val helper = Helper("> ")
    return helper.decorate(Greeter(helper).greet("test"))
}
