plugins {
    kotlin("multiplatform") version "2.4.20"
}

repositories {
    mavenCentral()
}

kotlin {
    jvm()
    sourceSets {
        // `library` stands for Compose: classes the project uses and a mirror
        // of its source sets (src/<set>/kotlin) does not hold, as it would not
        // hold the real library.
        commonMain {
            kotlin.srcDir("library")
        }
    }
}
