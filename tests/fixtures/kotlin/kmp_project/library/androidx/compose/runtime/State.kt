package androidx.compose.runtime

import kotlin.reflect.KProperty

class MutableState<T>(var value: T)

fun <T> mutableStateOf(value: T) = MutableState(value)

fun <T> remember(calculation: () -> T): T = calculation()

// Used only through `by`: nothing in the code that uses them names them.
operator fun <T> MutableState<T>.getValue(thisObj: Any?, property: KProperty<*>): T = value

operator fun <T> MutableState<T>.setValue(thisObj: Any?, property: KProperty<*>, new: T) {
    value = new
}
