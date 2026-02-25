# Tapyrus Wallet C# WPF Example

A WPF desktop wallet application for the Tapyrus blockchain, equivalent to the iOS (SwiftUI) and Android (Jetpack Compose) example apps.

## Features

- View TPC balance
- Generate new addresses and copy to clipboard
- Sync wallet with the blockchain (full sync)
- Send TPC to other addresses
- Switch between Esplora and Electrum connections
- Persistent master key and connection settings

## Prerequisites

- .NET 8.0 SDK (Windows)
- Docker and Docker Compose (for local testnet)

## Setup

### 1. Download DLLs

Download `tapyrus-wallet-csharp-win-x64-v0.1.5.zip` from:
https://github.com/chaintope/rust-tapyrus-wallet-ffi/releases/tag/v0.1.5

Extract and place the DLLs in `lib/` directory:

```
example/
├── lib/
│   ├── TapyrusWalletCSharp.dll
│   └── tapyrus_wallet_ffi.dll
├── TapyrusWalletExample.csproj
└── ...
```

### 2. Start Local Testnet

```bash
docker compose up -d
```

This starts:
- **tapyrusd**: Tapyrus node (RPC on port 2377)
- **esplora**: Block explorer with Esplora HTTP API (port 3001) and Electrum RPC (port 50001)

### 3. Build and Run

```bash
dotnet build
dotnet run
```

## Usage

1. The wallet initializes automatically on startup and performs an initial sync.
2. Click **"Generate & Copy Address"** to create a new address (copied to clipboard).
3. Send TPC to the generated address (e.g., from a faucet or another wallet).
4. Click **"Sync Wallet"** to update the balance.
5. Click **"Send"** to transfer TPC to another address.
6. Click **"Connection Settings"** to switch between Esplora and Electrum connections.

## Connection Settings

Default connections:
- **Esplora**: `http://localhost:3001`
- **Electrum**: `localhost:50001`

Settings are persisted in `settings.json` in the application directory.

## File Storage

The following files are stored in the application's output directory:
- `master_key`: HD wallet master key (BIP32 extended key)
- `tapyrus_wallet.db`: SQLite wallet database
- `settings.json`: Connection settings

## Stopping the Testnet

```bash
docker compose down
```

To also remove the blockchain data:

```bash
docker compose down -v
rm -rf data/ electrs/
```
