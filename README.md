# PFP Checker 🔎

<div align="center">

![GitHub release (latest by date)](https://img.shields.io/github/v/release/j4ytr1n1ty/pfp-checker)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/j4ytr1n1ty/pfp-checker/rust.yml)
![License](https://img.shields.io/github/license/j4ytr1n1ty/pfp-checker)
![Rust Version](https://img.shields.io/badge/rust-1.86+-orange.svg)
![Discord](https://img.shields.io/badge/Discord-Bot-7289DA)

</div>

> **Do you have friends on Discord who change their profile pictures way too often? Ever wondered how many times they change their profile pictures or server icons? This bot will be your _blazingly fast_ solution.** 🚀

PFP Checker is a Discord bot that tracks and archives users' profile pictures, username history, and server icons, providing statistics and detailed historical records of changes.

## ✨ Features

- 📊 **Comprehensive Tracking**: Monitor profile pictures, usernames, and server icons
- 📅 **Historical Records**: View the complete history of changes
- 📈 **Advanced Statistics**: Track frequency of changes with averages
- 🔍 **User & Server Insights**: Understand patterns in profile and server updates
- 🔄 **Automatic Updates**: Regular checking for changes
- 🐳 **Docker Support**: Easy deployment with Docker and Docker Compose

## 🚀 Quick Start

### Running with Docker (Recommended)

```bash
# Create a directory for the bot
mkdir pfp-checker && cd pfp-checker

# Download the production compose file
curl -o docker-compose.yml https://raw.githubusercontent.com/j4ytr1n1ty/pfp-checker/main/docker-compose.production.yml

# Create and configure your .env file
cat > .env << 'EOF'
DISCORD_TOKEN=your_discord_token_here
IMGBB_API_KEY=your_imgbb_api_key_here
EOF
# Edit the .env file with your actual credentials

# Start the bot
docker compose up -d
```

### Manual Installation

```bash
# Clone the repository
git clone https://github.com/j4ytr1n1ty/pfp-checker.git
cd pfp-checker

# Install SQLx CLI for database migrations
cargo install sqlx-cli

# Set up the database
sqlx database setup --database-url sqlite:database.sqlite

# Create and configure your .env file
cp .env.example .env
# Edit the .env file with your Discord token and ImgBB API key

# Build and run the project
cargo build --release
./target/release/pfp-checker
```

## 📋 Available Commands

### User Tracking

| Command                  | Description                                            |
| ------------------------ | ------------------------------------------------------ |
| `/monitor @user`         | Start tracking a user's profile picture and username   |
| `/removemonitor @user`   | Stop tracking a user                                   |
| `/pfphistory @user`      | View a user's profile picture history                  |
| `/usernamehistory @user` | View a user's username history                         |
| `/stats @user`           | Show statistics about a user's profile picture changes |

### Server Tracking

| Command                | Description                                      |
| ---------------------- | ------------------------------------------------ |
| `/monitorserver`       | Start tracking this server's icon changes        |
| `/removemonitorserver` | Stop tracking this server's icon changes         |
| `/serverpfphistory`    | View this server's icon history                  |
| `/serverstats`         | Show statistics about this server's icon changes |

### General

| Command | Description                |
| ------- | -------------------------- |
| `/ping` | Check if the bot is online |

## 🧰 Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.86 or newer)
- [SQLite](https://www.sqlite.org/download.html)
- Discord Bot Token
- ImgBB API Key

### Setup Steps

1. Install the SQLx CLI:

   ```bash
   cargo install sqlx-cli
   ```

2. Clone the repository:

   ```bash
   git clone https://github.com/j4ytr1n1ty/pfp-checker.git
   cd pfp-checker
   ```

3. Create your configuration:

   ```bash
   cp .env.example .env
   # Edit .env with your Discord token and ImgBB API key
   ```

4. Set up the database:

   ```bash
   sqlx database setup --database-url sqlite:database.sqlite
   ```

5. Build and run in development mode:

   ```bash
   cargo run
   ```

For more detailed information about contributing to the project, please see [CONTRIBUTING.md](CONTRIBUTING.md).

## 📦 Deployment

### Docker Compose (Production)

```bash
# Using the production configuration
docker-compose -f docker-compose.production.yml up -d
```

This will:

- Start the bot container
- Set up volume persistence for the database
- Configure Watchtower for automatic updates

## 📄 Documentation

- [Changelog](CHANGELOG.md)
- [Roadmap](ROADMAP.md)
- [Contributing Guidelines](CONTRIBUTING.md)

## 📝 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgements

- [Serenity](https://github.com/serenity-rs/serenity) - A Rust library for the Discord API
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL for Rust
- [ImgBB](https://imgbb.com/) - Image hosting service

---

<div align="center">
  <sub>Built with ❤️ by <a href="https://github.com/j4ytr1n1ty">Jay</a></sub>
</div>
