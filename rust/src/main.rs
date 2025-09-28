use clap::{Parser, Subcommand};
use lumo::database::Database;
use lumo::transaction::{ConfirmationStatus, TransactionDirection};
use lumo::{init, Amount, FeeRate, Network, Wallet};

#[derive(Parser)]
#[command(name = "lumo")]
#[command(about = "A Bitcoin wallet CLI for testing")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new wallet
    CreateWallet {
        /// Name of the wallet
        name: String,
        /// Bitcoin network (testnet or mainnet)
        #[arg(long, default_value = "testnet")]
        network: String,
        /// Create wallet from existing mnemonic
        #[arg(long)]
        from_mnemonic: Option<String>,
    },
    /// List all wallets
    ListWallets {
        /// Filter by network
        #[arg(long)]
        network: Option<String>,
    },
    /// Select a wallet for operations
    SelectWallet {
        /// Name of wallet to select
        name: String,
    },
    /// Get a receiving address
    GetAddress {
        /// Address index
        #[arg(long)]
        index: Option<u32>,
    },
    /// Get wallet balance
    GetBalance {
        // Display units (sats, btc)
        #[arg(long, default_value = "sats")]
        unit: String,
    },
    /// Show transaction history
    ShowHistory {
        #[arg(long, default_value = "sats")]
        unit: String,
    },
    /// Send a transaction
    SendTransaction {
        /// Recipient address
        address: String,
        /// Amount in satoshis
        amount: u64,
        /// Fee rate in sat/vB
        #[arg(long, default_value = "10")]
        fee_rate: f32,
    },
    /// Generate a new mnemonic
    GenerateMnemonic,
}

fn format_amount(amount: lumo::Amount, unit: &str) -> String {
    match unit.to_lowercase().as_str() {
        "btc" => format!("{} BTC", amount.as_btc()),
        "sats" => format!("{} sats", amount.as_sat()),
        _ => format!("{} sats", amount.as_sat()),
    }
}

fn parse_network(network_str: &str) -> Result<Network, String> {
    match network_str.to_lowercase().as_str() {
        "mainnet" => Ok(Network::Mainnet),
        "testnet" => Ok(Network::Testnet),
        "testnet4" => Ok(Network::Testnet4),
        "signet" => Ok(Network::Signet),
        "regtest" => Ok(Network::Regtest),
        _ => Err(format!(
            "Invalid network: {}. Valid options: mainnet, testnet, testnet4, signet, regtest",
            network_str
        )),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the library
    init()?;

    let cli = Cli::parse();

    match cli.command {
        Commands::CreateWallet {
            name,
            network,
            from_mnemonic,
        } => {
            let network = parse_network(&network)?;

            let wallet = if let Some(mnemonic) = from_mnemonic {
                println!("Creating wallet: {}", name);
                Wallet::new_from_mnemonic(name, &mnemonic, network)?
            } else {
                println!("Creating wallet: {}", name);
                let (wallet, mnemonic) = Wallet::new_random(name, network)?;
                println!();
                println!("🔑 RECOVERY PHRASE (WRITE THIS DOWN!):");
                println!();
                println!("{}", mnemonic);
                println!();
                println!("⚠️  IMPORTANT: Save these words in a secure location!");
                println!();
                wallet
            };

            println!("✅ Wallet created successfully: {}", wallet.id);
            println!("   Name: {}", wallet.name());
            println!("   ID: {}", wallet.id);
            println!("   Network: {}", wallet.network());
            if let Some(fingerprint) = &wallet.metadata.master_fingerprint {
                println!("   Fingerprint: {}", fingerprint);
            }
        }
        Commands::ListWallets { network } => {
            println!("Listing wallets");
            let filter_network = if let Some(net_str) = network {
                Some(parse_network(&net_str)?)
            } else {
                None
            };

            let wallets = Wallet::list_all(filter_network)?;

            if wallets.is_empty() {
                println!("No wallets found");
            } else {
                println!("Found {} wallets:", wallets.len());
                for (i, wallet) in wallets.iter().enumerate() {
                    println!("{}. {}", i + 1, wallet.name);
                    println!("    Network: {}", wallet.network);
                    println!("    ID: {}", wallet.id);
                    if let Some(fingerprint) = &wallet.master_fingerprint {
                        println!("    Fingerprint: {}", fingerprint);
                    }
                    println!();
                }
            }
        }
        Commands::SelectWallet { name } => {
            let wallets = Wallet::list_all(None)?;
            let wallet = wallets.iter().find(|w| w.name == name);

            match wallet {
                Some(wallet_meta) => {
                    let database = Database::global();
                    database.global_config.select_wallet(&wallet_meta.id)?;

                    println!("✅ Selected wallet: {}", wallet_meta.name);
                    println!("   ID: {}", wallet_meta.id);
                    println!("   Network: {}", wallet_meta.network);
                    if let Some(fp) = &wallet_meta.master_fingerprint {
                        println!("   Fingerprint: {}", fp);
                    }
                }
                None => {
                    println!("❌ Wallet not found: {}", name);
                    println!("\nAvailable wallets:");
                    for wallet in &wallets {
                        println!("{}. {}", wallet.id, wallet.name);
                    }
                }
            }
        }
        Commands::GetAddress { index } => {
            let database = Database::global();
            let selected_id = database.global_config.selected_wallet()?;

            match selected_id {
                Some(wallet_id) => {
                    let wallets = Wallet::list_all(None)?;
                    let wallet_meta = wallets.iter().find(|w| w.id == wallet_id);

                    if let Some(meta) = wallet_meta {
                        // Load actual wallet
                        let wallet = Wallet::try_load_persisted(&wallet_id, meta.network)?;

                        let (address, description) = match index {
                            Some(idx) => {
                                let addr = wallet.address_at(idx)?;
                                (addr, format!("Address at index {}", idx))
                            }
                            None => {
                                let addr = wallet.get_current_address()?;
                                (addr, "Current receiving address".to_string())
                            }
                        };

                        println!("📍 {}: {}", description, address);
                        println!("   Wallet: {}", meta.name);
                        println!("   Network: {}", meta.network);
                    } else {
                        println!("❌ Selected wallet not found: {}", wallet_id);
                    }
                }
                None => {
                    println!("❌ No wallet selected. Use 'select-wallet' command first.");
                }
            }
        }
        Commands::GetBalance { unit } => {
            let database = Database::global();
            let selected_id = database.global_config.selected_wallet()?;

            match selected_id {
                Some(wallet_id) => {
                    let wallets = Wallet::list_all(None)?;
                    let wallet_meta = wallets.iter().find(|w| w.id == wallet_id);

                    if let Some(meta) = wallet_meta {
                        let mut wallet = Wallet::try_load_persisted(&wallet_id, meta.network)?;

                        // Auto sync wallet
                        println!("🔄 Syncing with blockchain...");
                        wallet.sync().await?;

                        let balance = wallet.balance();

                        println!(
                            "💰 Wallet balance: {}",
                            format_amount(balance.spendable(), &unit)
                        );
                        println!(
                            "   Spendable: {}",
                            format_amount(balance.spendable(), &unit)
                        );
                        println!(
                            "   Confirmed: {}",
                            format_amount(balance.confirmed(), &unit)
                        );
                        println!("   Wallet: {}", meta.name);
                        println!("   Network: {}", meta.network);
                    } else {
                        println!("❌ Selected wallet not found: {}", wallet_id);
                    }
                }
                None => {
                    println!("❌ No wallet selected. Use 'select-wallet' command first.");
                }
            }
        }
        Commands::ShowHistory { unit } => {
            let database = Database::global();
            let selected_id = database.global_config.selected_wallet()?;

            match selected_id {
                Some(wallet_id) => {
                    let wallets = Wallet::list_all(None)?;
                    let wallet_meta = wallets.iter().find(|w| w.id == wallet_id);

                    if let Some(meta) = wallet_meta {
                        let mut wallet = Wallet::try_load_persisted(&wallet_id, meta.network)?;

                        // Auto-sync for latest transactions
                        println!("🔄 Syncing with blockchain...");
                        wallet.sync().await?;

                        let transactions = wallet.transactions()?;

                        if transactions.is_empty() {
                            println!("📝 No transactions found");
                        } else {
                            println!(
                                "📝 Transaction History ({} transactions):",
                                transactions.len()
                            );
                            println!();

                            for (i, tx) in transactions.iter().enumerate() {
                                let direction = match tx.direction {
                                    TransactionDirection::Incoming => "📥 Received",
                                    TransactionDirection::Outgoing => "📤 Sent",
                                    TransactionDirection::SelfTransfer => "🔄 Self Transfer",
                                };

                                println!("{}. {} {}", i + 1, direction, format_amount(tx.amount, &unit));
                                match tx.direction {
                                    TransactionDirection::Outgoing => {
                                        if let Some(fee) = &tx.fee {
                                            let recipient_amount = tx.amount.as_sat().saturating_sub(fee.as_sat());
                                            println!("   ├── To recipient: {} {}", recipient_amount, if unit == "sats" { "sats" } else { "BTC" });
                                            println!("   ├── Network fee: {}", format_amount(*fee, &unit));
                                        }
                                    }
                                    TransactionDirection::SelfTransfer => {
                                        println!("   ├── Transfer fee: {}", format_amount(tx.amount, &unit));
                                        println!("   ├── (Sent to your own address)");
                                    }
                                    TransactionDirection::Incoming => {
                                        // No additional details needed for received transactions
                                    }
                                }

                                // Truncated TXID
                                let txid_str = tx.id.to_string();
                                let short_txid = if txid_str.len() > 16 {
                                    format!("{}...{}", &txid_str[0..8], &txid_str[txid_str.len()-8..])
                                } else {
                                    txid_str
                                };
                                println!("   └── TXID: {}", short_txid);

                                // Better status display
                                let status = match &tx.confirmation_status {
                                    ConfirmationStatus::Unconfirmed => "Pending".to_string(),
                                    ConfirmationStatus::Confirmed { block_height } => format!("Confirmed (Block {})", block_height),
                                };
                                println!("   Status: {}", status);
                                println!();
                            }
                        }

                        println!("   Wallet: {}", meta.name);
                        println!("   Network: {}", meta.network);
                    } else {
                        println!("❌ Selected wallet not found: {}", wallet_id);
                    }
                }
                None => {
                    println!("❌ No wallet selected. Use 'select-wallet' command first.");
                }
            }
        }
        Commands::SendTransaction {
            address,
            amount,
            fee_rate,
        } => {
            let database = Database::global();
            let selected_id = database.global_config.selected_wallet()?;

            match selected_id {
                Some(wallet_id) => {
                    let wallets = Wallet::list_all(None)?;
                    let wallet_meta = wallets.iter().find(|w| w.id == wallet_id);

                    if let Some(meta) = wallet_meta {
                        let mut wallet = Wallet::try_load_persisted(&wallet_id, meta.network)?;

                        // Auto-sync for latest UTXOs
                        println!("🔄 Syncing with blockchain...");
                        wallet.sync().await?;

                        // Parse recipient address
                        let recipient = lumo::Address::from_string(&address, meta.network)?;
                        let send_amount = Amount::from_sat(amount);
                        let fee_rate = FeeRate::from_sat_per_vb(fee_rate);

                        println!("💸 Sending Transaction:");
                        println!("   To: {}", address);
                        println!("   Amount: {} sats", amount);
                        println!("   Fee Rate: {}", fee_rate);
                        println!("   From: {}", meta.name);

                        // Build transaction
                        println!("🔨 Building transaction...");
                        let psbt = wallet.build_transaction(recipient, send_amount, fee_rate)?;

                        // Add this debug section:
                        println!("📋 PSBT Debug Info:");
                        println!("   Inputs: {}", psbt.inputs.len());
                        println!("   Outputs: {}", psbt.outputs.len());
                        if let Ok(fee) = psbt.fee() {
                            println!("   Fee: {} sats", fee.to_sat());
                        }

                        // Sign transaction
                        println!("✍️ Signing transaction...");
                        let signed_tx = wallet.sign_transaction(psbt)?;

                        // Get TXID before broadcasting
                        let txid = signed_tx.compute_txid();

                        // Broadcast transaction
                        println!("📡 Broadcasting to network...");
                        wallet.broadcast_transaction(signed_tx).await?;

                        println!("✅ Transaction sent successfully!");
                        println!("   TXID: {}", txid);
                        println!(
                            "   View on mempool.space: https://mempool.space/testnet/tx/{}",
                            txid
                        );
                    } else {
                        println!("❌ Selected wallet not found: {}", wallet_id);
                    }
                }
                None => {
                    println!("❌ No wallet selected. Use 'select-wallet' command first.");
                }
            }
        }
        Commands::GenerateMnemonic => {
            println!("Generating new mnemonic");
            // TODO: Implement mnemonic generation
        }
    }

    Ok(())
}
