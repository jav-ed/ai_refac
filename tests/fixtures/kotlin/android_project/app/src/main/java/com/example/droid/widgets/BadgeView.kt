package com.example.droid.widgets

import android.content.Context
import android.util.AttributeSet
import android.widget.TextView

class BadgeView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
) : TextView(context, attrs) {
    var count: Int = 0
        set(value) {
            field = value
            text = value.toString()
        }
}
