use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod crypto;
mod storage;

use crypto::TimeLockedMessage;


#[derive(Parser)]
#[command(name = "timecapsule")]
#[command(about = "A time capsule for your messages. encrypt content that can only be decrypted after a specific date")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lock a message until a specific date
    Lock {
        /// Message to lock (or use --file to read from file)
        #[arg(short, long)]
        message: Option<String>,
        
        /// File to read message from
        #[arg(short, long)]
        file: Option<PathBuf>,
        
        /// Unlock date (e.g., "2024-12-25", "2024-12-25 15:30:00")
        #[arg(short, long)]
        date: String,
        
        /// Optional label for the message
        #[arg(short, long)]
        label: Option<String>,
        
        /// Output file (optional, defaults to storage directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Try to unlock a message
    Unlock {
        /// Message ID or file path
        #[arg(short, long)]
        id: Option<String>,
        
        /// File path to unlock
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
    /// List all locked messages
    List,
    /// Check if any messages are ready to unlock
    Check,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lock { message, file, date, label, output } => {
            let content = get_message_content(message, file)?;
            let unlock_date = parse_date(&date)?;
            
            if unlock_date <= Utc::now() {
                return Err(anyhow!("Unlock date must be in the future"));
            }
            
            let password = rpassword::prompt_password("Enter password to encrypt the message: ")?;
            if password.trim().is_empty() {
                return Err(anyhow!("Password cannot be empty"));
            }
            
            let locked_msg = TimeLockedMessage::new(&content, &password, unlock_date, label)?;
            
            let id = if let Some(output_path) = output {
                storage::save_to_file(&locked_msg, &output_path)?;
                output_path.file_stem().unwrap().to_string_lossy().to_string()
            } else {
                storage::save_message(&locked_msg)?
            };
            
            println!("✅ Message locked successfully!");
            println!("📦 Message ID: {}", id);
            println!("🔓 Unlock date: {}", unlock_date.format("%Y-%m-%d %H:%M:%S UTC"));
            println!("⏰ Time remaining: {}", format_duration(unlock_date - Utc::now()));
        }
        
        Commands::Unlock { id, file } => {
            let locked_msg = if let Some(file_path) = file {
                storage::load_from_file(&file_path)?
            } else if let Some(message_id) = id {
                storage::load_message(&message_id)?
            } else {
                return Err(anyhow!("Must specify either --id or --file"));
            };
            
            if locked_msg.unlock_date > Utc::now() {
                println!("🔒 Message is still locked!");
                println!("🔓 Unlock date: {}", locked_msg.unlock_date.format("%Y-%m-%d %H:%M:%S UTC"));
                println!("⏰ Time remaining: {}", format_duration(locked_msg.unlock_date - Utc::now()));
                return Ok(());
            }
            
            let password = rpassword::prompt_password("Enter password to decrypt the message: ")?;
            
            match locked_msg.unlock(&password) {
                Ok(content) => {
                    println!("🎉 Message unlocked successfully!");
                    println!("📄 Content:");
                    println!("{}", "=".repeat(50));
                    println!("{}", content);
                    println!("{}", "=".repeat(50));
                }
                Err(e) => {
                    println!("❌ Failed to unlock message: {}", e);
                }
            }
        }
        
        Commands::List => {
            let messages = storage::list_messages()?;
            if messages.is_empty() {
                println!("📭 No locked messages found");
                return Ok(());
            }
            
            println!("📦 Locked Messages:");
            println!("{}", "=".repeat(80));
            for (id, msg) in messages {
                let status = if msg.unlock_date <= Utc::now() { "🔓 READY" } else { "🔒 LOCKED" };
                let label = msg.label.as_deref().unwrap_or("(no label)");
                println!("ID: {} | {} | {} | {}", 
                    id, 
                    status, 
                    msg.unlock_date.format("%Y-%m-%d %H:%M UTC"),
                    label
                );
            }
        }
        
        Commands::Check => {
            let messages = storage::list_messages()?;
            let ready_messages: Vec<_> = messages.into_iter()
                .filter(|(_, msg)| msg.unlock_date <= Utc::now())
                .collect();
            
            if ready_messages.is_empty() {
                println!("⏰ No messages are ready to unlock yet");
            } else {
                println!("🎉 {} message(s) are ready to unlock:", ready_messages.len());
                for (id, msg) in ready_messages {
                    let label = msg.label.as_deref().unwrap_or("(no label)");
                    println!("  📦 {}: {}", id, label);
                }
                println!("\nUse 'timelock unlock --id <ID>' to unlock them");
            }
        }
    }
    
    Ok(())
}

fn get_message_content(message: Option<String>, file: Option<PathBuf>) -> Result<String> {
    match (message, file) {
        (Some(msg), None) => Ok(msg),
        (None, Some(path)) => {
            fs::read_to_string(&path)
                .map_err(|e| anyhow!("Failed to read file {:?}: {}", path, e))
        }
        (Some(_), Some(_)) => Err(anyhow!("Cannot specify both --message and --file")),
        (None, None) => {
            println!("Enter your message (press Ctrl+D when done):");
            use std::io::{self, Read};
            let mut buffer = String::new();
            io::stdin().read_to_string(&mut buffer)?;
            Ok(buffer.trim().to_string())
        }
    }
}

fn parse_date(date_str: &str) -> Result<DateTime<Utc>> {
    // Try different date formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d",
    ];
    
    for format in &formats {
        if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
            return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
        }
    }
    
    // Try date only format
    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let naive_dt = naive_date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
    }
    
    Err(anyhow!("Invalid date format. Use: YYYY-MM-DD or 'YYYY-MM-DD HH:MM:SS'"))
}

fn format_duration(duration: chrono::Duration) -> String {
    let days = duration.num_days();
    let hours = duration.num_hours() % 24;
    let minutes = duration.num_minutes() % 60;
    
    if days > 0 {
        format!("{} days, {} hours, {} minutes", days, hours, minutes)
    } else if hours > 0 {
        format!("{} hours, {} minutes", hours, minutes)
    } else {
        format!("{} minutes", minutes)
    }
}
