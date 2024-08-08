using System;
using com.chaintope.tapyrus.wallet;


public class WalletExample
{
    public static void Main()
    {
        // Initialize the wallet configuration
        var config = new Config(
            Network.Prod,
            1939510133,
            "038b114875c2f78f5a2fd7d8549a905f38ea5faee6e29a3d79e547151d6bdd8a",
            "localhost",
            3001,
            null,
            null,
            null,
            "master_key",
            "wallet.sqlite");

        // Create a new wallet instance
        var wallet = new HdWallet(config);

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