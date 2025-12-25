# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

This is a Foreign Function Interface (FFI) library for the Tapyrus Wallet, written in Rust and exposed to multiple languages through UniFFI. The repository contains:

- **Core Rust library** (`tapyrus-wallet-ffi/`): The main wallet implementation
- **Language bindings**: Kotlin/Android, Swift/iOS, and C# wrappers around the Rust core

## Architecture

### Core Components

The Rust library (`tapyrus-wallet-ffi/src/lib.rs`, ~1770 lines) implements:

- **HdWallet**: Main wallet struct with HD (Hierarchical Deterministic) wallet functionality
- **Config**: Wallet configuration including network mode, genesis hash, Esplora URL, and database path
- **Network types**: Prod and Dev environments for Tapyrus blockchain
- **Transaction management**: Transfer creation, signing, and broadcasting
- **Colored coins support**: Handle both TPC and colored coin balances/transfers
- **Pay-to-Contract Protocol**: P2C address generation and contract management
- **Message signing/verification**: Sign and verify messages with wallet keys

### UniFFI Interface

The API surface is defined in `tapyrus-wallet-ffi/src/wallet.udl` using UniFFI Definition Language:
- All public types, methods, and errors are declared here
- This file drives code generation for all language bindings
- Changes to the API must be made here first, then implement in `lib.rs`

### Dependencies

The project depends on the Tapyrus Development Kit (TDK):
- `tdk_wallet`: Core wallet functionality (from chaintope/tdk fork)
- `tdk_sqlite`: SQLite persistence layer
- `tdk_esplora`: Blockchain synchronization via Esplora API
- UniFFI v0.29.0 for FFI bindings generation

## Building and Testing

### Rust Core Library

```bash
cd tapyrus-wallet-ffi
cargo build --release
cargo test --all
cargo fmt -- --check  # Check formatting
```

For size-optimized builds (used in production):
```bash
cargo build --profile release-smaller
```

### Android Library

Prerequisites: Set `ANDROID_NDK_ROOT` environment variable

```bash
cd TapyrusWalletAndroid
./scripts/build-android.sh  # Builds for arm64-v8a, x86_64, armeabi-v7a
./gradlew build
./scripts/generate-docs.sh  # Generate API documentation
```

The build script:
1. Cross-compiles Rust to Android targets (aarch64, x86_64, armv7)
2. Generates Kotlin bindings via uniffi-bindgen
3. Copies native libraries to `lib/src/main/jniLibs/`

### Swift/iOS Library

```bash
cd TapyrusWalletSwift
./create_xcframework.sh
```

The build script:
1. Cross-compiles Rust to iOS/macOS targets (aarch64-apple-ios, x86_64-apple-darwin, etc.)
2. Generates Swift bindings via uniffi-bindgen
3. Uses `lipo` to create universal binaries for simulator targets
4. Creates XCFramework bundle with `xcodebuild -create-xcframework`

### C# Library

Prerequisites: Install `uniffi-bindgen-cs` (version must match `Cargo.toml` uniffi version):

```bash
cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.7.0+v0.25.0
```

Build steps:
```bash
cd tapyrus-wallet-ffi
cargo build --release
uniffi-bindgen-cs target/release/tapyrus_wallet_ffi.dll --library --out-dir ../TapyrusWalletCSharp/TapyrusWalletCSharp/src/com/chaintope/tapyrus/wallet/

cd ../TapyrusWalletCSharp/TapyrusWalletCSharp
dotnet build -c Release
```

Run tests:
```bash
cd TapyrusWalletCSharp/TapyrusWalletCSharp.Tests
dotnet build
cp ../../tapyrus-wallet-ffi/target/release/tapyrus_wallet_ffi.dll ./bin/Debug/net8.0/
dotnet test
```

## Development Workflow

### Making API Changes

1. Update `tapyrus-wallet-ffi/src/wallet.udl` with new types/methods
2. Implement corresponding Rust code in `tapyrus-wallet-ffi/src/lib.rs`
3. Run `cargo build` to trigger UniFFI scaffolding generation via `build.rs`
4. Rebuild platform-specific bindings (Android/Swift/C#)
5. Update platform-specific example projects to demonstrate new features

### Testing

- Rust tests live in `tapyrus-wallet-ffi/tests/`
- Run with `cargo test --all` (CI uses this command)
- Each platform has its own test suite using the generated bindings

### Platform-Specific Configuration

- **Android**: `uniffi-android.toml` - Kotlin package configuration
- **All platforms**: `uniffi.toml` - General UniFFI settings
- **Rust profiles**: `release-smaller` profile optimizes for binary size (LTO, opt-level='z')

## Key Concepts

### Wallet Initialization

Wallets require:
- Network mode (Prod/Dev) and network ID
- Genesis block hash (validates blockchain connection)
- Esplora URL (blockchain data source)
- Master key (BIP32 extended key) or path to load from
- Database file path (SQLite persistence)

### Colored Coins

The Tapyrus blockchain supports colored coins (custom tokens). Many methods accept an optional `color_id` parameter:
- `null`/`None` = TPC (native Tapyrus coin)
- `Some(color_id)` = specific colored coin

### Synchronization

- `sync()`: Incremental sync from last checkpoint
- `full_sync()`: Full sync from genesis block
- Uses Esplora API with configurable parallel requests (default: 1) and stop gap (25 addresses)

### Pay-to-Contract Protocol

Special protocol where payments commit to a contract:
1. Generate base public key
2. Calculate P2C address from public key + contract
3. Store contract with `store_contract()`
4. Mark as payable/non-payable with `update_contract()`

## Troubleshooting

### uniffi-bindgen-cs version mismatch
Check that the `uniffi-bindgen-cs` tag matches the UniFFI version in `Cargo.toml`. The tag format is `v{cs-version}+v{uniffi-version}`.

### Android NDK errors
Ensure `ANDROID_NDK_ROOT` points to your NDK installation and includes the `toolchains/llvm/prebuilt/linux-x86_64/bin` directory.

### XCFramework build failures
Verify all required Rust targets are installed:
```bash
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim aarch64-apple-darwin x86_64-apple-darwin
```

### TDK dependency updates
The project depends on the `master` branch of `chaintope/tdk`. If updating, ensure all TDK crates are updated together (wallet, sqlite, esplora, chain, testenv).
