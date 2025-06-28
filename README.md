# TimeCapsule

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

## How it works

1. **Encryption**: Messages are encrypted using AES-256-GCM with Argon2 password hashing
2. **Time lock**: The tool checks the current date before allowing decryption
3. **Storage**: Capsules are stored as JSON files in `~/.timecapsule/` (or custom location)
4. **Portability**: Capsule files are self-contained and can be copied anywhere

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