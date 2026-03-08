#!/bin/bash
set -euo pipefail

# Docker デーモン起動 - Session 中の全言語バインディングテスト実行を可能にする
# このスクリプトは Claude がコミット前テストを実行するたびに
# Docker が利用可能な状態を保証する

if ! pgrep -x "dockerd" > /dev/null 2>&1; then
  echo "Starting Docker daemon..."
  dockerd --host unix:///var/run/docker.sock &> /tmp/dockerd.log &

  # Docker デーモンの起動を待機（最大10秒）
  local attempts=0
  while [ $attempts -lt 10 ]; do
    if docker ps > /dev/null 2>&1; then
      echo "✓ Docker daemon started successfully"
      break
    fi
    attempts=$((attempts + 1))
    sleep 1
  done

  if [ $attempts -eq 10 ]; then
    echo "✗ Failed to start Docker daemon"
    tail -20 /tmp/dockerd.log
    exit 1
  fi
else
  echo "✓ Docker daemon is already running"
fi

# docker compose の確認
if ! docker compose version > /dev/null 2>&1; then
  echo "✗ docker compose plugin is not available"
  exit 1
fi

echo "✓ Docker environment is ready for make docker-test"
