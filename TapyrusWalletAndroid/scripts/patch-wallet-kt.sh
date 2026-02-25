#!/bin/bash
#
# Workaround for JNA struct-by-value bug on Android ARM64.
# See: https://github.com/chaintope/rust-tapyrus-wallet-ffi/issues/12
#      https://github.com/mozilla/uniffi-rs/issues/2624
#
# This script patches the UniFFI-generated wallet.kt to bypass JNA's
# broken Structure.ByValue handling by using Function.invoke() with
# explicit Pointer parameters for the Config constructor.
#

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <path-to-wallet.kt>"
    exit 1
fi

WALLET_KT="$1"

if [ ! -f "$WALLET_KT" ]; then
    echo "Error: wallet.kt not found at $WALLET_KT"
    exit 1
fi

# Skip if already patched
if grep -q "uniffiConfigNewWorkaround" "$WALLET_KT"; then
    echo "wallet.kt is already patched, skipping."
    exit 0
fi

echo "Patching wallet.kt for JNA ARM64 workaround..."

ORIGINAL_CALL="UniffiLib.INSTANCE.uniffi_tapyrus_wallet_ffi_fn_constructor_config_new("
ANCHOR="^open class Config"

# Validate that the expected anchor and call site exist before patching
ANCHOR_COUNT=$(grep -c "$ANCHOR" "$WALLET_KT" || true)
if [ "$ANCHOR_COUNT" -ne 1 ]; then
    echo "Error: expected exactly 1 '$ANCHOR' in wallet.kt, found $ANCHOR_COUNT"
    echo "UniFFI generated code may have changed. Patch needs updating."
    exit 1
fi

CALL_COUNT=$(grep -c "$ORIGINAL_CALL" "$WALLET_KT" || true)
if [ "$CALL_COUNT" -eq 0 ]; then
    echo "Error: original call site not found in wallet.kt"
    echo "UniFFI generated code may have changed. Patch needs updating."
    exit 1
fi

# 1. Write the workaround function to a temp file
WORKAROUND_TMP=$(mktemp)
cat > "$WORKAROUND_TMP" << 'KOTLIN_EOF'

// Workaround for JNA struct-by-value bug on Android ARM64.
// See: https://github.com/chaintope/rust-tapyrus-wallet-ffi/issues/12
private fun uniffiConfigNewWorkaround(
    networkMode: RustBuffer.ByValue,
    networkId: Int,
    genesisHash: RustBuffer.ByValue,
    esploraUrl: RustBuffer.ByValue,
    esploraUser: RustBuffer.ByValue,
    esploraPassword: RustBuffer.ByValue,
    electrumDomain: RustBuffer.ByValue,
    electrumPort: RustBuffer.ByValue,
    masterKeyPath: RustBuffer.ByValue,
    masterKey: RustBuffer.ByValue,
    dbFilePath: RustBuffer.ByValue,
    status: UniffiRustCallStatus
): Pointer {
    // ABI assumption: RustBuffer.ByValue is 24 bytes (capacity: Long, len: Long, data: Pointer).
    // If UniFFI changes the RustBuffer layout, this must be updated accordingly.
    fun rbToPtr(rb: RustBuffer.ByValue): com.sun.jna.Memory {
        val mem = com.sun.jna.Memory(24)
        mem.setLong(0, rb.capacity)
        mem.setLong(8, rb.len)
        mem.setPointer(16, rb.data)
        return mem
    }

    val fn = com.sun.jna.NativeLibrary.getInstance("tapyrus_wallet_ffi")
        .getFunction("uniffi_tapyrus_wallet_ffi_fn_constructor_config_new")

    val result = fn.invoke(Pointer::class.java, arrayOf(
        rbToPtr(networkMode),
        networkId,
        rbToPtr(genesisHash),
        rbToPtr(esploraUrl),
        rbToPtr(esploraUser),
        rbToPtr(esploraPassword),
        rbToPtr(electrumDomain),
        rbToPtr(electrumPort),
        rbToPtr(masterKeyPath),
        rbToPtr(masterKey),
        rbToPtr(dbFilePath),
        status
    )) as Pointer

    return result
}

KOTLIN_EOF

# 2. Insert the workaround function before "open class Config"
awk -v anchor="$ANCHOR" '
    FNR==NR { workaround = workaround $0 "\n"; next }
    $0 ~ anchor { printf "%s", workaround }
    { print }
' "$WORKAROUND_TMP" "$WALLET_KT" > "${WALLET_KT}.tmp"
mv "${WALLET_KT}.tmp" "$WALLET_KT"

# 3. Replace the direct native call with the workaround
sed -i.bak 's/UniffiLib\.INSTANCE\.uniffi_tapyrus_wallet_ffi_fn_constructor_config_new(/uniffiConfigNewWorkaround(/' "$WALLET_KT"

# Clean up
rm -f "${WALLET_KT}.bak" "$WORKAROUND_TMP"

# Verify the patch was applied correctly
ERRORS=0

# Check that the workaround function was inserted
if ! grep -q "uniffiConfigNewWorkaround" "$WALLET_KT"; then
    echo "Error: workaround function was not inserted"
    ERRORS=$((ERRORS + 1))
fi

# Check that the original call site was fully replaced
REMAINING=$(grep -c "$ORIGINAL_CALL" "$WALLET_KT" || true)
if [ "$REMAINING" -ne 0 ]; then
    echo "Error: $REMAINING original call site(s) still remain unreplaced"
    ERRORS=$((ERRORS + 1))
fi

if [ "$ERRORS" -ne 0 ]; then
    echo "Patch verification failed with $ERRORS error(s)"
    exit 1
fi

echo "Successfully patched wallet.kt ($CALL_COUNT call site(s) replaced)"
