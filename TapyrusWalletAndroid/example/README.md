# Tapyrus Wallet Android Example

This is an example Android application that demonstrates how to use the Tapyrus Wallet library.

## Setup

### GitHub Packages Authentication

This project uses a library from GitHub Packages, which requires authentication even for public packages. Follow these steps to set up authentication:

1. Create a GitHub Personal Access Token (PAT) with the `read:packages` scope
2. Add your GitHub username and token to your Gradle properties file:
   - Option 1: Project-level gradle.properties (not recommended for sensitive information)
     ```
     # In project's gradle.properties
     gpr.user=YOUR_GITHUB_USERNAME
     gpr.key=YOUR_GITHUB_TOKEN
     ```
   - Option 2: Global gradle.properties (recommended)
     ```
     # In ~/.gradle/gradle.properties
     gpr.user=YOUR_GITHUB_USERNAME
     gpr.key=YOUR_GITHUB_TOKEN
     ```

### Building the App

Once you've set up the GitHub authentication, you can build and run the app:

```bash
./gradlew assembleDebug
```

Or open the project in Android Studio and run it from there.

## Features

This example app demonstrates:

- Creating a Tapyrus wallet configuration
- Generating a master key
- Creating an HDWallet instance
- Generating a new address using the wallet

## License

[Add license information here]
