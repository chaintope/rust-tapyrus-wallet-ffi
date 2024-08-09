# Tapyrus Wallet C Sharp Library

This is WIP.

# How to build

## 1. Build /tapyrus-wallet-ffi

Setup Rustc env on Windows env and build /tapyrus-wallet-ffi like below.

     $ cd tapyrus-wallet-ffi
     $ cargo build --release

## 2. Generate CSharp binding file

First, You need to install [uniffi-bindgen-cs](https://github.com/NordSecurity/uniffi-bindgen-cs).

     $ cargo install uniffi-bindgen-cs --git https://github.com/NordSecurity/uniffi-bindgen-cs --tag v0.7.0+v0.25.0

Check the later half of the tag is same with uniffi version in `tapyrus-wallet-ffi/Cargo.toml`. 

Then, generate binding file below command.

     $ cd tapyrus-wallet-ffi
     $ uniffi-bindgen-cs target/release/tapyrus_wallet_ffi.dll --library --out-dir ..\TapyrusWalletCSharp\TapyrusWalletCSharp\src\com\chaintope\tapyrus\wallet\

Now you can find the binding file at `TapyrusWalletCSharp/src/com/chaintope/tapyrus/wallet/wallet.cs`

## 3. Build C# binding

Build the C# binding generated previous step like below.

     $ cd TapyrusWalletCSharp\TapyrusWalletCSharp
     $ dotnet build -c Release

You need to put in two dll files to your project.

* `TapyrusWalletCSharp\TapyrusWalletCSharp\bin\Debug\net6.0\TapyrusWalletCSharp.dll`
* `tapyrus-wallet-ffi\target\release\tapyrus_wallet_ffi.dll`

Note: `tapyrus_wallet_ffi.dll` cannot be added as a reference directly in the C# project. This DLL should be placed in a location accessible to the application, such as the system directory or the same directory as the executable file.

# Run unit tests

Before you need to do the How to build process.

Then

     $ cd TapyrusWalletCSharp\TapyrusWalletCSharp.Tests
     $ dotnet build
     $ cp ..\..\tapyrus-wallet-ffi\target\release\tapyrus_wallet_ffi.dll .\bin\Debug\net8.0\
     $ dotnet test

# Run TapyrusWalletExample

     $ cd TapyrusWalletCSharp\TapyrusWalletExample
     $ dotnet build
     $ cp ..\..\tapyrus-wallet-ffi\target\release\tapyrus_wallet_ffi.dll .\bin\Debug\net8.0\
     $ dotnet run