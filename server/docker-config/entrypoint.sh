#!/bin/sh

echo "Hey there"

envsubst < "./config/local-template.toml" > "./config/local.toml" 
cat ./config/local.toml

export API_CONFIG_FILE=./config/local.toml
RUST_LOG="info,apis_catalog_server=debug" ./target/debug/apis_catalog_server