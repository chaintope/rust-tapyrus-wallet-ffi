# Getting Started

## Overview of the Tapyrus Blockchain Wallet

The Tapyrus Blockchain Wallet is a secure and user-friendly wallet designed for managing Tapyrus blockchain assets. It supports various features such as creating new addresses, transferring Tapyrus coins (TPC), and managing colored coins. The wallet is built with a focus on security and ease of use, making it suitable for both beginners and advanced users.

## Esplora Tapyrus

Esplora Tapyrus is a blockchain index server that provides an API for querying blockchain data and broadcasting transactions. It is essential for syncing the wallet with the blockchain and ensuring that all transactions are properly indexed and accessible.

### Setting Up Tapyrus Core and Esplora-Tapyrus Using Docker Compose

#### 1. **Create a `docker-compose.yml` file** with the following content:

```yaml
version: '3'
services:
  tapyrusd:
    image: tapyrus/tapyrusd:latest
    environment:
      GENESIS_BLOCK_WITH_SIG: '01000000000000000000000000000000000000000000000000000000000000000000000044cc181bd0e95c5b999a13d1fc0d193fa8223af97511ad2098217555a841b3518f18ec2536f0bb9d6d4834fcc712e9563840fe9f089db9e8fe890bffb82165849f52ba5e01210366262690cbdf648132ce0c088962c6361112582364ede120f3780ab73438fc4b402b1ed9996920f57a425f6f9797557c0e73d0c9fbafdebcaa796b136e0946ffa98d928f8130b6a572f83da39530b13784eeb7007465b673aa95091619e7ee208501010000000100000000000000000000000000000000000000000000000000000000000000000000000000ffffffff0100f2052a010000002776a92231415132437447336a686f37385372457a4b6533766636647863456b4a74356e7a4188ac00000000'
    volumes:
      - tapyrus_data:/var/lib/tapyrus
      - ./tapyrus.conf:/etc/tapyrus/tapyrus.conf
    ports:
      - '2377:2377'
  esplora:
    image: tapyrus/esplora-tapyrus:latest
    user: root
    volumes:
      - electrs_data:/var/lib/electrs
      - tapyrus_data:/var/lib/tapyrus:ro
    ports:
      - '3001:3001'
    depends_on:
      - tapyrusd
    command:
      [
        'electrs',
        '-vvvv',
        '--timestamp',
        '--db-dir',
        '/var/lib/electrs',
        '--daemon-rpc-addr',
        'tapyrusd:2377',
        '--daemon-dir',
        '/var/lib/tapyrus/prod-1939510133',
        '--network-id',
        '1939510133',
        '--network',
        'prod',
        '--http-addr',
        '0.0.0.0:3001',
        '--cookie',
        'rpcuser:rpcpassword'
      ]
volumes:
  tapyrus_data:
  electrs_data:
```

#### 2. **Create a `tapyrus.conf` file** with the following content:

```conf
networkid=1939510133
txindex=1
server=1
rest=1
rpcuser=rpcuser
rpcpassword=rpcpassword
rpcbind=0.0.0.0
rpcallowip=127.0.0.1
rpcallowip=0.0.0.0/0
addseeder=static-seed.tapyrus.dev.chaintope.com
fallbackfee=0.00001
```

3. **Run Docker Compose** to start the services:

    ```sh
    docker-compose up -d
    ```

This setup will start Tapyrus Core and Esplora-Tapyrus, making them available for use with your wallet.

### Tapyrus Testnet

The above Docker Compose configuration connects `tapyrusd` and `esplora-tapyrus` to the Tapyrus testnet. The testnet is a separate blockchain used for development and testing purposes. It allows developers to experiment and test their applications without using real Tapyrus coins. The testnet is ideal for development as it mimics the main network's behavior but with no real-world value.

### Getting TPC from the Faucet

You can get a small amount of TPC for testing purposes from the Tapyrus testnet faucet. Visit [https://testnet-faucet.tapyrus.dev.chaintope.com/](https://testnet-faucet.tapyrus.dev.chaintope.com/) and enter your Tapyrus testnet address to receive TPC.

## Difference Between `HDWallet.Sync` and `HDWallet.FullSync`

- **`HDWallet.Sync`**: This method performs a quick synchronization with the blockchain, updating only the most recent transactions. It is suitable for daily operations to keep your wallet updated with the latest transactions. Note that `Sync` only scans addresses revealed by `GetNewAddress()`. If you import the same master key into multiple wallets, `Sync` might not be enough to synchronize because another wallet might reveal more addresses and receive transactions to those addresses.
- **`HDWallet.FullSync`**: This method performs a complete synchronization, scanning the entire blockchain for all transactions related to your wallet. It is useful when setting up a new wallet, restoring from a backup, or if you suspect that some transactions might be missing. During the synchronization process, the wallet will stop scanning for new addresses after encountering 25 consecutive unused addresses (stop gap). This helps to optimize the synchronization process by avoiding unnecessary checks for addresses that are unlikely to have transactions.

## Simple TPC Transfer Sample Code

Here is a simple example of getting TPC from the faucet and transferring it back to the faucet return address:

```csharp
using System;
using com.chaintope.tapyrus.wallet;

public class WalletExample
{
    public static void Main()
    {
        var masterKey = WalletMethods.GenerateMasterKey(Network.Prod);

        // Initialize the wallet configuration
        var config = new Config(
            networkMode: Network.Prod,
            networkId: 1939510133,
            genesisHash: "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
            esploraUrl: "http://localhost:3001",
            masterKey: masterKey,
            dbFilePath: "wallet.sqlite");

        // Create a new wallet instance
        var wallet = new HdWallet(config: config);

        // Perform a full sync to ensure the wallet is up-to-date
        wallet.FullSync();

        // Get a new address to receive TPC from the faucet
        var address = wallet.GetNewAddress(null);
        Console.WriteLine("Request TPC from the faucet (https://testnet-faucet.tapyrus.dev.chaintope.com) to this address: " + address);

        // Wait for the user to get TPC from the faucet
        Console.WriteLine("Press Enter after receiving TPC from the faucet...");
        Console.ReadLine();

        // Sync the wallet to update the balance
        wallet.Sync();

        // Show the balance after syncing
        var balance = wallet.Balance(null);
        Console.WriteLine("Balance after receiving TPC from the faucet: " + balance);

        // Transfer 1000 TPC to the faucet return address
        var faucetReturnAddress = "1LxWufmUothBSe78DYESKcoP8ppmPcSHZ6";
        var transferParams = new TransferParams(1000, faucetReturnAddress);
        var txid = wallet.Transfer([transferParams], []);

        // Output the transaction ID
        Console.WriteLine("Transaction ID: " + txid);
        Console.WriteLine("You can find the transaction details in https://testnet-explorer.tapyrus.dev.chaintope.com/tx/" + txid);
    }
}
```