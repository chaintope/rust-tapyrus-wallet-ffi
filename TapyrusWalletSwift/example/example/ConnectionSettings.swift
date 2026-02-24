//
//  ConnectionSettings.swift
//  example
//

import Foundation

enum ConnectionType: String, CaseIterable {
    case esplora
    case electrum
}

enum SettingsKey {
    static let connectionType = "connectionType"
    static let esploraHost = "esploraHost"
    static let esploraPort = "esploraPort"
    static let electrumHost = "electrumHost"
    static let electrumPort = "electrumPort"
}

enum DefaultConnection {
    static let esploraHost = "localhost"
    static let esploraPort = "3001"
    static let electrumHost = "localhost"
    static let electrumPort = "50001"
}
