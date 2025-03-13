package com.chaintope.tapyrus.wallet.example

import android.content.Context
import android.util.Log
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.chaintope.tapyrus.wallet.Config
import com.chaintope.tapyrus.wallet.HdWallet
import com.chaintope.tapyrus.wallet.Network
import com.chaintope.tapyrus.wallet.TransferParams
import com.chaintope.tapyrus.wallet.generateMasterKey
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File

/**
 * Manager class for Tapyrus Wallet operations
 */
class TapyrusWalletManager(val context: Context) {
    companion object {
        private const val TAG = "TapyrusWalletManager"
        private const val DB_FILENAME = "tapyrus_wallet.db"
        private const val MASTER_KEY_PREF = "tapyrus_master_key"
        private const val PREF_NAME = "tapyrus_wallet_prefs"
    }

    // State properties
    var currentAddress by mutableStateOf("")
        private set
    var balance by mutableStateOf(0.0)
        private set
    var isSyncing by mutableStateOf(false)
        private set
    var errorMessage by mutableStateOf<String?>(null)
        private set

    // Wallet instances
    private var config: Config? = null
    private var wallet: HdWallet? = null
    private var isInitialized = false

    /**
     * Initialize the wallet
     */
    suspend fun initialize() {
        if (isInitialized) return
        
        withContext(Dispatchers.IO) {
            try {
                // Ensure JNA is loaded
                JnaLoader.load(context)
                
                // Get or create master key
                val masterKey = getMasterKey()

                // Get database file path
                val dbFilePath = File(context.filesDir, DB_FILENAME).absolutePath
                
                // Create config
                val networkMode = Network.PROD
                config = Config(
                    networkMode = networkMode,
                    networkId = 1939510133u,
                    genesisHash = "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
                    esploraUrl = "http://10.0.2.2:3001", // 10.0.2.2 is the host for the emulator
                    masterKey = masterKey,
                    dbFilePath = dbFilePath
                )
                
                // Create wallet
                wallet = HdWallet(config!!)
                
                // Mark as initialized before calling methods that might check this flag
                isInitialized = true
                
                // Get initial address if none exists
                if (currentAddress.isEmpty()) {
                    getNewAddressInternal()
                }
                
                // Sync wallet to get latest balance
                isSyncing = true
                wallet?.sync()
                isSyncing = false
                
                // Update balance
                updateBalanceInternal()
                errorMessage = null
            } catch (e: Exception) {
                Log.e(TAG, "Failed to initialize wallet: ${e.message}", e)
                errorMessage = "Failed to initialize wallet: ${e.message}"
            }
        }
    }

    /**
     * Get the master key from preferences or generate a new one
     */
    private fun getMasterKey(): String {
        val prefs = context.getSharedPreferences(PREF_NAME, Context.MODE_PRIVATE)
        var masterKey = prefs.getString(MASTER_KEY_PREF, null)
        
        if (masterKey == null) {
            // Generate a new master key
            masterKey = generateMasterKey(Network.PROD)
            
            // Save to preferences
            prefs.edit().putString(MASTER_KEY_PREF, masterKey).apply()
            Log.d(TAG, "Generated and saved new master key")
        } else {
            Log.d(TAG, "Using existing master key from preferences")
        }
        
        return masterKey
    }

    /**
     * Generate a new address - internal implementation that doesn't check initialization
     */
    private suspend fun getNewAddressInternal(): String {
        return withContext(Dispatchers.IO) {
            try {
                val result = wallet?.getNewAddress(null)
                if (result != null) {
                    currentAddress = result.address
                    Log.d(TAG, "Generated new address: ${result.address}")
                    currentAddress
                } else {
                    throw Exception("Failed to generate address")
                }
            } catch (e: Exception) {
                Log.e(TAG, "Error generating address: ${e.message}", e)
                errorMessage = "Error generating address: ${e.message}"
                ""
            }
        }
    }
    
    /**
     * Generate a new address - public method that ensures initialization
     */
    suspend fun getNewAddress(): String {
        return withContext(Dispatchers.IO) {
            try {
                if (!isInitialized) {
                    initialize()
                    return@withContext currentAddress // Return the address created during initialization
                }
                
                getNewAddressInternal()
            } catch (e: Exception) {
                Log.e(TAG, "Error generating address: ${e.message}", e)
                errorMessage = "Error generating address: ${e.message}"
                ""
            }
        }
    }

    /**
     * Update the wallet balance - internal implementation that doesn't check initialization
     */
    private suspend fun updateBalanceInternal() {
        withContext(Dispatchers.IO) {
            try {
                wallet?.let {
                    // Get balance in satoshis and convert to TPC
                    val balanceInSatoshis = it.balance(null)
                    balance = balanceInSatoshis.toDouble() / 100_000_000.0
                    Log.d(TAG, "Updated balance: $balance TPC")
                }
                errorMessage = null
            } catch (e: Exception) {
                Log.e(TAG, "Error updating balance: ${e.message}", e)
                errorMessage = "Error updating balance: ${e.message}"
            }
        }
    }
    
    /**
     * Update the wallet balance - public method that ensures initialization
     */
    suspend fun updateBalance() {
        withContext(Dispatchers.IO) {
            try {
                if (!isInitialized) {
                    initialize()
                    return@withContext // Balance is already updated during initialization
                }
                
                updateBalanceInternal()
            } catch (e: Exception) {
                Log.e(TAG, "Error updating balance: ${e.message}", e)
                errorMessage = "Error updating balance: ${e.message}"
            }
        }
    }

    /**
     * Sync the wallet with the blockchain
     */
    suspend fun syncWallet() {
        withContext(Dispatchers.IO) {
            try {
                if (!isInitialized) {
                    initialize()
                    return@withContext // No need to sync immediately after initialization
                }
                
                isSyncing = true
                wallet?.sync()
                updateBalanceInternal() // Use internal method to avoid potential recursion
                isSyncing = false
                errorMessage = null
            } catch (e: Exception) {
                Log.e(TAG, "Error syncing wallet: ${e.message}", e)
                errorMessage = "Error syncing wallet: ${e.message}"
                isSyncing = false
            }
        }
    }

    /**
     * Transfer TPC to another address
     */
    suspend fun transfer(toAddress: String, amount: Double): String {
        return withContext(Dispatchers.IO) {
            try {
                if (!isInitialized) {
                    initialize()
                }
                
                // Convert amount from TPC to satoshis
                val amountInSatoshis = (amount * 100_000_000.0).toULong()
                
                // Create transfer parameters
                val transferParams = TransferParams(amount = amountInSatoshis, toAddress = toAddress)
                
                // Execute the transfer
                val txid = wallet?.transfer(params = listOf(transferParams), utxos = listOf()) ?: 
                    throw Exception("Wallet is not initialized")
                
                // Update balance after transfer
                updateBalanceInternal()
                
                errorMessage = null
                txid
            } catch (e: Exception) {
                Log.e(TAG, "Error transferring funds: ${e.message}", e)
                errorMessage = "Error transferring funds: ${e.message}"
                throw e
            }
        }
    }

    /**
     * Clean up resources
     */
    fun cleanup() {
        try {
            wallet?.close()
            config?.close()
            isInitialized = false
        } catch (e: Exception) {
            Log.e(TAG, "Error cleaning up wallet resources: ${e.message}", e)
        }
    }
}
