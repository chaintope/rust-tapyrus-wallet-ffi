package com.chaintope.tapyrus.wallet.example

import android.content.Context
import android.util.Log
import java.io.File
import java.io.FileOutputStream
import java.io.IOException

/**
 * Helper class to ensure JNA native libraries are properly loaded
 */
object JnaLoader {
    private const val TAG = "JnaLoader"

    /**
     * Attempt to load JNA native libraries
     */
    fun load(context: Context) {
        try {
            // Try to load JNA directly first
            System.loadLibrary("jnidispatch")
            Log.d(TAG, "Successfully loaded jnidispatch library directly")
        } catch (e: UnsatisfiedLinkError) {
            Log.w(TAG, "Failed to load jnidispatch directly: ${e.message}")
            
            try {
                // Try to load from the app's native library directory
                val libraryName = "libjnidispatch.so"
                val libraryPath = File(context.applicationInfo.nativeLibraryDir, libraryName)
                
                if (libraryPath.exists()) {
                    Log.d(TAG, "Found JNA library at: ${libraryPath.absolutePath}")
                    System.load(libraryPath.absolutePath)
                    Log.d(TAG, "Successfully loaded JNA from native library directory")
                } else {
                    Log.w(TAG, "JNA library not found in native library directory")
                    
                    // As a last resort, try to extract and load from assets
                    extractAndLoadFromAssets(context)
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to load JNA library: ${e.message}", e)
            }
        }
    }
    
    /**
     * Attempt to extract JNA library from assets and load it
     */
    private fun extractAndLoadFromAssets(context: Context) {
        val libraryName = "libjnidispatch.so"
        val assetPath = "lib/${getArchitectureName()}/libjnidispatch.so"
        
        try {
            // Create a temporary file to extract the library to
            val tempFile = File(context.cacheDir, libraryName)
            if (tempFile.exists()) {
                tempFile.delete()
            }
            
            // Extract the library from assets
            context.assets.open(assetPath).use { input ->
                FileOutputStream(tempFile).use { output ->
                    input.copyTo(output)
                }
            }
            
            // Make the file executable
            tempFile.setExecutable(true)
            
            // Load the library
            System.load(tempFile.absolutePath)
            Log.d(TAG, "Successfully loaded JNA from assets")
        } catch (e: IOException) {
            Log.e(TAG, "Failed to extract JNA library from assets: ${e.message}", e)
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load extracted JNA library: ${e.message}", e)
        }
    }
    
    /**
     * Get the current device architecture name
     */
    private fun getArchitectureName(): String {
        val arch = System.getProperty("os.arch") ?: ""
        return when {
            arch.contains("arm64") || arch.contains("aarch64") -> "arm64-v8a"
            arch.contains("arm") -> "armeabi-v7a"
            arch.contains("86_64") || arch.contains("amd64") -> "x86_64"
            arch.contains("86") -> "x86"
            else -> throw UnsupportedOperationException("Unsupported architecture: $arch")
        }
    }
}
