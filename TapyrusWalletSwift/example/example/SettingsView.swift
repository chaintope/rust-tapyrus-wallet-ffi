//
//  SettingsView.swift
//  example
//

import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var walletManager: TapyrusWalletManager
    @Environment(\.dismiss) private var dismiss

    @AppStorage(SettingsKey.connectionType)
    private var connectionType: String = ConnectionType.esplora.rawValue

    @AppStorage(SettingsKey.esploraHost)
    private var esploraHost: String = DefaultConnection.esploraHost

    @AppStorage(SettingsKey.esploraPort)
    private var esploraPort: String = DefaultConnection.esploraPort

    @AppStorage(SettingsKey.electrumHost)
    private var electrumHost: String = DefaultConnection.electrumHost

    @AppStorage(SettingsKey.electrumPort)
    private var electrumPort: String = DefaultConnection.electrumPort

    @State private var showError = false
    @State private var errorMessage = ""

    var body: some View {
        NavigationView {
            Form {
                Section("Connection Method") {
                    Picker("Type", selection: $connectionType) {
                        Text("Esplora").tag(ConnectionType.esplora.rawValue)
                        Text("Electrum").tag(ConnectionType.electrum.rawValue)
                    }
                    .pickerStyle(.segmented)
                }

                if connectionType == ConnectionType.esplora.rawValue {
                    Section("Esplora Server") {
                        TextField("Host", text: $esploraHost)
                            .autocapitalization(.none)
                            .disableAutocorrection(true)
                            .keyboardType(.URL)
                        TextField("Port", text: $esploraPort)
                            .keyboardType(.numberPad)
                    }
                } else {
                    Section("Electrum Server") {
                        TextField("Host", text: $electrumHost)
                            .autocapitalization(.none)
                            .disableAutocorrection(true)
                            .keyboardType(.URL)
                        TextField("Port", text: $electrumPort)
                            .keyboardType(.numberPad)
                    }
                }

                Section {
                    Button(action: applySettings) {
                        HStack {
                            Image(systemName: "arrow.triangle.2.circlepath")
                            Text("Apply & Reinitialize Wallet")
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .disabled(!isValid)
                }
            }
            .navigationTitle("Connection Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Close") { dismiss() }
                }
            }
            .alert("Error", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage)
            }
        }
    }

    private var isValid: Bool {
        if connectionType == ConnectionType.esplora.rawValue {
            return !esploraHost.isEmpty && !esploraPort.isEmpty && UInt16(esploraPort) != nil
        } else {
            return !electrumHost.isEmpty && !electrumPort.isEmpty && UInt16(electrumPort) != nil
        }
    }

    private func applySettings() {
        do {
            try walletManager.reinitializeWallet()
            dismiss()
        } catch {
            errorMessage = error.localizedDescription
            showError = true
        }
    }
}
