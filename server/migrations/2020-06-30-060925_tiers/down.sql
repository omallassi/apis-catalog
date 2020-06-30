-- This file should undo anything in `up.sql`

drop table if exists tiers;
drop table if exists status;
drop table if exists apis;
drop table if exists deployments;
drop table if exists domains;
drop table if exists envs;
drop table if exists metrics_pr_num;
drop table if exists metrics_pr_ages;
drop table if exists metrics_endpoints_num;
