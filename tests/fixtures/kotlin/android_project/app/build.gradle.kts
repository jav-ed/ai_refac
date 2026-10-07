plugins {
    id("com.android.application")
}

android {
    namespace = "com.example.droid"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.example.droid"
        minSdk = 24
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"
    }

    buildFeatures {
        viewBinding = true
    }
}
