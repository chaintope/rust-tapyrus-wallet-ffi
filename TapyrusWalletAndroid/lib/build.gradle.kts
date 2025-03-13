import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
import java.io.FileOutputStream
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

// library version is defined in gradle.properties
val libraryVersion: String by project

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("org.gradle.maven-publish")
    id("org.jetbrains.dokka")
    id("org.jetbrains.dokka-javadoc")
}

android {
    namespace = "com.chaintope.tapyrus.wallet"
    compileSdk = 33

    defaultConfig {
        minSdk = 24
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(file("proguard-android-optimize.txt"), file("proguard-rules.pro"))
        }
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
            withJavadocJar()
        }
    }
    
    lint {
        // Disable lint errors that would prevent the build
        abortOnError = false
        // Ignore the NewApi issue with java.lang.ref.Cleaner
        disable += "NewApi"
    }
}

kotlin {
    tasks.withType<KotlinCompile>().configureEach {
        compilerOptions {
            jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
        }
    }
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

// Configure Dokka for HTML documentation
tasks.withType<org.jetbrains.dokka.gradle.DokkaTask>().configureEach {
    outputDirectory.set(file("${buildDir}/dokka/html"))
    
    dokkaSourceSets {
        named("main") {
            moduleName.set("TapyrusWalletAndroid")
            includes.from(listOf("packages.md"))
            
            // Link to Android SDK documentation
            externalDocumentationLink {
                url.set(uri("https://developer.android.com/reference/").toURL())
                packageListUrl.set(uri("https://developer.android.com/reference/package-list").toURL())
            }
        }
    }
}

// Create a packages.md file if it doesn't exist
tasks.register("createPackagesMd") {
    doLast {
        val packagesFile = file("packages.md")
        if (!packagesFile.exists()) {
            packagesFile.writeText("""
                # Module TapyrusWalletAndroid
                
                TapyrusWalletAndroid is a Kotlin library for interacting with the Tapyrus blockchain.
                
                ## Packages
                
                | Name | Description |
                |------|-------------|
                | com.chaintope.tapyrus.wallet | Core wallet functionality |
            """.trimIndent())
        }
    }
}

// Make dokka tasks depend on createPackagesMd
tasks.withType<org.jetbrains.dokka.gradle.DokkaTask>().configureEach {
    dependsOn(tasks.named("createPackagesMd"))
}

// We'll handle zipping the documentation in the GitHub Actions workflow

dependencies {
    implementation("net.java.dev.jna:jna:5.14.0@aar")
    implementation("org.jetbrains.kotlin:kotlin-stdlib-jdk7")
    implementation("androidx.appcompat:appcompat:1.4.0")
    implementation("androidx.core:core-ktx:1.7.0")
    api("org.slf4j:slf4j-api:1.7.30")

    androidTestImplementation("com.github.tony19:logback-android:2.0.0")
    androidTestImplementation("androidx.test.ext:junit:1.1.3")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.4.0")
    androidTestImplementation("org.jetbrains.kotlin:kotlin-test:2.1.10")
    androidTestImplementation("org.jetbrains.kotlin:kotlin-test-junit:2.1.10")
    
    testImplementation("org.jetbrains.kotlin:kotlin-test:2.1.10")
    testImplementation("org.jetbrains.kotlin:kotlin-test-junit:2.1.10")
    testImplementation("junit:junit:4.13.2")
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("maven") {
                groupId = "com.chaintope.tapyrus.wallet"
                artifactId = "tapyrus-wallet-android"
                version = libraryVersion

                from(components["release"])
                
                // Documentation artifact will be handled separately in the GitHub Actions workflow
                pom {
                    name.set("TapyrusWalletAndroid")
                    description.set("Kotlin bindings for Tapyrus Wallet")
                    url.set("https://github.com/chaintope/rust-tapyrus-wallet-ffi")
                    licenses {
                        license {
                            name.set("MIT")
                            url.set("https://github.com/chaintope/rust-tapyrus-wallet-ffi/blob/master/LICENSE")
                        }
                    }
                    developers {
                        developer {
                            id.set("chaintope")
                            name.set("Chaintope Inc.")
                            email.set("info@chaintope.com")
                        }
                    }
                    scm {
                        connection.set("scm:git:github.com/chaintope/rust-tapyrus-wallet-ffi.git")
                        developerConnection.set("scm:git:ssh://github.com/chaintope/rust-tapyrus-wallet-ffi.git")
                        url.set("https://github.com/chaintope/rust-tapyrus-wallet-ffi/tree/master")
                    }
                }
            }
        }

        repositories {
            maven {
                name = "GitHubPackages"
                url = uri("https://maven.pkg.github.com/chaintope/rust-tapyrus-wallet-ffi")
                credentials {
                    username = project.findProperty("gpr.user") as String? ?: System.getenv("GITHUB_ACTOR")
                    password = project.findProperty("gpr.key") as String? ?: System.getenv("GITHUB_TOKEN")
                }
            }
        }
    }
}
