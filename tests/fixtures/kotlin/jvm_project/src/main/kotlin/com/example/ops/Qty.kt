package com.example.ops

import kotlin.reflect.KProperty

// Operator functions declared at the top level of a package. Calc.kt uses them
// without an import and without writing their names.

data class Qty(val n: Int)

operator fun Qty.compareTo(other: Qty) = n.compareTo(other.n)
operator fun Qty.contains(value: Int) = n == value
operator fun Qty.get(index: Int) = n / (index + 1)
operator fun Qty.set(index: Int, value: Int) {}
operator fun Qty.component1() = n
operator fun Qty.component2() = n + 1

class Bag
operator fun Bag.iterator(): Iterator<Int> = listOf(1, 2).iterator()

class Registry {
    var stored = 0
}

operator fun Registry.getValue(thisRef: Any?, property: KProperty<*>): Int = stored
operator fun Registry.setValue(thisRef: Any?, property: KProperty<*>, value: Int) {
    stored = value
}
