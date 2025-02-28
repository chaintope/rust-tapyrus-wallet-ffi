//
//  TransferView.swift
//  example
//
//  Created on 2025/02/28.
//

import SwiftUI
import TapyrusWallet

struct TransferView: View {
    @Binding var isPresented: Bool
    @Binding var address: String
    @Binding var amount: String
    @ObservedObject var walletManager: TapyrusWalletManager
    @Binding var showError: Bool
    @Binding var errorMessage: String
    @Binding var showSuccess: Bool
    @Binding var transactionId: String
    
    @State private var isProcessing = false
    
    // Computed property to validate the form
    private var isFormValid: Bool {
        !address.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
        !amount.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
        Double(amount) != nil &&
        Double(amount)! > 0 &&
        Double(amount)! <= walletManager.balance
    }
    
    var body: some View {
        NavigationView {
            Form {
                Section(header: Text("Recipient")) {
                    TextField("Address", text: $address)
                        .autocapitalization(.none)
                        .disableAutocorrection(true)
                }
                
                Section(header: Text("Amount")) {
                    TextField("Amount (TPC)", text: $amount)
                        .keyboardType(.decimalPad)
                    
                    if let amountValue = Double(amount), amountValue > walletManager.balance {
                        Text("Insufficient balance")
                            .foregroundColor(.red)
                            .font(.caption)
                    }
                    
                    Text("Available Balance: \(String(format: "%.8f", walletManager.balance)) TPC")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
                
                Section {
                    Button(action: executeTransfer) {
                        if isProcessing {
                            HStack {
                                Spacer()
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle())
                                Text("Processing...")
                                Spacer()
                            }
                        } else {
                            HStack {
                                Spacer()
                                Image(systemName: "paperplane.fill")
                                Text("Send")
                                Spacer()
                            }
                        }
                    }
                    .disabled(!isFormValid || isProcessing)
                }
            }
            .navigationTitle("Send TPC")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button("Cancel") {
                        isPresented = false
                    }
                }
            }
        }
    }
    
    private func executeTransfer() {
        guard let amountValue = Double(amount), amountValue <= walletManager.balance else {
            errorMessage = "Invalid amount or insufficient balance"
            showError = true
            return
        }
        
        isProcessing = true
        
        Task {
            do {
                let txid = try await walletManager.transfer(toAddress: address, amount: amountValue)
                
                // Update UI on main thread
                DispatchQueue.main.async {
                    isProcessing = false
                    transactionId = txid
                    showSuccess = true
                    isPresented = false
                }
            } catch {
                // Handle error on main thread
                DispatchQueue.main.async {
                    isProcessing = false
                    errorMessage = error.localizedDescription
                    showError = true
                }
            }
        }
    }
}

#Preview {
    TransferView(
        isPresented: .constant(true),
        address: .constant(""),
        amount: .constant(""),
        walletManager: TapyrusWalletManager(),
        showError: .constant(false),
        errorMessage: .constant(""),
        showSuccess: .constant(false),
        transactionId: .constant("")
    )
}
