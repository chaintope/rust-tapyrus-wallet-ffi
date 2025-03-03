# Tapyrus Wallet Swift Library

This repository contains the configuration for generating binary frameworks used by the [TapyrusWallet Swift Package](https://github.com/chaintope/TapyrusWalletSwift). The Swift Package provides a convenient way to interact with the Tapyrus blockchain in Swift applications.

## Overview

The TapyrusWalletSwift package relies on binary frameworks that are generated from this repository. These binaries provide the core functionality for interacting with the Tapyrus blockchain from Swift applications.

## Usage

For examples of how to use the TapyrusWallet Swift Package in your application, please refer to the sample project in the [example](./example) directory. The sample project demonstrates common operations such as:

- Creating and managing wallets
- Handling transactions
- Working with the Tapyrus blockchain

## Building the Binary Frameworks

The `create_xcframework.sh` script in this repository is used to generate the XCFramework binaries required by the Swift Package.