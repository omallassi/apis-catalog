#!/bin/sh

# Create the config file w/ env var
echo -e "\033[1;32m*** Creating config file [./config/local.toml] ***\033[0m"

envsubst < "./config/local-template.toml" > "./config/local.toml" 
cat ./config/local.toml

# Start the server
echo -e "\033[1;32m*** Starting apis-catalog Server ***\033[0m"
export API_SERVER_CONFIG_FILE=./config/local.toml
RUST_LOG="info,apis_catalog_server=debug" ./apis_catalog_server