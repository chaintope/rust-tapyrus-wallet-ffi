plugins {
    id("com.android.library").version("8.3.1").apply(false)
    id("org.jetbrains.kotlin.android").version("2.1.10").apply(false)
    id("org.gradle.maven-publish")
    id("org.gradle.signing")
    id("org.jetbrains.dokka").version("2.0.0").apply(false)
    id("org.jetbrains.dokka-javadoc").version("2.0.0").apply(false)
}

publishing {
    repositories {
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/chaintope/rust-tapyrus-wallet-ffi")
            credentials {
                username = System.getenv("GITHUB_ACTOR")
                password = System.getenv("GITHUB_TOKEN")
            }
        }
    }
}