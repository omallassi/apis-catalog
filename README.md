[![Actions Status](https://github.com/omallassi/apis-catalog/workflows/Rust/badge.svg)](https://github.com/omallassi/apis-catalog/actions)

# apis-catalog

```

                    +--------------+               +--------------+
                    |              |               |              |
                    |    Web UI    |               |     CLI      |
                    |              |               |              |
                    +-------+------+               +-----------+--+
                            |                                  |
                            |                                  |
                            |     +------------------------+   |
                            |     |                        |   |
                            +---->+       HTTP Backend     +---+
                                  |                        |
                                  +-----+------------+-----+
                                        |            |
                                        |            |
                            +-----------v---+    +---v------------+                      +------------------+
                            |  metadata     |    | Open API specs |                      |   apis-catalog   |
                            | (env, apis,   |    |                |                      |    git Repo      |
                            | domains...)   |    | (yml files)    |     git pull / push  |                  |
                            |               |    |                +--------------------->+                  |
                            |               |    |                |                      |                  +----+   |
                            +---------------+    +----------------+                      +-----------^------+    |
                                                                                                     |           |
                                                                                                     +-----------+

```
made with http://asciiflow.com/

## Getting Started

Some commands 

```
RUST_LOG=debug ./target/debug/catalog domains create --name domain1
RUST_LOG=debug ./target/debug/catalog domains create --name domain2
RUST_LOG=debug ./target/debug/catalog domains create --name domain3
RUST_LOG=debug ./target/debug/catalog domains list
RUST_LOG=debug ./target/debug/catalog apis create --domain-id f8e87f4c-953c-4330-b522-f7d5f883d7ab --name my_sampe_api --spec-ids 12
RUST_LOG=debug ./target/debug/catalog apis create --domain-id f8e87f4c-953c-4330-b522-f7d5f883d7ab --name my_sampe_api_2 --spec-ids 12
RUST_LOG=debug ./target/debug/catalog apis list

RUST_LOG=debug ./target/debug/catalog env create --name xva.apac.murex.com --description "APAC env for xVA related solutions"

RUST_LOG=debug ./target/debug/catalog deploy --api 160d9e73-3e6a-4387-87f4-a16e692d0d80 --env a3904f15-83ea-46b3-bca0-1e0df2337e90

RUST_LOG=debug ./target/debug/catalog env list

RUST_LOG=debug ./target/debug/catalog deployments list --api 160d9e73-3e6a-4387-87f4-a16e692d0d80

```

## Run the server
`RUST_LOG="info,apis_catalog_server=debug" ./target/debug/apis_catalog_server`

static page : http://127.0.0.1:8088 -> route to underlying folder to apis-catalog-web build folder

## More 
https://github.com/omallassi/apis-catalog/wiki
