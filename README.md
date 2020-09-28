![logo](https://raw.githubusercontent.com/wiki/omallassi/apis-catalog/assets/logo.png)

[![Actions Status](https://github.com/omallassi/apis-catalog/workflows/Rust/badge.svg)](https://github.com/omallassi/apis-catalog/actions) [![Build Status](https://travis-ci.org/omallassi/apis-catalog.svg?branch=master)](https://travis-ci.org/omallassi/apis-catalog) [![Coverage Status](https://coveralls.io/repos/github/omallassi/apis-catalog/badge.svg?branch=master)](https://coveralls.io/github/omallassi/apis-catalog?branch=master)

## Overview 
> :warning: All of this is, at this stage ideas and POC

In all companies that expose APIs (so in fact all companies in 2020), there is a need to ensure a proper level of quality and consistency of these APIs, as they reflect your product, its quality and its capabilities.

This is then not uncommon then to talk about *API [Governance](#about-governance)*. Usually, when you talk about governance, you get this reaction: "_Outch! `Governance`! That's from another age_". Well, I do think governance is still needed even if I do believe this is complex balance and it can quickly fall into a trap of becoming a bottleneck for the rest of the organisation.

Talking about Governance, you can think about (at least) :

* _Design time Governance_ where we usually try to manage questions like : What are my APIs? how many versions of my specifications do I have? Are my APIs syntaxically correct (ie. how many zally ignore do I have...) ? Are my APIs semantically correct (hum, this one will generate a lot more [discussions](#about-governance))? Are security and compliance policies followed? Is this evolution backward compatible (or not)? How frequent is an evolution on my APIs?

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

This picture tries to depict a [_possible state_](https://github.com/omallassi/apis-catalog/wiki/overview) : 

![overview](https://raw.githubusercontent.com/wiki/omallassi/apis-catalog/assets/287a566176d137e603a1305388877384.png)

## Getting Started

### Available Statistics

Refer to [this page](https://github.com/omallassi/apis-catalog/wiki/stats-overview) to get a list of available statistics and their definitions. 

### Prepare the DB 

There are two modes : 

* based on sqlite. For more details about using sqlite, please [refer to this page](https://github.com/omallassi/apis-catalog/wiki/installation).
* based on mysql (not yet supported).

### Run  the REST Backend

* Rename `server/config/sample.toml`into `server/config/local.toml`and upate the properties

* Run `RUST_LOG="info,apis_catalog_server=debug" ./target/debug/apis_catalog_server`

### Run the CLI
Some commands: 
```
[omallassi@sup-sachs apis-catalog]$./target/debug/catalog tiers create --name "API Gateway"
[omallassi@sup-sachs apis-catalog]$./target/debug/catalog tiers create --name "Application Layer"
[omallassi@sup-sachs apis-catalog]$./target/debug/catalog tiers create --name "Business Layer"

[omallassi@sup-sachs apis-catalog]$./target/debug/catalog tiers list
 Id                                   | Name 
--------------------------------------+-------------------
 cbd81ed2-09e0-424b-a0d8-6e7bf939d50f | Business Layer 
 542cb0ce-8c89-431a-b6cb-0b18c6c439e2 | Application Layer 
 79a0effb-a755-466b-a4d5-da87159b2149 | API Gateway 



RUST_LOG=debug ./target/debug/catalog domains create --name /domain1
RUST_LOG=debug ./target/debug/catalog domains create --name /domain2 --description "This domain owns........ all you have ever dreamed about. Do not think a lot. this is just the place to be...."
RUST_LOG=debug ./target/debug/catalog domains create --name /domain3
RUST_LOG=debug ./target/debug/catalog domains create --name /domain2/subdomain2.1/subdomain2.2 --description "This is a smaller part of your dreamed domain. smaller, but still enjoyable"
RUST_LOG=debug ./target/debug/catalog domains list

Id                                   | Domain Name 
--------------------------------------+------------------------------------
 f503f8e3-0e35-4e75-b9dd-3d8a549b85f3 | /domain2/subdomain2.1/subdomain2.2 
 0f9c652c-291b-4b4b-840e-d5b32627db60 | /domain3 
 d2046e28-cee3-413b-ac81-d4405d0e375e | /domain2 
 c6cceac2-0905-4fe8-9b06-b8f45c373c90 | /domain1 


RUST_LOG=debug ./target/debug/catalog env create --name pre-prod.apac.my-corp.com --description "APAC preprod env for my solutions"
RUST_LOG=debug ./target/debug/catalog env create --name apac.my-corp.com --description "APAC env for my solutions"
RUST_LOG=debug ./target/debug/catalog env list


Id                                   | Env Name                  | Description 
--------------------------------------+---------------------------+-----------------------------------
 f7346d04-44f6-417e-84ba-bc623086d9fd | apac.my-corp.com          | APAC env for my solutions 
 06cf9312-11e2-4d49-818f-332827e5a24a | pre-prod.apac.my-corp.com | APAC preprod env for my solutions 





RUST_LOG=debug ./target/debug/catalog apis create --name my_api_1 --spec-ids 12 --domain-id c6cceac2-0905-4fe8-9b06-b8f45c373c90
RUST_LOG=debug ./target/debug/catalog apis create --name my_api_2 --spec-ids 12 --domain-id c6cceac2-0905-4fe8-9b06-b8f45c373c90
RUST_LOG=debug ./target/debug/catalog apis list


Id                                   | Name     | Tier | Domain                               | Domain   | Specs 
--------------------------------------+----------+------+--------------------------------------+----------+-------
 afebabf3-a55c-4915-8ff9-663ef98f260e | my_api_2 | N/A  | c6cceac2-0905-4fe8-9b06-b8f45c373c90 | /domain1 | [] 
 47588986-8102-49d6-ab1b-ee96a5b964a4 | my_api_1 | N/A  | c6cceac2-0905-4fe8-9b06-b8f45c373c90 | /domain1 | [] 




RUST_LOG=debug ./target/debug/catalog deployments create --api 47588986-8102-49d6-ab1b-ee96a5b964a4 --env 06cf9312-11e2-4d49-818f-332827e5a24a

RUST_LOG=debug ./target/debug/catalog deployments list

Apis                                 | Env 
--------------------------------------+--------------------------------------
 47588986-8102-49d6-ab1b-ee96a5b964a4 | 06cf9312-11e2-4d49-818f-332827e5a24a 


same command as: 

RUST_LOG=debug ./target/debug/catalog deployments list --api 47588986-8102-49d6-ab1b-ee96a5b964a4


RUST_LOG=debug ./target/debug/catalog apis tier --api 47588986-8102-49d6-ab1b-ee96a5b964a4 --tier 542cb0ce-8c89-431a-b6cb-0b18c6c439e2

RUST_LOG=debug ./target/debug/catalog apis status --api 47588986-8102-49d6-ab1b-ee96a5b964a4 --value validated

```

## Run the Web UI 
Refer to [apis-catalog-web](https://github.com/omallassi/apis-catalog-web) for more details. 

## To play with metrics

```
curl http://127.0.0.1:8088/v1/metrics
curl -X POST  http://127.0.0.1:8088/v1/metrics/refresh -d user=fff -d pwd=sdf
```

## More Details
available in the [wiki](https://github.com/omallassi/apis-catalog/wiki).
