[![Actions Status](https://github.com/omallassi/apis-catalog/workflows/Rust/badge.svg)](https://github.com/omallassi/apis-catalog/actions)

# apis-catalog

## Overview

In all companies that expose APIs, there is a need to ensure a proper level of quality of these APIs, as they reflect your product, its quality and its capabilities.

This is not uncommon then to talk about `API Governance`. Outch! `Governance`! That's from another age. Well, I do not think even if I do believe it can fall into a trap of becoming a bottleneck for the rest of the organisation.

Yet, reviewing an API, discussing an API, co-building an API is key: it is key to define the concepts, their definition (ubquitous langage) their modelisation (_e.g._ UML or other...), how concepts relate to each other, their lifecycles (and the problem of the atomicity behind the scene), their performance (which quickly relate to the size of the payload), their potential evolutions. While tools like https://github.com/zalando/zally provide awesome support, this is the syntaxic validation. The semantic validation is another game that requires discussions. 

I also believe that specifications (WSDL, Open API), while not having the same lifecycle than the business logic implementation, should be managed the same way as code. Source Control Software now support "Pull requests" mechanisms that could ease and support these discussions around the API and its "more formal approval or 4-eyes checking". 

Yet, this is not enough. You need to know how many versions you have, who owns them, in which envs are deployed the specifications, to which domain the API belongs to. This is where `apis-catalog` & `apis-catalog-web` (try to) fit in: 

* It is not replicating the API specifications (Open API, AsyncAPI, WSDL...). It is built on top of a git repository that contains these specifications
* It is not providing any validation workflow, letting you the freedom to define your own validation workflow
* It is providing a way to manage domains, APIs, their versions, environments. 
* It is providing a way to follow and know which specifications are deployed into which environments. For instance, it should be easy to know that this specification (_e.g_ this open api specification at this commit-id) is deployed in `pred-prod.my-company.com` and `prod.my-company.com`.

```
                                                                       +------------------+
                                                                       |                  |
                                                                       |    Environment   |
                                                                       |   (API gateway..)|
                                                                       |                  |
+--------------------------------------------------+                   +----+-------------+
| apis+catalog & apis+catalog+web                  |                        ^ deploy API Specs
|                                                  |                        |
| +--------------+               +--------------+  |                   +----+-------------+
| |              |               |              |  |                   |                  |  get API Specs to deploy
| |    Web UI    |               |     CLI      |  |                   |      CI/CD       +-+(commit+id / master)
| |              |               |              |  |                   |    pipeline      |               +
| +-------+------+               +-----------+--+  |                   +-------+----------+               |
|         |                                  |     |                           |                          |
|         |                                  |     |                           |                          |
|         |     +------------------------+   |     |                           |                          |
|         |     |                        |   |     |                           |                          |
|         +---->+       HTTP Backend     +---+     |                           |                          |
|               |                        <-------------------------------------+                          |
|               +-----+------------+-----+         |        update "deployment" when API is deployed      |
|                     |            |               |                                                      |
|                     |            |               |                                                      |
|         +-----------v---+    +---v------------+  |                   +------------------+               |
|         |  metadata     |    | Open API specs |  |                   |   apis+catalog   |               |
|         | (env, apis,   |    | Async API specs|  |                   |   (git Repo)     +<--------------+
|         | domains...)   |    | (yml files)    |  |  git pull / push  |                  |
|         |               |    |                +--------------------->+    (yml files)   |
|         |               |    |                |  |                   |                  +----+
|         +---------------+    +----------------+  |          +--------+---+-------^------+    |
|                                                  |          |            |       |           |
+--------------------------------------------------+          |            |       +-----------+
                                                              |            |           Pull Requests / API Review
                                                              |            |
                                                              |            | generate code (stubs, mock...)
                                             +----------------v-+      +---v--------------+
                                             |                  |      |                  |
                                             |   HTML API doc   |      |  Artifacts Repo  |
                                             |                  |      |    (Nexus...)    |
                                             |                  |      |                  |
                                             |                  |      |                  |
                                             +------------------+      +------------------+

```
made with http://asciiflow.com/

Web UI is available here https://github.com/omallassi/apis-catalog-web

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
https://github.com/omallassi/apis-catalog/wiki (In particular (kind of) UML modelisation)
