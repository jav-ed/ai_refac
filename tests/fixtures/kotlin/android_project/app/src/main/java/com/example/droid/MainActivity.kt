package com.example.droid

import android.app.Activity
import android.os.Bundle
import com.example.droid.databinding.ActivityMainBinding
import com.example.droid.widgets.BadgeView

class MainActivity : Activity() {
    private lateinit var binding: ActivityMainBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)
        binding.titleText.setText(R.string.app_name)
        val badge: BadgeView = binding.badge
        badge.count = 3
        findViewById<BadgeView>(R.id.badge).count = 4
    }
}
