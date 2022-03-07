#!/bin/sh

echo "Hey there"

envsubst < "./config/local-template.toml" > "./config/local.toml" 
cat ./config/local.toml


# need to git your repo...

export API_SERVER_CONFIG_FILE=./config/local.toml
RUST_LOG="info,apis_catalog_server=debug" ./target/debug/apis_catalog_server