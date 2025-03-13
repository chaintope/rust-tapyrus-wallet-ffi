package com.chaintope.tapyrus.wallet.example

import android.content.Context
import android.util.Log
import java.io.File
import java.util.zip.ZipFile

/**
 * Helper class to check if native libraries are properly packaged in the APK
 */
object NativeLibraryChecker {
    private const val TAG = "NativeLibraryChecker"

    /**
     * Check if the JNA native libraries are properly packaged in the APK
     */
    fun checkJnaLibraries(context: Context): String {
        val result = StringBuilder()
        
        try {
            // Check the native library directory
            val nativeLibDir = context.applicationInfo.nativeLibraryDir
            result.append("Native library directory: $nativeLibDir\n")
            
            val nativeLibDirFile = File(nativeLibDir)
            if (nativeLibDirFile.exists() && nativeLibDirFile.isDirectory) {
                val files = nativeLibDirFile.listFiles()
                result.append("Files in native library directory:\n")
                files?.forEach { file ->
                    result.append("  - ${file.name} (${file.length()} bytes)\n")
                }
            } else {
                result.append("Native library directory does not exist or is not a directory\n")
            }
            
            // Check if libjnidispatch.so exists
            val jnaLibFile = File(nativeLibDir, "libjnidispatch.so")
            result.append("libjnidispatch.so exists: ${jnaLibFile.exists()}\n")
            if (jnaLibFile.exists()) {
                result.append("libjnidispatch.so size: ${jnaLibFile.length()} bytes\n")
                result.append("libjnidispatch.so can read: ${jnaLibFile.canRead()}\n")
                result.append("libjnidispatch.so can execute: ${jnaLibFile.canExecute()}\n")
            }
            
            // Check the APK file
            val apkFile = context.applicationInfo.sourceDir
            result.append("\nAPK file: $apkFile\n")
            
            try {
                val zip = ZipFile(apkFile)
                val entries = zip.entries()
                val jnaEntries = mutableListOf<String>()
                
                while (entries.hasMoreElements()) {
                    val entry = entries.nextElement()
                    if (entry.name.contains("libjnidispatch.so")) {
                        jnaEntries.add("  - ${entry.name} (${entry.size} bytes)")
                    }
                }
                
                result.append("JNA libraries in APK:\n")
                if (jnaEntries.isEmpty()) {
                    result.append("  No JNA libraries found in APK\n")
                } else {
                    jnaEntries.forEach { result.append("$it\n") }
                }
                
                zip.close()
            } catch (e: Exception) {
                result.append("Failed to check APK contents: ${e.message}\n")
            }
            
            // Check system properties
            result.append("\nSystem properties:\n")
            result.append("  - os.arch: ${System.getProperty("os.arch")}\n")
            result.append("  - java.library.path: ${System.getProperty("java.library.path")}\n")
            
        } catch (e: Exception) {
            result.append("Error checking native libraries: ${e.message}")
            Log.e(TAG, "Error checking native libraries", e)
        }
        
        return result.toString()
    }
}
