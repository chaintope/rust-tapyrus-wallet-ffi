//
//  TapyrusWalletManager.swift
//  example
//
//  Created on 2025/02/26.
//

import Foundation
import Security
import TapyrusWallet

// Enum for wallet-related errors
enum WalletError: Error {
    case keyGenerationFailed
    case keychainSaveFailed
    case keychainLoadFailed
    case walletInitializationFailed
    case walletSyncFailed
    case addressGenerationFailed
}

// Class to manage HdWallet operations
class TapyrusWalletManager: ObservableObject {
    // Published properties for UI updates
    @Published var currentAddress: String = ""
    @Published var balance: Double = 0.0
    @Published var isSyncing: Bool = false
    @Published var connectionInfo: String?
    @Published var syncResult: SyncResult?

    enum SyncResult {
        case success
        case failure(String)
    }

    // The wallet instance
    private var wallet: HdWallet?

    // Network mode
    private let networkMode: Network = .prod

    // Keychain constants
    private let keychainService = "com.example.TapyrusWalletApp"
    private let masterKeyAccount = "masterKey"

    // Initialize the wallet manager
    init() {
        do {
            try setupWallet()
        } catch {
            print("Failed to setup wallet: \(error)")
        }
    }

    // Setup the wallet - create or load from storage
    private func setupWallet() throws {
        // Check if we have a master key in the keychain
        var masterKey: String

        if let storedMasterKey = loadMasterKeyFromKeychain() {
            // Use the stored master key
            masterKey = storedMasterKey
            print("Using existing master key from keychain")
        } else {
            // Generate a new master key
            masterKey = TapyrusWallet.generateMasterKey(networkMode: networkMode)

            // Save the master key to keychain
            if !saveMasterKeyToKeychain(masterKey) {
                throw WalletError.keychainSaveFailed
            }
            print("Generated and saved new master key")
        }

        // Get the Documents directory path which is more accessible in iOS apps
        let documentsDir = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
        let dbFilePath = documentsDir.appendingPathComponent("tapyrus_wallet.db").path

        // Ensure the directory exists
        try? FileManager.default.createDirectory(at: documentsDir, withIntermediateDirectories: true)

        print("Database path: \(dbFilePath)")

        // Read connection settings from UserDefaults
        let defaults = UserDefaults.standard
        let connectionType = defaults.string(forKey: SettingsKey.connectionType) ?? ConnectionType.esplora.rawValue

        let config: Config

        if connectionType == ConnectionType.electrum.rawValue {
            let host = defaults.string(forKey: SettingsKey.electrumHost) ?? DefaultConnection.electrumHost
            let portStr = defaults.string(forKey: SettingsKey.electrumPort) ?? DefaultConnection.electrumPort
            let port = UInt16(portStr) ?? 50001

            config = Config(
                networkMode: networkMode,
                networkId: 1939510133,
                genesisHash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
                electrumDomain: host,
                electrumPort: port,
                masterKey: masterKey,
                dbFilePath: dbFilePath
            )

            DispatchQueue.main.async { [weak self] in
                self?.connectionInfo = "Electrum: \(host):\(port)"
            }
        } else {
            let host = defaults.string(forKey: SettingsKey.esploraHost) ?? DefaultConnection.esploraHost
            let portStr = defaults.string(forKey: SettingsKey.esploraPort) ?? DefaultConnection.esploraPort
            let esploraUrl = "http://\(host):\(portStr)"

            config = Config(
                networkMode: networkMode,
                networkId: 1939510133,
                genesisHash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
                esploraUrl: esploraUrl,
                masterKey: masterKey,
                dbFilePath: dbFilePath
            )

            DispatchQueue.main.async { [weak self] in
                self?.connectionInfo = "Esplora: \(esploraUrl)"
            }
        }

        // Create a new wallet with the configuration
        do {
            wallet = try HdWallet(config: config)

            // Perform initial sync
            syncWallet()
        } catch {
            print("Error initializing wallet: \(error)")
            throw WalletError.walletInitializationFailed
        }
    }

    // Reinitialize the wallet with current settings
    func reinitializeWallet() throws {
        wallet = nil

        DispatchQueue.main.async { [weak self] in
            self?.currentAddress = ""
            self?.balance = 0.0
        }

        try setupWallet()
    }

    // Save master key to keychain
    private func saveMasterKeyToKeychain(_ key: String) -> Bool {
        guard let data = key.data(using: .utf8) else {
            return false
        }

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: keychainService,
            kSecAttrAccount as String: masterKeyAccount,
            kSecValueData as String: data
        ]

        // Delete any existing key before saving
        SecItemDelete(query as CFDictionary)

        // Add the new key
        let status = SecItemAdd(query as CFDictionary, nil)
        return status == errSecSuccess
    }

    // Load master key from keychain
    private func loadMasterKeyFromKeychain() -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: keychainService,
            kSecAttrAccount as String: masterKeyAccount,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]

        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)

        if status == errSecSuccess, let data = result as? Data, let masterKey = String(data: data, encoding: .utf8) {
            return masterKey
        }
        return nil
    }

    // Sync the wallet with the blockchain
    func syncWallet() {
        guard let wallet = wallet else { return }

        DispatchQueue.global(qos: .background).async { [weak self] in
            DispatchQueue.main.async {
                self?.isSyncing = true
                self?.syncResult = nil
            }

            do {
                // Perform full sync
                try wallet.fullSync()

                // Update balance
                self?.updateBalance()

                DispatchQueue.main.async {
                    self?.isSyncing = false
                    self?.syncResult = .success
                }
            } catch {
                print("Sync error: \(error)")
                DispatchQueue.main.async {
                    self?.isSyncing = false
                    self?.syncResult = .failure("\(error)")
                }
            }
        }
    }

    // Generate a new address
    func getNewAddress() -> String {
        guard let wallet = wallet else { return "" }
        debugPrint(wallet)

        do {
            // Call the wallet's getNewAddress function with colorId set to nil
            let result = try wallet.getNewAddress(colorId: nil)
            let address = result.address

            DispatchQueue.main.async { [weak self] in
                self?.currentAddress = address
            }

            return address
        } catch {
            print("Error generating address: \(error)")
            return ""
        }
    }

    // Update the wallet balance
    private func updateBalance() {
        guard let wallet = wallet else { return }

        do {
            // Call the wallet's balance function with colorId set to nil for TPC
            let balanceValue = try wallet.balance(colorId: nil)

            // Convert to Double for display (balance is in satoshis, convert to TPC)
            let tpcBalance = Double(balanceValue) / 100_000_000.0

            DispatchQueue.main.async { [weak self] in
                self?.balance = tpcBalance
            }
        } catch {
            print("Error getting balance: \(error)")
        }
    }

    // Transfer TPC to another address
    func transfer(toAddress: String, amount: Double) async throws -> String {
        guard let wallet = wallet else {
            throw WalletError.walletInitializationFailed
        }

        // Convert amount from TPC to satoshis (smallest unit)
        let amountInSatoshis = UInt64(amount * 100_000_000.0)

        // Create transfer parameters
        let transferParams = TransferParams(amount: amountInSatoshis, toAddress: toAddress)

        // Execute the transfer
        let txid = try wallet.transfer(params: [transferParams], utxos: [])

        // Update balance after transfer
        updateBalance()

        return txid
    }
}
