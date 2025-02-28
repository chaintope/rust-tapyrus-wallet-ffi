//
//  exampleApp.swift
//  example
//
//  Created by Kohei Taniguchi on 2025/02/18.
//

import SwiftUI
import TapyrusWallet

@main
struct exampleApp: App {
    // Create a shared wallet manager that will be initialized when the app starts
    @StateObject private var walletManager = TapyrusWalletManager()
    
    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(walletManager)
                .onAppear {
                    // Log app startup
                    print("Tapyrus Wallet App started")
                }
        }
    }
}
