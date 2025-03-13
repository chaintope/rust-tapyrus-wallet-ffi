pluginManagement {
    repositories {
        google {
            content {
                includeGroupByRegex("com\\.android.*")
                includeGroupByRegex("com\\.google.*")
                includeGroupByRegex("androidx.*")
            }
        }
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven {
            url = uri("https://maven.pkg.github.com/chaintope/rust-tapyrus-wallet-ffi")
            credentials {
                username = providers.gradleProperty("pgr.user").getOrElse("")
                password = providers.gradleProperty("pgr.key").getOrElse("")
            }
        }
    }
}

rootProject.name = "example"
include(":app")
