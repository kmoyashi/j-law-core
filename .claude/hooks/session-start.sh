#!/bin/bash
set -euo pipefail

# Check if gh is already installed
if command -v gh &> /dev/null; then
  echo "gh is already installed"
  exit 0
fi

# Install GitHub CLI (gh)
apt-get update -qq
apt-get install -y -qq gh

echo "gh has been installed successfully"
