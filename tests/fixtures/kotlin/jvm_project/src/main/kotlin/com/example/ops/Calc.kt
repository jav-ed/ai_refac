package com.example.ops

fun less(a: Qty, b: Qty): Boolean = a < b

fun has(a: Qty): Boolean = 3 in a

fun missing(a: Qty): Boolean = 3 !in a

fun part(a: Qty): Int = a[1]

fun assign(a: Qty) {
    a[1] = 2
}

fun parts(a: Qty): Int {
    val (x, y) = a
    return x + y
}

fun sum(bag: Bag): Int {
    var total = 0
    for (value in bag) total += value
    return total
}

fun counter(): Int {
    val registry = Registry()
    var n by registry
    n = 3
    return n
}
