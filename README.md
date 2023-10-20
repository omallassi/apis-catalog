![logo](https://raw.githubusercontent.com/wiki/omallassi/apis-catalog/assets/logo.png)

[![Actions Status](https://github.com/omallassi/apis-catalog/actions/workflows/rust.yml/badge.svg) [![Coverage Status](https://coveralls.io/repos/github/omallassi/apis-catalog/badge.svg)](https://coveralls.io/github/omallassi/apis-catalog)


> :warning: All of this is, at this stage ideas, POC...tested in real life. 

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

> **_NOTE:_**  from the above schema, the box apis-catalog is the current github repository whereas the box apis-catalog-web is the this [repository](https://github.com/omallassi/apis-catalog-web).

## Overview 
### Available Statistics

Refer to [this page](https://github.com/omallassi/apis-catalog/wiki/stats-overview) to get a list of available statistics and their definitions. 

### Available Clients

There are two clients that can be used

* a [CLI](https://github.com/omallassi/apis-catalog/wiki/CLI-overview) that provides way to manage the artefacts of the catalog (_ie._ create domains, env, deployments etc...).
* a [Web UI](https://github.com/omallassi/apis-catalog/wiki/Web-UI-Overview) that provides part of the `CLI`capabilities in terms of management but provides [additional reports](https://github.com/omallassi/apis-catalog-web) and [statistics](https://github.com/omallassi/apis-catalog/wiki/stats-overview).

## Getting Started

Please refer to this [page](https://github.com/omallassi/apis-catalog/wiki/installation)

## More Details
available in the [wiki](https://github.com/omallassi/apis-catalog/wiki).
