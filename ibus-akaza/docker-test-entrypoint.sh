#!/bin/bash
set -e

echo "==> Starting Xvfb..."
Xvfb :99 -screen 0 1024x768x24 &
sleep 2

echo "==> Starting D-Bus session..."
dbus-daemon --session --fork --address="unix:path=/tmp/dbus-test-session"
sleep 1

echo "==> Building test data..."
cd akaza-data && make test-data
cd ..

echo "==> Creating ibus-akaza configuration..."
cd ibus-akaza && make
cd ..

case "$1" in
    "test")
        echo "==> Running all tests..."
        cargo test
        ;;
    "test-unit")
        echo "==> Running unit tests..."
        cargo test --lib
        ;;
    "test-integration")
        echo "==> Running integration tests..."
        cargo test --test integration
        ;;
    "test-e2e")
        echo "==> Running E2E tests..."
        cargo test --test e2e -- --ignored --test-threads=1
        ;;
    "bash")
        echo "==> Starting bash shell..."
        exec /bin/bash
        ;;
    *)
        echo "Usage: $0 {test|test-unit|test-integration|test-e2e|bash}"
        exit 1
        ;;
esac
