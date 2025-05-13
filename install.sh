#!/bin/sh

export PATH="$HOME/.cargo/bin:$PATH"

PROGRAM=re
BINDIR=/usr/local/bin
TARGET="$BINDIR/$PROGRAM"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo is not installed or not in PATH"
  exit 1
fi

if [ -f "$TARGET" ]; then
  echo "Warning: $TARGET already exists."
  printf "Do you want to overwrite it [y/N]: "
  read answer
  case "$answer" in
    [Yy]* ) echo "Overwriting $TARGET...";;
    * ) echo "Installation aborted."; exit 0;;
  esac
fi

cargo build --release

sudo cp target/release/$PROGRAM $BINDIR
sudo chmod 755 $BINDIR/$PROGRAM

echo "Installed $PROGRAM to $BINDIR"
