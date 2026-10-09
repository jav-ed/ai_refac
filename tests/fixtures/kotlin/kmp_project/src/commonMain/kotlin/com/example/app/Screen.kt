package com.example.app

import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import com.example.util.Helper
import com.example.util.shout

// The Greeter of the same package needs no import until one of them moves.
@Composable
fun Screen(helper: Helper): Modifier {
    // `by` reaches getValue and setValue through their imports alone.
    var clicks by remember { mutableStateOf(0) }
    clicks += 1
    LaunchedEffect(helper) { println(shout(Greeter(helper).greet("screen $clicks"))) }
    return Modifier.padding(8)
}
