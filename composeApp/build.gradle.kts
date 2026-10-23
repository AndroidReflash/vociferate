import org.gradle.kotlin.dsl.implementation
import org.jetbrains.compose.desktop.application.dsl.TargetFormat
import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    alias(libs.plugins.kotlinMultiplatform)
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.composeMultiplatform)
    alias(libs.plugins.composeCompiler)
    alias(libs.plugins.composeHotReload)
    id("com.github.johnrengelman.shadow") version "8.1.1"
    // id("com.gradleup.shadow") version "8.3.6"
}

kotlin {
    androidTarget {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
        }
    }
    
    jvm()

    sourceSets {
        androidMain.dependencies {
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.activity.compose)

            implementation(libs.koin.android)
            implementation("net.java.dev.jna:jna:5.14.0@aar")
        }
        commonMain.dependencies {
            implementation(libs.compose.runtime)
            implementation(libs.compose.foundation)
            implementation(libs.compose.material3)
            implementation(libs.compose.ui)
            implementation(libs.compose.components.resources)
            implementation(libs.compose.uiToolingPreview)
            implementation(libs.androidx.lifecycle.viewmodelCompose)
            implementation(libs.androidx.lifecycle.runtimeCompose)
            implementation(libs.compose.material.icons.extended)

            implementation(libs.kotlinx.coroutines.core)
            implementation(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)

            implementation(libs.androidx.navigation.composee)


        }
        commonTest.dependencies {
            implementation(libs.kotlin.test)
        }
        jvmMain.dependencies {
            implementation(compose.desktop.currentOs)
            implementation(libs.kotlinx.coroutinesSwing)

            implementation("net.java.dev.jna:jna:5.14.0")

            implementation("org.jetbrains.skiko:skiko-awt-runtime-windows-x64:0.9.3")
            implementation(compose.desktop.windows_x64)
        }
    }
}

android {
    namespace = "app.vociferate"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "app.vociferate"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = 1
        versionName = "1.0"
    }
    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
}

dependencies {
    debugImplementation(libs.compose.uiTooling)
}

compose.desktop {
    application {
        mainClass = "app.vociferate.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb, TargetFormat.Exe)
            packageName = "app.vociferate"
            packageVersion = "1.0.0"

            windows {
                iconFile.set(project.file("src/jvmMain/resources/icon.ico"))

                menu = true
            }
        }
    }
    tasks.withType<JavaExec>().configureEach {
        // линукс
        systemProperty("jna.library.path", file("./src/main/rust/target/release").absolutePath)
        // винда
        // systemProperty("jna.library.path", file("./src/main/rust/target/x86_64-pc-windows-gnu").absolutePath)
    }

    tasks.register<Jar>("packageWindowsFatJar") {
        group = "package"
        description = "Assembles Fat JAR for Windows"

        archiveClassifier.set("windows")

        // Игнорируем дубликаты манифестов и модулей из разных JAR
        duplicatesStrategy = DuplicatesStrategy.EXCLUDE

        val jvmTarget = kotlin.targets.getByName("jvm")
        val mainCompilation = jvmTarget.compilations.getByName("main")

        from(mainCompilation.output.classesDirs)
        from(mainCompilation.output.resourcesDir)

        dependsOn(mainCompilation.runtimeDependencyFiles)
        from({
            mainCompilation.runtimeDependencyFiles
                ?.filter { it.name.endsWith(".jar") }
                ?.map { zipTree(it) }
        }) {
            exclude("META-INF/*.SF", "META-INF/*.DSA", "META-INF/*.RSA")
        }

        manifest {
            attributes["Main-Class"] = "app.vociferate.MainKt"
        }
    }
}
