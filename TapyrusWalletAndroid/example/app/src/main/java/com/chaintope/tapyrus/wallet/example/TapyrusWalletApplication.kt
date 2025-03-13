package com.chaintope.tapyrus.wallet.example

import android.app.Application
import android.util.Log

/**
 * Custom Application class to initialize libraries as early as possible
 */
class TapyrusWalletApplication : Application() {
    companion object {
        private const val TAG = "TapyrusWalletApp"
    }

    override fun onCreate() {
        super.onCreate()
        
        // Initialize JNA library at application startup
        try {
            Log.d(TAG, "Initializing JNA library...")
            JnaLoader.load(this)
            Log.d(TAG, "JNA library initialized successfully")
        } catch (e: Exception) {
            Log.e(TAG, "Failed to initialize JNA library: ${e.message}", e)
        }
    }
}
