#!/usr/bin/env bash

# Directory of the Rust project
PROJECT_DIR="/opt/pfp-checker"
TMUX_SESSION_NAME="pfp-checker"
BINARY_PATH="$PROJECT_DIR/target/release/pfp-checker"

cd "$PROJECT_DIR" || { echo "Deployment directory not found"; exit 1; }

# Pull the latest code from GitHub
echo "Pulling latest changes..."
git fetch origin main || { echo "Git fetch failed"; exit 1; }

# Check if there are any changes to pull
LOCAL_COMMIT=$(git rev-parse HEAD)
REMOTE_COMMIT=$(git rev-parse origin/main)

BUILD_NEEDED=false

if [ "$LOCAL_COMMIT" != "$REMOTE_COMMIT" ]; then
  echo "Changes detected. Pulling latest changes and rebuilding..."
  git pull origin main || { echo "Git pull failed"; exit 1; }
  BUILD_NEEDED=true

  # Build the project in release mode
  echo "Building project in release mode..."
  cargo build --release || { echo "Build failed"; exit 1; }
fi

# Check if the tmux session already exists
if tmux has-session -t "$TMUX_SESSION_NAME" 2>/dev/null; then
  # If the binary file has changed (build happened), restart the session
  if [ "$BUILD_NEEDED" = true ]; then
    echo "Restarting tmux session with the updated build..."
    tmux kill-session -t "$TMUX_SESSION_NAME"
  else
    echo "Tmux session already running. No rebuild needed."
    exit 0
  fi
fi

# Start a new tmux session with the built binary
echo "Starting a new tmux session..."
tmux new-session -d -s "$TMUX_SESSION_NAME" "$BINARY_PATH"
