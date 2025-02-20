#!/bin/bash
set -e

# 既存の XCFramework が存在する場合は削除
OUTPUT_DIR="TapyrusWallet.xcframework"
if [ -d "$OUTPUT_DIR" ]; then
  echo "Removing previous XCFramework at $OUTPUT_DIR"
  rm -rf "$OUTPUT_DIR"
fi

# Rust ツールチェーン用のターゲットを追加（シミュレーターは "sim" 表記）
echo "Adding iOS targets for Rust..."
rustup target add aarch64-apple-ios aarch64-apple-ios-sim

# 固定バージョンの設定
echo "Using fixed version: 1.0.0"
FIXED_VERSION="1.0.0"

# ディレクトリ定義
GENERATED_DIR="build-xcframework/Generated"
LIB_DIR="build-xcframework/lib"
HEADERS_DIR="build-xcframework/Headers"

# 各ディレクトリをクリーンアップおよび再作成
rm -rf "$GENERATED_DIR" "$LIB_DIR" "$HEADERS_DIR"
mkdir -p "$GENERATED_DIR" "$LIB_DIR" "$HEADERS_DIR"

cd ../tapyrus-wallet-ffi/ || exit

# uniffi-bindgen により Swift バインディングを生成（Generated ディレクトリへ出力）
echo "Generating Swift bindings from src/wallet.udl..."
cargo run --release --bin uniffi-bindgen generate src/wallet.udl --language swift --out-dir "../TapyrusWalletSwift/$GENERATED_DIR"

# Rust 静的ライブラリのビルド
echo "Building Rust static library for iOS (device)..."
cargo build --release --target aarch64-apple-ios
echo "Building Rust static library for iOS Simulator..."
cargo build --release --target aarch64-apple-ios-sim

cd ../TapyrusWalletSwift/ || exit

# modulemap ファイルのリネーム：uniffi が "TapyrusWalletFFI.modulemap" として生成するので、
# Xcode の互換性のため "module.modulemap" に変更する
if [ -f "$GENERATED_DIR/TapyrusWalletFFI.modulemap" ]; then
  echo "Renaming TapyrusWalletFFI.modulemap to module.modulemap"
  mv "$GENERATED_DIR/TapyrusWalletFFI.modulemap" "$GENERATED_DIR/module.modulemap"
fi

mkdir -p "$LIB_DIR/device"
mkdir -p "$LIB_DIR/simulator"

# デバイス用ライブラリを "TapyrusWallet.a" にリネームしてコピー
cp ../tapyrus-wallet-ffi/target/aarch64-apple-ios/release/libtapyrus_wallet_ffi.a "$LIB_DIR/device/TapyrusWallet.a"
# Rust側のシミュレーターターゲットは "aarch64-apple-ios-sim" を使用
cp ../tapyrus-wallet-ffi/target/aarch64-apple-ios-sim/release/libtapyrus_wallet_ffi.a "$LIB_DIR/simulator/TapyrusWallet.a"

# Generated ディレクトリ内の Swift ソースファイルを取得
SWIFT_SOURCES=$(find "$GENERATED_DIR" -name "*.swift")

# ① ライブラリ本体のコンパイル（-emit-library）
echo "Compiling combined Swift + Rust library for device..."
xcrun swiftc -static -emit-library -module-name TapyrusWallet \
  -target arm64-apple-ios13.0 \
  -sdk $(xcrun --sdk iphoneos --show-sdk-path) \
  -Xcc -fmodule-map-file="$GENERATED_DIR/module.modulemap" \
  -import-objc-header "$GENERATED_DIR/TapyrusWalletFFI.h" \
  -o "$LIB_DIR/TapyrusWallet_Device.a" \
  $SWIFT_SOURCES \
  "$LIB_DIR/device/TapyrusWallet.a" \
  -I "$GENERATED_DIR"

echo "Compiling combined Swift + Rust library for simulator..."
xcrun swiftc -static -emit-library -module-name TapyrusWallet \
  -target arm64-apple-ios13.0-simulator \
  -emit-module -emit-module-path  $GENERATED_DIR \
  -sdk $(xcrun --sdk iphonesimulator --show-sdk-path) \
  -Xcc -fmodule-map-file="$GENERATED_DIR/module.modulemap" \
  -import-objc-header "$GENERATED_DIR/TapyrusWalletFFI.h" \
  -o "$LIB_DIR/TapyrusWallet_Sim.a" \
  $SWIFT_SOURCES \
  "$LIB_DIR/simulator/TapyrusWallet.a" \
  -I "$GENERATED_DIR"

cp "$GENERATED_DIR/module.modulemap" $HEADERS_DIR
cp "$GENERATED_DIR/TapyrusWalletFFI.h" $HEADERS_DIR

# XCFramework の作成：Module ディレクトリをヘッダーとして指定
echo "Creating XCFramework..."
xcodebuild -create-xcframework \
  -library "$LIB_DIR/TapyrusWallet_Device.a" -headers "$HEADERS_DIR" \
  -library "$LIB_DIR/TapyrusWallet_Sim.a" -headers "$HEADERS_DIR" \
  -output "$OUTPUT_DIR"

# XCFramework の Info.plist の修正
echo "Modifying XCFramework Info.plist..."
XCFRAMEWORK_PLIST="$OUTPUT_DIR/Info.plist"
echo "Using XCFramework plist at: $XCFRAMEWORK_PLIST"

# CFBundleIdentifier の設定
/usr/libexec/PlistBuddy -c "Delete :CFBundleIdentifier" "$XCFRAMEWORK_PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :CFBundleIdentifier string com.chaintope.tapyrus.wallet" "$XCFRAMEWORK_PLIST"

# CFBundleName の設定
/usr/libexec/PlistBuddy -c "Delete :CFBundleName" "$XCFRAMEWORK_PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :CFBundleName string TapyrusWallet" "$XCFRAMEWORK_PLIST"

# CFBundleVersion の設定
/usr/libexec/PlistBuddy -c "Delete :CFBundleVersion" "$XCFRAMEWORK_PLIST" 2>/dev/null || true
/usr/libexec/PlistBuddy -c "Add :CFBundleVersion string ${FIXED_VERSION}" "$XCFRAMEWORK_PLIST"

echo "XCFramework successfully created at $OUTPUT_DIR"

rm TapyrusWallet.xcframework.zip || true
zip -9 -r TapyrusWallet.xcframework.zip $OUTPUT_DIR

swift package compute-checksum TapyrusWallet.xcframework.zip
