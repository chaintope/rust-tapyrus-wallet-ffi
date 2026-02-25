package com.chaintope.tapyrus.wallet.example

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Button
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(
    walletManager: TapyrusWalletManager,
    onBack: () -> Unit
) {
    val prefs = walletManager.context.getSharedPreferences("tapyrus_wallet_prefs", 0)
    val coroutineScope = rememberCoroutineScope()

    var connectionType by remember {
        val saved = prefs.getString(SettingsKey.CONNECTION_TYPE, ConnectionType.ESPLORA.name)
        mutableStateOf(ConnectionType.valueOf(saved ?: ConnectionType.ESPLORA.name))
    }
    var esploraHost by remember {
        mutableStateOf(prefs.getString(SettingsKey.ESPLORA_HOST, DefaultConnection.ESPLORA_HOST) ?: DefaultConnection.ESPLORA_HOST)
    }
    var esploraPort by remember {
        mutableStateOf(prefs.getString(SettingsKey.ESPLORA_PORT, DefaultConnection.ESPLORA_PORT) ?: DefaultConnection.ESPLORA_PORT)
    }
    var electrumHost by remember {
        mutableStateOf(prefs.getString(SettingsKey.ELECTRUM_HOST, DefaultConnection.ELECTRUM_HOST) ?: DefaultConnection.ELECTRUM_HOST)
    }
    var electrumPort by remember {
        mutableStateOf(prefs.getString(SettingsKey.ELECTRUM_PORT, DefaultConnection.ELECTRUM_PORT) ?: DefaultConnection.ELECTRUM_PORT)
    }
    var errorMessage by remember { mutableStateOf<String?>(null) }

    val isValid = if (connectionType == ConnectionType.ESPLORA) {
        esploraHost.isNotBlank() && esploraPort.isNotBlank() && esploraPort.toUShortOrNull() != null
    } else {
        electrumHost.isNotBlank() && electrumPort.isNotBlank() && electrumPort.toUShortOrNull() != null
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Connection Settings") },
                navigationIcon = {
                    IconButton(onClick = onBack) {
                        Icon(Icons.AutoMirrored.Filled.ArrowBack, contentDescription = "Back")
                    }
                }
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(16.dp)
        ) {
            Text("Connection Method", style = MaterialTheme.typography.titleMedium)
            Spacer(modifier = Modifier.height(8.dp))

            Row(modifier = Modifier.fillMaxWidth()) {
                ConnectionType.entries.forEach { type ->
                    FilterChip(
                        selected = connectionType == type,
                        onClick = { connectionType = type },
                        label = { Text(type.label) },
                        modifier = Modifier.padding(end = 8.dp)
                    )
                }
            }

            Spacer(modifier = Modifier.height(24.dp))

            if (connectionType == ConnectionType.ESPLORA) {
                Text("Esplora Server", style = MaterialTheme.typography.titleMedium)
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = esploraHost,
                    onValueChange = { esploraHost = it },
                    label = { Text("Host") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = esploraPort,
                    onValueChange = { esploraPort = it },
                    label = { Text("Port") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    modifier = Modifier.fillMaxWidth()
                )
            } else {
                Text("Electrum Server", style = MaterialTheme.typography.titleMedium)
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = electrumHost,
                    onValueChange = { electrumHost = it },
                    label = { Text("Host") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth()
                )
                Spacer(modifier = Modifier.height(8.dp))
                OutlinedTextField(
                    value = electrumPort,
                    onValueChange = { electrumPort = it },
                    label = { Text("Port") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                    modifier = Modifier.fillMaxWidth()
                )
            }

            errorMessage?.let { msg ->
                Spacer(modifier = Modifier.height(8.dp))
                Text(msg, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
            }

            Spacer(modifier = Modifier.height(24.dp))

            Button(
                onClick = {
                    prefs.edit()
                        .putString(SettingsKey.CONNECTION_TYPE, connectionType.name)
                        .putString(SettingsKey.ESPLORA_HOST, esploraHost)
                        .putString(SettingsKey.ESPLORA_PORT, esploraPort)
                        .putString(SettingsKey.ELECTRUM_HOST, electrumHost)
                        .putString(SettingsKey.ELECTRUM_PORT, electrumPort)
                        .apply()

                    coroutineScope.launch {
                        try {
                            walletManager.reinitialize()
                            errorMessage = null
                            onBack()
                        } catch (e: Exception) {
                            errorMessage = "Failed to reinitialize: ${e.message}"
                        }
                    }
                },
                enabled = isValid,
                modifier = Modifier.fillMaxWidth()
            ) {
                Text("Apply & Reinitialize Wallet")
            }
        }
    }
}
