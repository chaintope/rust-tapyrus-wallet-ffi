//
//  ContentView.swift
//  example
//
//  Created by Kohei Taniguchi on 2025/02/18.
//

import SwiftUI
import TapyrusWallet

struct ContentView: View {
    @EnvironmentObject private var walletManager: TapyrusWalletManager
    @State private var showCopiedAlert = false
    @State private var showTransferSheet = false
    @State private var transferAddress = ""
    @State private var transferAmount = ""
    @State private var showTransferError = false
    @State private var transferErrorMessage = ""
    @State private var showTransferSuccess = false
    @State private var transactionId = ""
    
    let mode: Network = .prod
    
    var body: some View {
        NavigationView {
            VStack(spacing: 20) {
                // Logo and title
                Image("TapyrusLogo")
                    .resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(height: 80)
                    .padding(.bottom, 10)
                
                Text("Tapyrus Wallet")
                    .font(.largeTitle)
                    .fontWeight(.bold)
                
                // Balance section
                VStack(alignment: .leading, spacing: 8) {
                    Text("Balance")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    
                    HStack(alignment: .firstTextBaseline) {
                        Text(String(format: "%.8f", walletManager.balance))
                            .font(.system(size: 36, weight: .bold))
                        
                        Text("TPC")
                            .font(.title2)
                            .foregroundColor(.secondary)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)
                
                // Address section
                VStack(alignment: .leading, spacing: 8) {
                    Text("Current Address")
                        .font(.headline)
                        .foregroundColor(.secondary)
                    
                    if walletManager.currentAddress.isEmpty {
                        Text("No address generated yet")
                            .foregroundColor(.secondary)
                            .italic()
                    } else {
                        Text(walletManager.currentAddress)
                            .font(.system(.body, design: .monospaced))
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                    
                    HStack {
                        Button(action: {
                            let address = walletManager.getNewAddress()
                            print("Generate & Copy Address: \(address)")
                            UIPasteboard.general.string = address
                            showCopiedAlert = true
                        }) {
                            Label("Generate & Copy Address", systemImage: "doc.on.doc")
                                .frame(maxWidth: .infinity)
                        }
                        .buttonStyle(.borderedProminent)
                        .tint(.blue)
                    }
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding()
                .background(Color(.systemGray6))
                .cornerRadius(12)
                
                // Action buttons
                HStack(spacing: 12) {
                    // Sync button
                    Button(action: {
                        walletManager.syncWallet()
                    }) {
                        HStack {
                            Image(systemName: "arrow.triangle.2.circlepath")
                            Text("Sync Wallet")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.bordered)
                    .disabled(walletManager.isSyncing)
                    
                    // Send button
                    Button(action: {
                        showTransferSheet = true
                    }) {
                        HStack {
                            Image(systemName: "paperplane.fill")
                            Text("Send")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .buttonStyle(.borderedProminent)
                    .tint(.green)
                    .disabled(walletManager.isSyncing || walletManager.balance <= 0)
                }
                
                if walletManager.isSyncing {
                    HStack {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle())
                        Text("Syncing...")
                            .foregroundColor(.secondary)
                    }
                }
                
                Spacer()
            }
            .padding()
            .navigationBarTitleDisplayMode(.inline)
            .alert("Address Copied", isPresented: $showCopiedAlert) {
                Button("OK", role: .cancel) {}
            } message: {
                Text("The address has been copied to clipboard.")
            }
            .sheet(isPresented: $showTransferSheet) {
                // Reset form when sheet is dismissed
                transferAddress = ""
                transferAmount = ""
            } content: {
                TransferView(
                    isPresented: $showTransferSheet,
                    address: $transferAddress,
                    amount: $transferAmount,
                    walletManager: walletManager,
                    showError: $showTransferError,
                    errorMessage: $transferErrorMessage,
                    showSuccess: $showTransferSuccess,
                    transactionId: $transactionId
                )
            }
            .alert("Transfer Error", isPresented: $showTransferError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(transferErrorMessage)
            }
            .alert("Transfer Successful", isPresented: $showTransferSuccess) {
                Button("OK", role: .cancel) {}
            } message: {
                Text("Transaction ID: \(transactionId)")
            }
        }
    }
}

#Preview {
    ContentView()
        .environmentObject(TapyrusWalletManager())
}
