# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - Current

### Added

- Server icon tracking functionality with monitoring commands
- Improved ping command with latency display (API latency in milliseconds)
- Comprehensive database tests for server tracking
- Server history and statistics commands (`/serverpfphistory`, `/serverstats`)
- Monitor server commands (`/monitorserver`, `/removemonitorserver`)

### Fixed

- Permission checks for monitorserver and removemonitorserver commands
- Error handling consistency and diagnostics across the codebase
- Security vulnerabilities in dependencies
- Removed unused duplicate ImgBB structs

### Changed

- Replaced `unwrap()` with proper error handling in critical paths
- Extracted duplicate monitoring logic into generic helper functions
- Comprehensive docstrings added to public functions
- Updated README with server tracking commands

### Dependencies

- Updated tokio from 1.44.2 to 1.47.1
- Updated chrono from 0.4.40 to 0.4.42
- Updated reqwest from 0.12.15 to 0.12.24
- Updated sqlx from 0.8.3 to 0.8.6
- Updated serde from 1.0.219 to 1.0.228
- Updated serde_json from 1.0.140 to 1.0.145
- Updated openssl from 0.10.66 to 0.10.72

## [0.4.0] - 2024

### Added

- Username history tracking and related command
- Docker Compose support for production environments
- GitHub Actions workflows for automated builds and releases

## [0.3.0]

### Added

- Stats Command
  - Amount of changes since monitor
  - Start of Monitor
  - Average changes per week, month and year

## [0.2.0]

### Added

- Database integration with local SQLite Database
- Scheduled monitoring of users
- Remove Monitor Command
- History Command
  - Previous Profile Pictures

## [0.1.0]

### Added

- Initial release
- Basic Discord bot functionality
- Profile picture tracking concept
