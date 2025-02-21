#!/bin/bash

HEADERPATH="Sources/TapyrusWallet/TapyrusWalletFFI.h"
MODMAPPATH="Sources/TapyrusWallet/TapyrusWalletFFI.modulemap"
TARGETDIR="../tapyrus-wallet-ffi/target"
OUTDIR="."
RELDIR="release-smaller"
NAME="tapyrus_wallet_ffi"
STATIC_LIB_NAME="lib${NAME}.a"
NEW_HEADER_DIR="../tapyrus-wallet-ffi/target/include"

# set required rust version and install component and targets
rustup default 1.84.1
rustup component add rust-src
rustup target add aarch64-apple-ios      # iOS arm64
rustup target add x86_64-apple-ios       # iOS x86_64
rustup target add aarch64-apple-ios-sim  # simulator mac M1
rustup target add aarch64-apple-darwin   # mac M1
rustup target add x86_64-apple-darwin    # mac x86_64

cd ../tapyrus-wallet-ffi/ || exit

# build tapyrus-wallet-ffi rust lib for apple targets
cargo build --package tapyrus-wallet-ffi --profile release-smaller --target x86_64-apple-darwin
cargo build --package tapyrus-wallet-ffi --profile release-smaller --target aarch64-apple-darwin
cargo build --package tapyrus-wallet-ffi --profile release-smaller --target x86_64-apple-ios
cargo build --package tapyrus-wallet-ffi --profile release-smaller --target aarch64-apple-ios
cargo build --package tapyrus-wallet-ffi --profile release-smaller --target aarch64-apple-ios-sim

# build tapyrus-wallet-ffi Swift bindings and put in TapyrusWallsetSwift Sources
cargo run --bin uniffi-bindgen generate --library ./target/aarch64-apple-ios/release-smaller/libtapyrus_wallet_ffi.dylib --language swift --out-dir ../TapyrusWalletSwift/Sources/TapyrusWallet --no-format

# combine tapyrus-wallet-ffi static libs for aarch64 and x86_64 targets via lipo tool
mkdir -p target/lipo-ios-sim/release-smaller
lipo target/aarch64-apple-ios-sim/release-smaller/libtapyrus_wallet_ffi.a target/x86_64-apple-ios/release-smaller/libtapyrus_wallet_ffi.a -create -output target/lipo-ios-sim/release-smaller/libtapyrus_wallet_ffi.a
mkdir -p target/lipo-macos/release-smaller
lipo target/aarch64-apple-darwin/release-smaller/libtapyrus_wallet_ffi.a target/x86_64-apple-darwin/release-smaller/libtapyrus_wallet_ffi.a -create -output target/lipo-macos/release-smaller/libtapyrus_wallet_ffi.a

cd ../TapyrusWalletSwift/ || exit

# move tapyrus-wallet-ffi static lib header files to temporary directory
mkdir -p "${NEW_HEADER_DIR}"
mv "${HEADERPATH}" "${NEW_HEADER_DIR}"
mv "${MODMAPPATH}" "${NEW_HEADER_DIR}/module.modulemap"
echo -e "\n" >> "${NEW_HEADER_DIR}/module.modulemap"

# remove old xcframework directory
rm -rf "${OUTDIR}/${NAME}.xcframework"

# create new xcframework directory from tapyrus-wallet-ffi static libs and headers
xcodebuild -create-xcframework \
    -library "${TARGETDIR}/lipo-macos/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/aarch64-apple-ios/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -library "${TARGETDIR}/lipo-ios-sim/${RELDIR}/${STATIC_LIB_NAME}" \
    -headers "${NEW_HEADER_DIR}" \
    -output "${OUTDIR}/${NAME}.xcframework"