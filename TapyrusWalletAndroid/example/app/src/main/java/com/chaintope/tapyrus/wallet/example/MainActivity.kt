package com.chaintope.tapyrus.wallet.example

import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Send
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.chaintope.tapyrus.wallet.example.ui.theme.ExampleTheme
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    private lateinit var walletManager: TapyrusWalletManager
    
    companion object {
        private const val TAG = "MainActivity"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Initialize wallet manager
        walletManager = TapyrusWalletManager(this)
        
        enableEdgeToEdge()
        setContent {
            ExampleTheme {
                TapyrusWalletApp(walletManager)
            }
        }
    }
    
    override fun onDestroy() {
        super.onDestroy()
        walletManager.cleanup()
    }
}

@Composable
fun TapyrusWalletApp(walletManager: TapyrusWalletManager) {
    var showSettings by remember { mutableStateOf(false) }

    if (showSettings) {
        SettingsScreen(
            walletManager = walletManager,
            onBack = { showSettings = false }
        )
        return
    }

    TapyrusWalletMainScreen(walletManager, onOpenSettings = { showSettings = true })
}

@Composable
fun TapyrusWalletMainScreen(
    walletManager: TapyrusWalletManager,
    onOpenSettings: () -> Unit
) {
    val coroutineScope = rememberCoroutineScope()
    val snackbarHostState = remember { SnackbarHostState() }
    val clipboardManager = LocalClipboardManager.current

    // State for showing copied message
    var showDiagnosticInfo by remember { mutableStateOf(false) }
    var diagnosticInfo by remember { mutableStateOf("") }

    // State for transfer dialog and alerts
    var showTransferDialog by remember { mutableStateOf(false) }
    var showTransferSuccessAlert by remember { mutableStateOf(false) }
    var showTransferErrorAlert by remember { mutableStateOf(false) }
    var transactionId by remember { mutableStateOf("") }
    var transferErrorMessage by remember { mutableStateOf("") }

    // Initialize wallet
    LaunchedEffect(Unit) {
        try {
            // Check native libraries and get diagnostic info
            diagnosticInfo = NativeLibraryChecker.checkJnaLibraries(walletManager.context)

            // Initialize wallet
            walletManager.initialize()
        } catch (e: Exception) {
            Log.e("TapyrusWalletApp", "Error initializing wallet: ${e.message}", e)
        }
    }

    Scaffold(
        modifier = Modifier.fillMaxSize(),
        snackbarHost = { SnackbarHost(snackbarHostState) }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Logo and title
            Image(
                painter = painterResource(id = R.drawable.logo),
                contentDescription = "Tapyrus Logo",
                modifier = Modifier
                    .size(120.dp)
                    .padding(bottom = 10.dp)
            )
            
            Text(
                text = "Tapyrus Wallet",
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Balance section
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Text(
                        text = "Balance",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Row(
                        verticalAlignment = Alignment.Bottom,
                        modifier = Modifier.padding(top = 8.dp)
                    ) {
                        Text(
                            text = String.format("%.8f", walletManager.balance),
                            style = MaterialTheme.typography.headlineMedium,
                            fontWeight = FontWeight.Bold
                        )
                        
                        Text(
                            text = " TPC",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(bottom = 4.dp)
                        )
                    }
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Address section
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Text(
                        text = "Current Address",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    if (walletManager.currentAddress.isEmpty()) {
                        Text(
                            text = "No address generated yet",
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(top = 8.dp)
                        )
                    } else {
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(top = 8.dp),
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text(
                                text = walletManager.currentAddress,
                                style = MaterialTheme.typography.bodyMedium,
                                modifier = Modifier.weight(1f)
                            )
                            
                            IconButton(
                                onClick = {
                                    clipboardManager.setText(AnnotatedString(walletManager.currentAddress))
                                    coroutineScope.launch {
                                        snackbarHostState.showSnackbar("Address copied to clipboard")
                                    }
                                }
                            ) {
                                Icon(
                                    imageVector = Icons.Outlined.ContentCopy,
                                    contentDescription = "Copy Address"
                                )
                            }
                        }
                    }
                    
                    Button(
                        onClick = {
                            coroutineScope.launch {
                                val newAddress = walletManager.getNewAddress()
                                if (newAddress.isNotEmpty()) {
                                    clipboardManager.setText(AnnotatedString(newAddress))
                                    snackbarHostState.showSnackbar("New address generated and copied to clipboard")
                                }
                            }
                        },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp)
                    ) {
                        Text("Generate & Copy Address")
                    }
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Action buttons
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = {
                        coroutineScope.launch {
                            walletManager.syncWallet()
                        }
                    },
                    modifier = Modifier.weight(1f),
                    enabled = !walletManager.isSyncing
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = Icons.Filled.Refresh,
                            contentDescription = "Sync",
                            modifier = Modifier.padding(end = 4.dp)
                        )
                        Text("Sync Wallet")
                    }
                }
                
                Button(
                    onClick = { showTransferDialog = true },
                    modifier = Modifier.weight(1f),
                    enabled = !walletManager.isSyncing && walletManager.balance > 0
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = Icons.Filled.Send,
                            contentDescription = "Send",
                            modifier = Modifier.padding(end = 4.dp)
                        )
                        Text("Send")
                    }
                }
            }
            
            // Show sync indicator or sync result
            if (walletManager.isSyncing) {
                Row(
                    modifier = Modifier
                        .padding(top = 16.dp)
                        .fillMaxWidth(),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(20.dp),
                        strokeWidth = 2.dp
                    )
                    Text(
                        text = "Syncing...",
                        style = MaterialTheme.typography.bodyMedium,
                        modifier = Modifier.padding(start = 8.dp)
                    )
                }
            } else {
                walletManager.syncResultMessage?.let { message ->
                    val backgroundColor = if (walletManager.syncResultIsError)
                        MaterialTheme.colorScheme.errorContainer
                    else
                        MaterialTheme.colorScheme.primaryContainer
                    val textColor = if (walletManager.syncResultIsError)
                        MaterialTheme.colorScheme.onErrorContainer
                    else
                        MaterialTheme.colorScheme.onPrimaryContainer
                    Box(
                        modifier = Modifier
                            .padding(top = 16.dp)
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(4.dp))
                            .background(backgroundColor)
                            .padding(8.dp)
                    ) {
                        Text(
                            text = message,
                            color = textColor,
                            style = MaterialTheme.typography.bodySmall,
                            textAlign = TextAlign.Center,
                            modifier = Modifier.fillMaxWidth()
                        )
                    }
                }
            }
            
            // Connection info and settings
            if (walletManager.connectionInfo.isNotEmpty()) {
                Text(
                    text = walletManager.connectionInfo,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 16.dp)
                )
            }

            // Settings button
            Button(
                onClick = onOpenSettings,
                modifier = Modifier.padding(top = 8.dp)
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        imageVector = Icons.Filled.Settings,
                        contentDescription = "Settings",
                        modifier = Modifier.padding(end = 4.dp)
                    )
                    Text("Connection Settings")
                }
            }

            // Toggle diagnostic info button
            Button(
                onClick = { showDiagnosticInfo = !showDiagnosticInfo },
                modifier = Modifier
                    .padding(top = 8.dp)
                    .align(Alignment.End)
            ) {
                Text(if (showDiagnosticInfo) "Hide Diagnostic Info" else "Show Diagnostic Info")
            }

            // Transfer dialog
            if (showTransferDialog) {
                TransferDialog(
                    onDismiss = { showTransferDialog = false },
                    walletManager = walletManager,
                    onTransferSuccess = { txid ->
                        transactionId = txid
                        showTransferSuccessAlert = true
                    },
                    onTransferError = { error ->
                        transferErrorMessage = error
                        showTransferErrorAlert = true
                    }
                )
            }
            
            // Success alert
            if (showTransferSuccessAlert) {
                AlertDialog(
                    onDismissRequest = { showTransferSuccessAlert = false },
                    title = { Text("Transfer Successful") },
                    text = { Text("Transaction ID: $transactionId") },
                    confirmButton = {
                        Button(onClick = { showTransferSuccessAlert = false }) {
                            Text("OK")
                        }
                    }
                )
            }
            
            // Error alert
            if (showTransferErrorAlert) {
                AlertDialog(
                    onDismissRequest = { showTransferErrorAlert = false },
                    title = { Text("Transfer Error") },
                    text = { Text(transferErrorMessage) },
                    confirmButton = {
                        Button(onClick = { showTransferErrorAlert = false }) {
                            Text("OK")
                        }
                    }
                )
            }
            
            // Diagnostic info
            if (showDiagnosticInfo && diagnosticInfo.isNotEmpty()) {
                Box(
                    modifier = Modifier
                        .padding(top = 8.dp)
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.surfaceVariant)
                        .padding(8.dp)
                ) {
                    Text(
                        text = diagnosticInfo,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    }
}

@Preview(showBackground = true)
@Composable
fun TapyrusWalletAppPreview() {
    // Create a dummy wallet manager for preview
    val dummyWalletManager = remember {
        object {
            val context = null
            val currentAddress = "tmXjiS7Qpw9aCFPU542eBdnQvZYH9MuVoy"
            val balance = 0.12345678
            val isSyncing = false
            val errorMessage: String? = null
        }
    }
    
    ExampleTheme {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            // Logo and title
            Image(
                painter = painterResource(id = R.drawable.logo),
                contentDescription = "Tapyrus Logo",
                modifier = Modifier
                    .size(120.dp)
                    .padding(bottom = 10.dp)
            )
            
            Text(
                text = "Tapyrus Wallet",
                style = MaterialTheme.typography.headlineLarge,
                fontWeight = FontWeight.Bold
            )
            
            Spacer(modifier = Modifier.height(24.dp))
            
            // Balance section
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Text(
                        text = "Balance",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Row(
                        verticalAlignment = Alignment.Bottom,
                        modifier = Modifier.padding(top = 8.dp)
                    ) {
                        Text(
                            text = String.format("%.8f", dummyWalletManager.balance),
                            style = MaterialTheme.typography.headlineMedium,
                            fontWeight = FontWeight.Bold
                        )
                        
                        Text(
                            text = " TPC",
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(bottom = 4.dp)
                        )
                    }
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Address section
            Card(
                modifier = Modifier.fillMaxWidth()
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp)
                ) {
                    Text(
                        text = "Current Address",
                        style = MaterialTheme.typography.titleMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(
                            text = dummyWalletManager.currentAddress,
                            style = MaterialTheme.typography.bodyMedium,
                            modifier = Modifier.weight(1f)
                        )
                        
                        IconButton(onClick = { }) {
                            Icon(
                                imageVector = Icons.Outlined.ContentCopy,
                                contentDescription = "Copy Address"
                            )
                        }
                    }
                    
                    Button(
                        onClick = { },
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(top = 8.dp)
                    ) {
                        Text("Generate & Copy Address")
                    }
                }
            }
            
            Spacer(modifier = Modifier.height(16.dp))
            
            // Action buttons
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                Button(
                    onClick = { },
                    modifier = Modifier.weight(1f)
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = Icons.Filled.Refresh,
                            contentDescription = "Sync",
                            modifier = Modifier.padding(end = 4.dp)
                        )
                        Text("Sync Wallet")
                    }
                }
                
                Button(
                    onClick = { },
                    modifier = Modifier.weight(1f)
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = Icons.Filled.Send,
                            contentDescription = "Send",
                            modifier = Modifier.padding(end = 4.dp)
                        )
                        Text("Send")
                    }
                }
            }
        }
    }
}
