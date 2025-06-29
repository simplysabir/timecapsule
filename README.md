# timecapsule

A command-line tool for creating time-locked encrypted messages that can only be decrypted after a specified date.

## Installation

```bash
cargo install timecapsule
```

Or build from source:
```bash
git clone https://github.com/simplysabir/timecapsule.git
cd timecapsule
cargo install --path .
```

## Usage

### Lock a message
```bash
# Text message
timecapsule lock -m "Remember to call mom" -d "2025-12-25" -l "Christmas Reminder"

# From file
timecapsule lock -f diary.txt -d "2025-01-01" -l "Year End Thoughts"

# Interactive input (no -m or -f flags)
timecapsule lock -d "2025-06-15" -l "Summer Goals"
```

### Manage capsules
```bash
# List all capsules
timecapsule list

# Check which capsules are ready to unlock
timecapsule check

# Unlock a capsule (only works after the unlock date)
timecapsule unlock --id <capsule-id>
timecapsule unlock --file path/to/capsule.json
```

### Date formats
- `2025-12-25` (unlocks at midnight UTC)
- `2025-12-25 15:30:00`
- `2025-12-25 15:30`

## Options

### `lock` command
- `-m, --message <TEXT>` - Message to encrypt
- `-f, --file <PATH>` - File to encrypt
- `-d, --date <DATE>` - Unlock date (required)
- `-l, --label <LABEL>` - Optional label for identification
- `-o, --output <PATH>` - Custom output file (default: ~/.timecapsule/)

### `unlock` command
- `--id <ID>` - Capsule ID from storage
- `--file <PATH>` - Path to capsule file

## How the Modules Work Together

This project is split into three main modules that work together to create and manage time-locked messages: `main.rs`, `crypto.rs`, and `storage.rs`.

### `main.rs` - The Conductor

*   **Role**: This is the entry point of the application. It's responsible for parsing command-line arguments, handling user input, and orchestrating the other modules.
*   **Interaction**: When you run a command like `timecapsule lock`, `main.rs` gathers the message, password, and unlock date from you. It then calls `crypto.rs` to perform the encryption and, once it receives the encrypted `TimeLockedMessage` object, it passes it to `storage.rs` to be saved to a file. The reverse happens for unlocking.

### `crypto.rs` - The Vault

*   **Role**: This module is the cryptographic engine. It knows nothing about files or command-line arguments; its sole purpose is to securely encrypt and decrypt data.
*   **Process**:
    1.  **Key Derivation**: It uses the **Argon2** algorithm to turn a user's password into a strong, secure encryption key. It uses a unique, random **salt** for each message to ensure that even identical passwords result in different keys.
    2.  **Encryption**: It uses **AES-256-GCM**, a modern and secure encryption standard, to encrypt the message content. This method provides both confidentiality (the message is unreadable) and integrity (the message cannot be secretly tampered with).
    3.  **Packaging**: All the necessary components—the encrypted content, the salt used for key derivation, and a unique value called a **nonce**—are packaged into a `TimeLockedMessage` struct, ready to be stored.

### `storage.rs` - The Archivist

*   **Role**: This module handles all interactions with the filesystem. It is responsible for saving, loading, listing, and deleting the encrypted message files.
*   **Process**: It takes the `TimeLockedMessage` struct from the `crypto` module and serializes it into a human-readable JSON format. It then saves this JSON to a file with a unique ID in a dedicated application directory (`~/.timecapsule/`). This separation of concerns means the storage method could be changed in the future (e.g., to a database) without altering the cryptographic or main logic.

## Security

- **Encryption**: AES-256-GCM with cryptographically secure random nonces
- **Key derivation**: Argon2 password hashing with random salts
- **Integrity**: Built-in authentication prevents tampering
- **Time lock**: Date verification prevents early access

**Note**: This provides access control, not cryptographic time-lock puzzles. The time check can be bypassed by modifying the source code.

## Storage

Capsules are stored in:
- Linux/macOS: `~/.timecapsule/`
- Windows: `%USERPROFILE%\.timecapsule\`

Each capsule is a JSON file containing encrypted content, metadata, and unlock date.

## Examples

```bash
# Personal reminder
timecapsule lock -m "Did you achieve your goals?" -d "2025-12-31" -l "Year Review"

# Project milestone
timecapsule lock -f project-notes.md -d "2025-06-01" -l "Project Retrospective"

# Gift message (save to file to share)
timecapsule lock -m "Happy graduation!" -d "2025-05-15" -o graduation.json

# Check what's ready
timecapsule check

# Unlock when ready
timecapsule unlock --id abc123...
```

## Building

```bash
cargo build --release
cargo test
```

## License

Licensed under either MIT or Apache-2.0 at your option.

## Contributing

Pull requests welcome. Please ensure tests pass and follow existing code style.