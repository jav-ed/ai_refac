package com.example.other

import com.example.app.Greeter
import com.example.app.Screen
import com.example.util.DEFAULT_GREETING
import com.example.util.Helper

// Uses Greeter and Helper. Screen (which stays in the package Greeter leaves) and
// DEFAULT_GREETING are imported and never used: a move of Greeter edits this file,
// and an import it was not asked to touch stays.
fun keep(): Any = Greeter(Helper("k:"))
