# Contributing to PFP Checker

Thank you for considering contributing to PFP Checker! This document outlines the process for contributing to the project.

## Code of Conduct

By participating in this project, you are expected to uphold our Code of Conduct: be respectful, constructive, and collaborative.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR-USERNAME/pfp-checker.git`
3. Create a branch for your changes: `git checkout -b feature/your-feature-name`

## Development Environment Setup

### Prerequisites

- Rust (latest stable version)
- SQLite
- Discord Bot Token

### Setup Steps

1. Install Rust using [rustup](https://rustup.rs/)
2. Install the SQLite CLI tools
3. Install the SQLx CLI:
   ```bash
   cargo install sqlx-cli
   ```
4. Create a `.env` file from the example:
   ```bash
   cp .env.example .env
   ```
5. Fill in your Discord token and ImgBB API key in the `.env` file
6. Set up the database:
   ```bash
   sqlx database setup --database-url sqlite:database.sqlite
   ```
7. Build the project:
   ```bash
   cargo build
   ```
8. Run the tests:
   ```bash
   cargo test
   ```

## Making Changes

1. Make your changes in your feature branch
2. Write or update tests as necessary
3. Ensure all tests pass
4. Update documentation if needed
5. Format your code with `cargo fmt`
6. Run `cargo clippy` to check for common issues

## Submitting Changes

1. Push your changes to your fork
2. Submit a pull request to the main repository
3. Describe your changes in detail
4. Link any related issues

## Pull Request Process

1. Ensure your PR includes tests if adding functionality
2. Update the README.md or documentation with details of changes if appropriate
3. The PR will be merged once it receives approval from a maintainer

## Database Migrations

If you need to make changes to the database schema:

1. Create a new migration:
   ```bash
   sqlx migrate add your_migration_name
   ```
2. Edit the generated SQL file in the `migrations` directory
3. Apply the migration:
   ```bash
   sqlx database reset --database-url sqlite:database.sqlite
   ```

## Coding Style

- Follow standard Rust style conventions
- Use meaningful variable and function names
- Comment complex sections of code
- Keep functions small and focused

## Questions?

Feel free to open an issue if you have any questions about contributing!
