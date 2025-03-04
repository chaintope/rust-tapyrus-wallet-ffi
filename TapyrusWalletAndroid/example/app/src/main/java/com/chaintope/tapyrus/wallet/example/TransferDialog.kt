package com.chaintope.tapyrus.wallet.example

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch

@Composable
fun TransferDialog(
    onDismiss: () -> Unit,
    walletManager: TapyrusWalletManager,
    onTransferSuccess: (String) -> Unit,
    onTransferError: (String) -> Unit
) {
    var address by remember { mutableStateOf("") }
    var amount by remember { mutableStateOf("") }
    var isProcessing by remember { mutableStateOf(false) }
    val coroutineScope = rememberCoroutineScope()

    // Validation
    val isAmountValid = amount.isNotEmpty() && amount.toDoubleOrNull() != null && 
                        amount.toDoubleOrNull()!! > 0 && 
                        amount.toDoubleOrNull()!! <= walletManager.balance
    val isAddressValid = address.isNotEmpty()
    val isFormValid = isAddressValid && isAmountValid

    AlertDialog(
        onDismissRequest = { if (!isProcessing) onDismiss() },
        title = { Text("Send TPC") },
        text = {
            Column {
                // Recipient address field
                OutlinedTextField(
                    value = address,
                    onValueChange = { address = it },
                    label = { Text("Recipient Address") },
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    enabled = !isProcessing
                )
                
                Spacer(modifier = Modifier.height(16.dp))
                
                // Amount field
                OutlinedTextField(
                    value = amount,
                    onValueChange = { amount = it },
                    label = { Text("Amount (TPC)") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal),
                    modifier = Modifier.fillMaxWidth(),
                    singleLine = true,
                    enabled = !isProcessing
                )
                
                // Amount validation message
                if (amount.isNotEmpty() && amount.toDoubleOrNull() != null && 
                    amount.toDoubleOrNull()!! > walletManager.balance) {
                    Text(
                        text = "Insufficient balance",
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(top = 4.dp)
                    )
                }
                
                // Available balance
                Text(
                    text = "Available Balance: ${String.format("%.8f", walletManager.balance)} TPC",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 4.dp)
                )
            }
        },
        confirmButton = {
            Button(
                onClick = {
                    if (isFormValid) {
                        isProcessing = true
                        coroutineScope.launch {
                            try {
                                val amountValue = amount.toDouble()
                                val txid = walletManager.transfer(address, amountValue)
                                isProcessing = false
                                onTransferSuccess(txid)
                                onDismiss()
                            } catch (e: Exception) {
                                isProcessing = false
                                onTransferError(e.message ?: "Unknown error")
                                onDismiss()
                            }
                        }
                    }
                },
                enabled = isFormValid && !isProcessing
            ) {
                if (isProcessing) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        CircularProgressIndicator(
                            modifier = Modifier
                                .padding(end = 8.dp)
                                .height(16.dp),
                            strokeWidth = 2.dp
                        )
                        Text("Processing...")
                    }
                } else {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Icon(
                            imageVector = Icons.Default.Send,
                            contentDescription = "Send",
                            modifier = Modifier.padding(end = 8.dp)
                        )
                        Text("Send")
                    }
                }
            }
        },
        dismissButton = {
            TextButton(
                onClick = onDismiss,
                enabled = !isProcessing
            ) {
                Text("Cancel")
            }
        }
    )
}
