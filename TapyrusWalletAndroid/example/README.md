# Tapyrus Wallet Android Example

This is an example Android application that demonstrates how to use the Tapyrus Wallet library to interact with the Tapyrus blockchain.

## Overview

This example application showcases the functionality of the Tapyrus Wallet Android library, which is a wallet library for the Tapyrus blockchain. The application is configured to connect to the Tapyrus blockchain testnet network.

Key features of the example app:
- Generate and manage wallet addresses
- View wallet balance
- Sync wallet with the blockchain
- Send TPC (Tapyrus coins) to other addresses

## Testnet Configuration

This example is configured to connect to the Tapyrus testnet network. You can explore the testnet using the explorer at:
[https://testnet-explorer.tapyrus.dev.chaintope.com](https://testnet-explorer.tapyrus.dev.chaintope.com)

## Setup

### Prerequisites

- Docker and Docker Compose
- Android Studio
- Android device or emulator
- Sufficient disk space for blockchain data

### Step 1: Start the Tapyrus Node and Electrs

The example includes a `docker-compose.yml` file that sets up both the Tapyrus node (tapyrusd) and the Electrum server (electrs/esplora-tapyrus).

1. Navigate to the example directory:
   ```
   cd example
   ```

2. Start the Docker containers:
   ```
   docker compose up -d
   ```

3. Wait for the Tapyrus node to synchronize with the testnet network. You can check the synchronization status by viewing the logs:
   ```
   docker compose logs -f tapyrusd
   ```

   The synchronization process may take some time depending on your internet connection and system performance.

### Step 2: GitHub Packages Authentication

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

### Step 3: Build and Run the App

Once you've set up the GitHub authentication and started the Docker services, you can build and run the app:

1. Open the project in Android Studio:
   ```
   # Open Android Studio and select "Open an existing Android Studio project"
   # Navigate to the TapyrusWalletAndroid/example directory
   ```

2. Build the project:
   ```
   ./gradlew assembleDebug
   ```

3. Install and run the app on your device or emulator:
   ```
   ./gradlew installDebug
   ```

   Or simply run the app directly from Android Studio by clicking the "Run" button.

## Application Usage

- **Generate Address**: Tap the "Generate & Copy Address" button to create a new Tapyrus address and copy it to the clipboard.
- **Sync Wallet**: Tap the "Sync Wallet" button to synchronize your wallet with the blockchain and update your balance.
- **Send TPC**: Use the transfer dialog to send TPC to another address by entering the recipient's address and the amount to send.

## Stopping the Services

When you're done using the example, you can stop the Docker containers:

```
docker-compose down
```

## Data Persistence

The blockchain data and wallet database are stored in the following locations:
- Blockchain data: `./data` directory
- Electrs data: `./electrs` directory
- Wallet database: In the app's internal storage on the Android device or emulator

## Note

This example is for demonstration purposes and connects to the testnet. For production applications, you would need to configure the wallet to connect to the Tapyrus mainnet and implement additional security measures.
