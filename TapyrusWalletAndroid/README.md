# Tapyrus Wallet Android

Tapyrus Wallet Android is a Kotlin library for interacting with the Tapyrus blockchain on Android devices. This library provides a comprehensive set of tools for creating and managing Tapyrus wallets, handling transactions, and interacting with the Tapyrus network.

## Features

- Create and manage HD wallets for Tapyrus
- Generate and manage wallet addresses
- View wallet balances (TPC and colored coins)
- Sign and verify messages
- Create and broadcast transactions
- Synchronize wallet with the blockchain
- Support for colored coins
- Pay-to-Contract Protocol support

## Requirements

- Android SDK 24 or higher
- Kotlin 2.1.10 or higher
- JDK 17 or higher

## Installation

### Step 1: Configure GitHub Packages Repository

Add the GitHub Packages repository to your project's `settings.gradle.kts` file:

```kotlin
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
        maven {
            url = uri("https://maven.pkg.github.com/chaintope/rust-tapyrus-wallet-ffi")
            credentials {
                username = providers.gradleProperty("gpr.user").getOrElse("")
                password = providers.gradleProperty("gpr.key").getOrElse("")
            }
        }
    }
}
```

### Step 2: Add Authentication Credentials

GitHub Packages requires authentication even for public packages. Add your GitHub username and a Personal Access Token (PAT) with the `read:packages` scope to your Gradle properties.

You can add these credentials in one of two ways:

#### Option 1: Project-level gradle.properties (not recommended for sensitive information)

```properties
# In project's gradle.properties
gpr.user=YOUR_GITHUB_USERNAME
gpr.key=YOUR_GITHUB_TOKEN
```

#### Option 2: Global gradle.properties (recommended)

```properties
# In ~/.gradle/gradle.properties
gpr.user=YOUR_GITHUB_USERNAME
gpr.key=YOUR_GITHUB_TOKEN
```

### Step 3: Add the Dependency

Add the Tapyrus Wallet Android dependency to your app's `build.gradle.kts` file:

```kotlin
dependencies {
    implementation("com.chaintope.tapyrus.wallet:tapyrus-wallet-android:0.1.5")
    // Other dependencies...
}
```

## Basic Usage

### Initialize Configuration

```kotlin
// Create a wallet configuration
val config = Config(
    networkMode = Network.PROD, // or Network.DEV for development
    networkId = 1939510133u,    // Tapyrus network ID
    genesisHash = "...",        // Genesis block hash
    esploraUrl = "https://esplora.example.com", // Esplora API URL
    dbFilePath = "path/to/wallet.db" // Path to store wallet database
)
```

### Create or Load a Wallet

```kotlin
// Generate a new master key
val masterKey = generateMasterKey(Network.PROD)

// Create a wallet with the master key
val config = Config(
    networkMode = Network.PROD,
    networkId = 1939510133u,
    genesisHash = "...",
    esploraUrl = "https://esplora.example.com",
    masterKey = masterKey,
    dbFilePath = "path/to/wallet.db"
)

// Initialize the wallet
val wallet = HdWallet(config)
```

### Synchronize with the Blockchain

```kotlin
// Sync the wallet with the blockchain
wallet.sync()

// For a full sync from the genesis block
wallet.fullSync()
```

### Generate a New Address

```kotlin
// Generate a new address for TPC
val result = wallet.getNewAddress()
val address = result.address
val publicKey = result.publicKey

// Generate a new address for a colored coin
val colorId = "c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46"
val coloredResult = wallet.getNewAddress(colorId)
```

### Check Balance

```kotlin
// Get TPC balance
val tpcBalance = wallet.balance()

// Get colored coin balance
val colorId = "c3ec2fd806701a3f55808cbec3922c38dafaa3070c48c803e9043ee3642c660b46"
val coloredBalance = wallet.balance(colorId)
```

### Transfer Funds

```kotlin
// Create transfer parameters
val params = listOf(
    TransferParams(
        amount = 1000000u,
        toAddress = "recipient-address"
    )
)

// Execute the transfer
val txid = wallet.transfer(params, emptyList())
```

### Sign and Verify Messages

```kotlin
// Sign a message
val signature = wallet.signMessage(publicKey, "Hello, Tapyrus!")

// Verify a signature
val isValid = wallet.verifySign(publicKey, "Hello, Tapyrus!", signature)
```

## Example Project

An example Android application demonstrating the usage of this library is available in the [example](./example) directory. The example includes a Docker Compose setup for running a local Tapyrus node and Electrs server for testing.

## Documentation

For more detailed documentation, please refer to the [API documentation ZIP](https://github.com/chaintope/rust-tapyrus-wallet-ffi/releases/download/v0.1.5/tapyrus-wallet-android-docs-0.1.5.zip).

## Release Procedure

To release a new version, follow these steps:

1. Update the version in `tapyrus-wallet-ffi/Cargo.toml`
2. Commit and push the changes
   ```bash
   git add .
   git commit -m "Bump version to X.Y.Z"
   git push
   ```
3. Create and push a tag (CI will automatically update Android/C# versions)
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

The CI workflows will:
- Verify that the tag version matches `Cargo.toml`
- Automatically update `gradle.properties` (Android) and `.csproj` (C#) with the tag version
- Build and publish packages to GitHub Packages / GitHub Releases
- Mark releases with `-` in the version (e.g., `v0.1.3-beta.1`) as prerelease