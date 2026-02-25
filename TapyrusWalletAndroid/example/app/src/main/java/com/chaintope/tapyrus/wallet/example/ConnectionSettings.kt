package com.chaintope.tapyrus.wallet.example

enum class ConnectionType(val label: String) {
    ESPLORA("Esplora"),
    ELECTRUM("Electrum");
}

object SettingsKey {
    const val CONNECTION_TYPE = "connectionType"
    const val ESPLORA_HOST = "esploraHost"
    const val ESPLORA_PORT = "esploraPort"
    const val ELECTRUM_HOST = "electrumHost"
    const val ELECTRUM_PORT = "electrumPort"
}

object DefaultConnection {
    const val ESPLORA_HOST = "10.0.2.2"
    const val ESPLORA_PORT = "3001"
    const val ELECTRUM_HOST = "10.0.2.2"
    const val ELECTRUM_PORT = "50001"
}
