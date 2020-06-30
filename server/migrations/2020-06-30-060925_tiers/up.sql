-- Your SQL goes here

CREATE TABLE IF NOT EXISTS apis (
    api_id INTEGER PRIMARY KEY,
    id UUID  NOT NULL UNIQUE,
    name TEXT NOT NULL, 
    domain_id UUID NOT NULL, 
    tier_id UUID NOT NULL
);

CREATE TABLE IF NOT EXISTS tiers (
    tier_id INTEGER PRIMARY KEY,
    id UUID NOT NULL,
    name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS status (
    status_id INTEGER PRIMARY KEY,
    api_id UUID NOT NULL,
    status TEXT NOT NULL,
    start_date_time TEXT NOT NULL, 
    end_date_time TEXT
);

CREATE TABLE IF NOT EXISTS deployments (
    deployment_id INTEGER PRIMARY KEY,
    api TEXT NOT NULL,
    env TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS domains (
    domain_id INTEGER PRIMARY KEY,
    id UUID  NOT NULL UNIQUE,
    name TEXT NOT NULL, 
    description TEXT
);

CREATE TABLE IF NOT EXISTS envs (
    env_id INTEGER PRIMARY KEY,
    id UUID  NOT NULL UNIQUE,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS metrics_pr_num (
    date_time TEXT NOT NULL UNIQUE, 
    value INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS metrics_pr_ages (
    date_time TEXT NOT NULL UNIQUE, 
    p0 INTEGER NOT NULL, 
    p50 INTEGER NOT NULL, 
    p100 INTEGER NOT NULL, 
    mean INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS metrics_endpoints_num (
    date_time TEXT NOT NULL UNIQUE, 
    value INTEGER NOT NULL
);
