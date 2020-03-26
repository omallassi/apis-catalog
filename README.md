[![Actions Status](https://github.com/omallassi/apis-catalog/workflows/Rust/badge.svg)](https://github.com/omallassi/apis-catalog/actions)

# apis-catalog

> :warning: All of this is, at this stage ideas and POC

## Overview

In all companies that expose APIs (so in fact all companies in 2020), there is a need to ensure a proper level of quality and consistency of these APIs, as they reflect your product, its quality and its capabilities.

This is then not uncommon then to talk about *API [Governance](#about-governance)*. Usually, when you talk about governance, you get this reaction: "_Outch! `Governance`! That's from another age_". Well, I do think governance is still needed even if I do believe this is complex balance and it can quickly fall into a trap of becoming a bottleneck for the rest of the organisation.

Talking about Governance, you can think about (at least) :

* _Design time Governance_ where we usually try to manage questions like : What are my APIs? how many versions of my specifications do I have? Are my APIs syntaxically correct? Are my APIs semantically correct (hum, this one will generate a lot more [discussions](#about-governance))? Are security and compliance policies followed? Is this evolution backward compatible (or not)? How frequent is an evolution on my APIs?

* _Runtime Governance_ where we usually try to manage questions like : Where are my APIs deployed (on which environments, which gateways...)? how many versions are deployed? is this API still used (or not) ?

A lot of great solutions exists in these spaces and the goal here is not to replace any of them. 

## Ok cool...why this, what are the goals

The goals are 

* to ease keeping track of your endpoints and APIs (be thy OpenAPI, WSDL, AsyncAPI etc...) : from inception to production and end of life
* to help you monitoring the efficiency of your governance process (whatever it is)

With the following principles in mind: 

* reuse and integrate with simple, massively used open source frameworks (like git, perforce, jenkins...)
* reuse and integrate with existing specification formats (_e.g._ OpenAPI, AsyncAPI...)
* Help organising your API specifications : in domains, with owners, in environments etc...
* give as much freedom as possible with respect to "reviews or validation workflows". The best freedom being to not provide anything, any kind of workflow on this topic. 
* do not be on the critical path of anything related to CI / CD pipeline, just keep track of things, and stay optional
* accept to fail & have fun with `Rust` (ok, langage has nothing to do here but I tend to like Rust and would like to continue learning it)

This picture tries to depict a possible end state : 

```ditaa {cmd=true args=["-E"]}
                                                                                                                                      +----+
                                                                                                                           RUNTIME GOV    |
                                                                                                                                          |
                                                                                                                                  +---+   |
                                                                               +------------------+                         DEPLOYMENT|   |
                                                               metrics         |                  |                                   |   |
                                                                 +------------>+    Environment   |                                   |   |
                                                                 |             |   (API gateway..)|                                   |   |
                                                                 |             |                  |                                   |   |
+----------------------------------------------------------+     |             +----+-------------+                                   |   |
|         apis+catalog & apis+catalog+web                  |     |                  ^ deploy API Specs                            +---+   |
|                                                          |     |                  |                                                     |
|         +--------------+               +--------------+  |     |             +----+-------------+                                       |
|         |              |               |              |  |     |             |                  |  get API Specs to deploy              |
|         |    Web UI    |               |     CLI      |  |     |             |      CI/CD       +-+(commit+id / master)                 |
|         |              |               |              |  |     |             |    pipeline      |               +                       |
|         +-------+------+               +-+------------+  |     |             +-------+----------+               |                       |
|                 |                        |               |     |                     |                          |                       |
|                 |                        v               |     |                     |                          |                  +----+
|                 |     +------------------+------         |     |                     |                          |
|                 |     |                        +---------------+                     |                          |
|                 +---->+       HTTP Backend     |         |        update "deployment"when API is deployed       |
|                       |                        <-------------------------------------+                          |                  +----+
|          +------------+                        |         |                                                      |        DESIGN TIME GOV|
|          |            +-----+------------+-----+--------------------------------+                               |                       |
|          |                  |            |               |    metrics           |                               |                       |
|    +-----v---+  +-----------v---+    +---v------------+  |                   +--v---------------+               |              +---+    |
|    | metrics |  |  metadata     |    | Open API specs |  |                   |   apis+catalog   |               |           DESIGN |    |
|    | history |  | (env, apis,   |    | Async API specs|  |                   |   (git Repo)     +<--------------+                  |    |
|    |         |  | domains...)   |    | (yml files)    |  |  git pull / push  |                  |                                  |    |
|    |         |  |               |    |                +--------------------->+    (yml files)   |                                  |    |
|    |         |  |               |    |                |  |                   |                  +----+                             |    |
|    +---------+  +---------------+    +----------------+  |          +--------+---+-------^------+    |                             |    |
|                                                          |          |            |       |           |                             |    |
+----------------------------------------------------------+          |            |       +-----------+                             |    |
                                                                      |            |           Pull Requests / API Review            |    |
                                                                      |            |                                             +---+    |
                                                                      |            | generate code (stubs, mock...)                       |
                                                     +----------------v-+      +---v--------------+                              +---+    |
                                                     |                  |      |                  |                           BUILD  |    |
                                                     |   HTML API doc   |      |  Artifacts Repo  |                                  |    |
                                                     |                  |      |    (Nexus...)    |                                  |    |
                                                     |                  |      |                  |                                  |    |
                                                     |                  |      |                  |                                  |    |
                                                     +------------------+      +------------------+                                  |    |
                                                                                                                                 +---+    |
                                                                                                                                          |
                                                                                                                                     +----+


```
made with http://asciiflow.com/

Web UI is available here https://github.com/omallassi/apis-catalog-web


## About Governance

Governance is not a goal but a way to improve quality of our outcomes (w.r.t to policies you have defined)

Yet, reviewing an API, discussing an API, co-building an API is key: it is key to define the concepts, their definition (ubquitous langage) their modelisation (_e.g._ UML or other...), how concepts relate to each other, their lifecycles (and the problem of the atomicity behind the scene), their performance (which quickly relate to the size of the payload), their potential evolutions. While tools like https://github.com/zalando/zally provide awesome support, this is the syntaxic validation. The semantic validation is another game that requires discussions. 


I also believe that specifications (WSDL, Open API), while not having the same lifecycle than the business logic implementation, should be managed the same way as code. Source Control Software now support "Pull requests" mechanisms that could ease and support these discussions around the API and its "more formal approval or 4-eyes checking". 

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
