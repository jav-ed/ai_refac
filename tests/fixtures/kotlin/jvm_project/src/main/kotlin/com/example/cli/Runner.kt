package com.example.cli

import com.example.app.Greeter
import com.example.util.Helper
import com.example.util.undecorate

class Runner {
    fun run(): String = Greeter(Helper("r:")).greet("runner")

    fun clean(text: String): String = Helper("r:").undecorate(text)
}
