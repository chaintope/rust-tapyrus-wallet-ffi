#!/bin/bash

if [ -z "$ANDROID_NDK_ROOT" ]; then
    echo "Error: ANDROID_NDK_ROOT is not defined in your environment"
    exit 1
fi

PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH"
CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=24"
AR="llvm-ar"
LIB_NAME="libtapyrus_wallet_ffi.so"
COMPILATION_TARGET_ARM64_V8A="aarch64-linux-android"
COMPILATION_TARGET_X86_64="x86_64-linux-android"
COMPILATION_TARGET_ARMEABI_V7A="armv7-linux-androideabi"
RESOURCE_DIR_ARM64_V8A="arm64-v8a"
RESOURCE_DIR_X86_64="x86_64"
RESOURCE_DIR_ARMEABI_V7A="armeabi-v7a"

# Move to the Rust library directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
RUST_LIB_DIR="$REPO_ROOT/tapyrus-wallet-ffi"

cd "$RUST_LIB_DIR" || exit
echo "Changed directory to: $(pwd)"

# Rustのターゲットが既に追加されている場合はスキップ
if ! rustup target list --installed | grep -q "$COMPILATION_TARGET_ARM64_V8A"; then
    rustup target add $COMPILATION_TARGET_ARM64_V8A $COMPILATION_TARGET_ARMEABI_V7A $COMPILATION_TARGET_X86_64
fi

# Build the binaries
# The CC and CARGO_TARGET_<TARGET>_LINUX_ANDROID_LINKER environment variables must be declared on the same line as the cargo build command
CC="aarch64-linux-android24-clang" CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android24-clang" cargo build --profile release-smaller --target $COMPILATION_TARGET_ARM64_V8A
CC="x86_64-linux-android24-clang" CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="x86_64-linux-android24-clang" cargo build --profile release-smaller --target $COMPILATION_TARGET_X86_64
CC="armv7a-linux-androideabi24-clang" CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER="armv7a-linux-androideabi24-clang" cargo build --profile release-smaller --target $COMPILATION_TARGET_ARMEABI_V7A

# Copy the binaries to their respective resource directories
mkdir -p ../TapyrusWalletAndroid/lib/src/main/jniLibs/$RESOURCE_DIR_ARM64_V8A/
mkdir -p ../TapyrusWalletAndroid/lib/src/main/jniLibs/$RESOURCE_DIR_ARMEABI_V7A/
mkdir -p ../TapyrusWalletAndroid/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/
cp ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME ../TapyrusWalletAndroid/lib/src/main/jniLibs/$RESOURCE_DIR_ARM64_V8A/
cp ./target/$COMPILATION_TARGET_ARMEABI_V7A/release-smaller/$LIB_NAME ../TapyrusWalletAndroid/lib/src/main/jniLibs/$RESOURCE_DIR_ARMEABI_V7A/
cp ./target/$COMPILATION_TARGET_X86_64/release-smaller/$LIB_NAME ../TapyrusWalletAndroid/lib/src/main/jniLibs/$RESOURCE_DIR_X86_64/

# Generate Kotlin bindings using uniffi-bindgen
# First, create the directory structure for the package
mkdir -p ../TapyrusWalletAndroid/lib/src/main/kotlin/com/chaintope/tapyrus/wallet/

# Generate the Kotlin bindings
if [ -f "$CARGO_HOME/bin/uniffi-bindgen" ]; then
    echo "Using installed uniffi-bindgen"
    $CARGO_HOME/bin/uniffi-bindgen generate --library ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME --language kotlin --out-dir ../TapyrusWalletAndroid/lib/src/main/kotlin/ --no-format
elif [ -f "./target/debug/uniffi-bindgen" ]; then
    echo "Using locally built uniffi-bindgen"
    ./target/debug/uniffi-bindgen generate --library ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME --language kotlin --out-dir ../TapyrusWalletAndroid/lib/src/main/kotlin/ --no-format
else
    echo "Using cargo run for uniffi-bindgen"
    cargo run --bin uniffi-bindgen generate --library ./target/$COMPILATION_TARGET_ARM64_V8A/release-smaller/$LIB_NAME --language kotlin --out-dir ../TapyrusWalletAndroid/lib/src/main/kotlin/ --no-format
fi

# Verify the generated files
echo "Generated Kotlin bindings:"
ls -la ../TapyrusWalletAndroid/lib/src/main/kotlin/com/chaintope/tapyrus/wallet/
