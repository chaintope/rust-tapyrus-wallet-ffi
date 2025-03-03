# Tapyrus Wallet Swift Example

This is an example iOS application that demonstrates how to use the [Tapyrus Wallet Swift Package](https://github.com/chaintope/TapyrusWalletSwift) to interact with the Tapyrus blockchain.

## Overview

This example application showcases the functionality of the Tapyrus Wallet Swift Package, which is a wallet library for the Tapyrus blockchain. The application is configured to connect to the Tapyrus blockchain testnet network.

Key features of the example app:
- Generate and manage wallet addresses
- View wallet balance
- Sync wallet with the blockchain
- Send TPC (Tapyrus coins) to other addresses

## Testnet Configuration

This example is configured to connect to the Tapyrus testnet network. You can explore the testnet using the explorer at:
[https://testnet-explorer.tapyrus.dev.chaintope.com](https://testnet-explorer.tapyrus.dev.chaintope.com)

## Running the Example

### Prerequisites

- Docker and Docker Compose
- Xcode
- macOS with sufficient disk space for blockchain data

### Step 1: Start the Tapyrus Node and Electrs

The example includes a `docker-compose.yml` file that sets up both the Tapyrus node (tapyrusd) and the Electrum server (electrs/esplora-tapyrus).

1. Navigate to the example directory:
   ```
   cd example
   ```

2. Start the Docker containers:
   ```
   docker-compose up -d
   ```

3. Wait for the Tapyrus node to synchronize with the testnet network. You can check the synchronization status by viewing the logs:
   ```
   docker-compose logs -f tapyrusd
   ```

   The synchronization process may take some time depending on your internet connection and system performance.

### Step 2: Build and Run the Example App

1. Open the example project in Xcode:
   ```
   open example.xcodeproj
   ```

2. Select a simulator or connected iOS device as the build target.

3. Build and run the application by clicking the play button or pressing `Cmd+R`.

4. The app will initialize a wallet, connect to the local Tapyrus node through the Electrum server, and allow you to interact with the Tapyrus blockchain.

## Application Usage

- **Generate Address**: Tap the "Generate & Copy Address" button to create a new Tapyrus address and copy it to the clipboard.
- **Sync Wallet**: Tap the "Sync Wallet" button to synchronize your wallet with the blockchain and update your balance.
- **Send TPC**: Tap the "Send" button to transfer TPC to another address. You'll need to enter the recipient's address and the amount to send.

## Stopping the Services

When you're done using the example, you can stop the Docker containers:

```
docker-compose down
```

## Data Persistence

The blockchain data and wallet database are stored in the following locations:
- Blockchain data: `./data` directory
- Electrs data: `./electrs` directory
- Wallet database: In the app's Documents directory on the iOS device or simulator

## Note

This example is for demonstration purposes and connects to the testnet. For production applications, you would need to configure the wallet to connect to the Tapyrus mainnet and implement additional security measures.
