#!/bin/sh

set -e

FILE=./ext_bin/diesel

if [ -f "$FILE" ]
then
  echo "already installed"
else
  echo "installing diesel_cli $DIESEL_CLI_VERSION"
  cargo install diesel_cli --version $DIESEL_CLI_VERSION --no-default-features --features postgres
  cp ~/.cargo/bin/diesel ./ext_bin/
fi
