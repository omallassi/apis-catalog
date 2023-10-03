use actix_web::{HttpResponse, Responder};
use actix_web::{get, post};
use serde::{Deserialize, Serialize};

extern crate reqwest;

use crate::app::dao::catalog::*;
use crate::app::apis::*;
use crate::shared::settings::*;

use log::{debug, error, info};

use chrono::{DateTime, TimeZone, Utc};

extern crate histogram;
use histogram::Histogram;

use std::convert::TryFrom;

/*
 * Metrics related APIs
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct Metrics {
    pub pr_num: Vec<(DateTime<Utc>, i32)>,
    pub pr_ages: Vec<(DateTime<Utc>, i64, i64, i64, i64)>,
    pub endpoints_num: Vec<(DateTime<Utc>, i32)>, //Vec<(DateTime<Utc>, Option<String>, Option<String>, i32)>,
    pub zally_violations: Vec<(DateTime<Utc>, std::collections::HashMap<i64, usize>)>,
    pub endpoints_num_per_audience: Vec<(DateTime<Utc>, std::collections::HashMap<String, usize>)>,
}

#[get("/v1/metrics")]
pub async fn get_all_metrics() -> impl Responder {
    info!("get all metrics");

    let pr_num_timeseries: Vec<(DateTime<Utc>, i32)> =
        match crate::app::dao::repo_metrics::get_metrics_pull_requests_number(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!(
                    "Error while getting get_metrics_pull_requests_number [{}]",
                    why
                );
                Vec::new()
            }
        };

    let pr_ages_timeseries: Vec<(DateTime<Utc>, i64, i64, i64, i64)> =
        match crate::app::dao::repo_metrics::get_metrics_pull_requests_ages(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!(
                    "Error while getting get_metrics_pull_requests_ages [{}]",
                    why
                );
                Vec::new()
            }
        };

    let endpoints_number: Vec<(DateTime<Utc>, i32)> =
        match crate::app::dao::repo_metrics::get_metrics_endpoints_number(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!("Error while getting get_metrics_endpoints_number [{}]", why);
                Vec::new()
            }
        };

    let zally_ignore_timeseries: Vec<(DateTime<Utc>, std::collections::HashMap<i64, usize>)> =
        match crate::app::dao::repo_metrics::get_metrics_zally_ignore(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!("Error while getting get_metrics_zally_ignore [{}]", why);
                Vec::new()
            }
        };
    let endpoints_audience_number: Vec<(DateTime<Utc>, std::collections::HashMap<String, usize>)> =
        match crate::app::dao::repo_metrics::get_metrics_endpoints_per_audience(&SETTINGS.database) {
            Ok(val) => val.points,
            Err(why) => {
                error!(
                    "Error while getting get_metrics_endpoints_per_audience [{}]",
                    why
                );
                Vec::new()
            }
        };

    //will combine PR informations with metrics
    let merged_prs: Vec<PullRequest> = get_pull_requests("MERGED").values;
    let merged_prs: Vec<(DateTime<Utc>, PullRequest)> = merged_prs
        .into_iter()
        .map(|val| {
            let dt = chrono::Utc.timestamp(val.closed_epoch.unwrap() / 1000, 0);
            (dt, val)
        })
        .collect();
    //
    // let endpoints_num: Vec<(DateTime<Utc>, i32)> = endpoints_number.points;
    let mut endpoints_num_incl_pr: Vec<(DateTime<Utc>, Option<String>, Option<String>, i32)> =
        Vec::new();
    for tuple in &endpoints_number {
        let date: DateTime<Utc> = tuple.0;
        for pr in &merged_prs {
            match date
                .format("%Y-%m-%d")
                .to_string()
                .starts_with(pr.0.format("%Y-%m-%d").to_string().as_str())
            {
                true => {
                    let annotation = format!(
                        "id: {}, title: {}, author: {:?}",
                        pr.1.id, pr.1.title, pr.1.author.user.email_address,
                    );
                    endpoints_num_incl_pr.push((
                        date,
                        Some(pr.1.id.to_string()),
                        Some(annotation),
                        tuple.1,
                    ));
                    break;
                }
                false => endpoints_num_incl_pr.push((date, None, None, tuple.1)),
            }
        }
    }

    //
    let metrics = Metrics {
        pr_num: pr_num_timeseries,
        pr_ages: pr_ages_timeseries,
        endpoints_num: endpoints_number,
        endpoints_num_per_audience: endpoints_audience_number,
        zally_violations: zally_ignore_timeseries,
    };

    HttpResponse::Ok().json(metrics)
}

#[post("/v1/metrics/refresh")]
pub async fn refresh_metrics() -> impl Responder {
    info!("refresh metrics");
    crate::app::dao::catalog::refresh_catalogs(&SETTINGS.catalogs, false);
    //
    let pull_requests: PullRequests = get_pull_requests("OPEN");

    //keep metric pr_num
    let metrics = get_metrics_pull_requests_number(&pull_requests);
    crate::app::dao::repo_metrics::save_metrics_pull_requests_number(&SETTINGS.database, metrics.0, metrics.1)
        .unwrap();
    //keep metric pr_age
    let current_epoch = std::time::SystemTime::now();
    let current_epoch = current_epoch.duration_since(std::time::UNIX_EPOCH).unwrap();
    let metrics = get_metrics_pull_requests_ages_stats(&pull_requests, current_epoch.as_secs());
    crate::app::dao::repo_metrics::save_metrics_pull_requests_ages(
        &SETTINGS.database,
        metrics.0,
        isize::try_from(metrics.1).unwrap(),
        isize::try_from(metrics.2).unwrap(),
        isize::try_from(metrics.3).unwrap(),
        isize::try_from(metrics.4).unwrap(),
    )
    .unwrap();

    //get # of endpoints
    let all_specs: Vec<SpecItem> = list_specs(&SETTINGS.catalogs);

    let all_specs_paths: Vec<String> = all_specs.iter().map(|val| val.path.to_string()).collect();
    info!(
        "List of retrieved and parsed OpenAPI Specifications [{:?}]",
        all_specs_paths
    );

    let len = &all_specs.len();
    let metrics = get_metrics_endpoints_num(&all_specs);
    info!(
        "Parsed [{}] specifications and got a total of [{:?}] paths",
        len, &metrics
    );
    crate::app::dao::repo_metrics::save_metrics_endpoints_num(&SETTINGS.database, metrics.0, metrics.1)
        .unwrap();

    //save metrics zally_ignore
    let stats = get_zally_ignore(&all_specs);
    crate::app::dao::repo_metrics::save_metrics_zally_ignore(&SETTINGS.database, Utc::now(), stats).unwrap();

    //save metrics endpoints_num_per audience
    let stats = get_endpoints_num_per_audience(&all_specs);
    crate::app::dao::repo_metrics::save_metrics_endpoints_num_per_audience(
        &SETTINGS.database,
        Utc::now(),
        stats,
    )
    .unwrap();
    //
    HttpResponse::Ok().json(pull_requests.size)
}

fn get_metrics_pull_requests_number(pull_requests: &PullRequests) -> (DateTime<Utc>, i32) {
    (Utc::now(), pull_requests.size)
}

fn get_metrics_pull_requests_ages_stats(
    pull_requests: &PullRequests,
    current_epoch: u64,
) -> (DateTime<Utc>, u64, u64, u64, u64) {
    let mut histogram = Histogram::new();
    //
    let _elapse_in_ms: Vec<_> = pull_requests
        .values
        .iter()
        .map(|val| {
            let created_epoch_in_sec = val.created_epoch / 1000;
            if current_epoch < created_epoch_in_sec {
                //val.created_epoch is in ms
                error!(
                    "Cannot compute epoch elapse as current epoch [{}] < obtained epoch [{}]",
                    current_epoch, val.created_epoch
                );
            }
            let delta: u64 = current_epoch - created_epoch_in_sec;
            //TODO clean unwrap
            histogram.increment(delta / 86400).unwrap(); //keep values in days

            delta
        })
        .collect();

    debug!(
        "Got Percentiles: p0: {} days p50: {} days p100: {} days mean: {} days",
        histogram.percentile(0.0).unwrap(),
        histogram.percentile(50.0).unwrap(),
        histogram.percentile(100.0).unwrap(),
        histogram.mean().unwrap(),
    );

    (
        Utc::now(),
        histogram.percentile(0.0).unwrap(),
        histogram.percentile(50.0).unwrap(),
        histogram.percentile(100.0).unwrap(),
        histogram.mean().unwrap(),
    )
}

//TODO move this method into catalog/mod.rs
fn get_metrics_endpoints_num(all_specs: &Vec<SpecItem>) -> (DateTime<Utc>, i32) {
    let endpoints_per_spec: Vec<_> = all_specs
        .iter()
        .map(|spec| {
            let num = spec.api_spec.paths.paths.len();
            debug!("# of paths - spec [{:?}] got [{:?}] paths", spec.path, num);

            num
        })
        .collect();

    let total: i32 = endpoints_per_spec.iter().sum::<usize>() as i32;
    info!(
        "# of paths - per spec [{:?}] - and total # of paths [{}]",
        &endpoints_per_spec, &total
    );

    (Utc::now(), total)
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    #[test]
    fn test_metrics_get_pr_number() {
        let response = r#"{"size":2,"limit":2,"isLastPage":false,"values":[{"id":57,"version":14,"title":"XXX API for currencies.","description":"XXX (partial) API.\nOnly exposes currencies list","state":"OPEN","open":true,"closed":false,"createdDate":1582305198106,"updatedDate":1585062047626,"fromRef":{"id":"refs/heads/xxx","displayId":"xxx","latestCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":2811,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"...","id":1504,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","role":"REVIEWER","approved":true,"status":"APPROVED"},{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":4304,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}],"participants":[],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":0,"commentCount":10,"openTaskCount":0},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/57"}]}},{"id":1,"version":93,"title":"Marketdata","description":"* Add 3 yamls about APIs for Service [MDS (w/ interpolation)] described under wiki https://my_wiki","state":"OPEN","open":true,"closed":false,"createdDate":1551955373000,"updatedDate":1582726600363,"fromRef":{"id":"refs/heads/marketdata","displayId":"marketdata","latestCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":4215,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":435,"displayName":"B","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":4436,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":3070,"displayName":"S","active":true,"slug":"dsubtil","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2842,"displayName":"E","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"6106a3ea81bd9fbbed4a7ccf694f572745040297","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"d","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"26d762f1c1242d3f2c29a328526154c13c923077","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","role":"REVIEWER","approved":true,"status":"APPROVED"}],"participants":[{"user":{"name":"","emailAddress":"...","id":1857,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"J....","id":3941,"displayName":"C","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":784,"displayName":"e","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/us"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"......","id":1483,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":2862,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"}],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":1,"commentCount":86,"openTaskCount":1},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/1"}]}}],"start":0,"nextPageStart":2}"#;
        let pull_requests: super::PullRequests = serde_json::from_str(response).unwrap();

        let metrics = super::get_metrics_pull_requests_number(&pull_requests);
        assert_eq!(2, metrics.1);
    }

    #[test]
    fn test_metrics_get_pr_ages() {
        let response = r#"{"size":2,"limit":2,"isLastPage":false,"values":[{"id":57,"version":14,"title":"XXX API for currencies.","description":"XXX (partial) API.\nOnly exposes currencies list","state":"OPEN","open":true,"closed":false,"createdDate":1582305198106,"updatedDate":1585062047626,"fromRef":{"id":"refs/heads/xxx","displayId":"xxx","latestCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":2811,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"...","id":1504,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"486e8c0b301114fcbfc53bdb4e4884765c7122db","role":"REVIEWER","approved":true,"status":"APPROVED"},{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":4304,"displayName":"L","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"}],"participants":[],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":0,"commentCount":10,"openTaskCount":0},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/57"}]}},{"id":1,"version":93,"title":"Marketdata","description":"* Add 3 yamls about APIs for Service [MDS (w/ interpolation)] described under wiki https://my_wiki","state":"OPEN","open":true,"closed":false,"createdDate":1551955373000,"updatedDate":1582726600363,"fromRef":{"id":"refs/heads/marketdata","displayId":"marketdata","latestCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"toRef":{"id":"refs/heads/master","displayId":"master","latestCommit":"eb26e8472c9beb4da8779b9783a2bbb68f176af1","repository":{"slug":"my_repo","id":4201,"name":"xxx","scmId":"git","state":"AVAILABLE","statusMessage":"Available","forkable":false,"project":{"key":"PAA","id":423,"name":"Arch.","description":"description .... ","public":false,"type":"NORMAL","links":{"self":[{"href":"https://stash_dns/projects/XYZ"}]}},"public":true,"links":{"clone":[{"href":"https://stash_dns/scm/xyz/xxx.git","name":"http"},{"href":"ssh://git@stash_dns:7999/xyz/xxx.git","name":"ssh"}],"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/browse"}]}}},"locked":false,"author":{"user":{"name":"","emailAddress":"...","id":4215,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"AUTHOR","approved":false,"status":"UNAPPROVED"},"reviewers":[{"user":{"name":"","emailAddress":"....","id":1283,"displayName":"W","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":435,"displayName":"B","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":4436,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"...","id":3070,"displayName":"S","active":true,"slug":"dsubtil","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2511,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"0fe3dff0f1a9415d35bddf0ffc004da155e5c26e","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"..","id":2842,"displayName":"E","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"6106a3ea81bd9fbbed4a7ccf694f572745040297","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"d","emailAddress":"...","id":2083,"displayName":"M","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"26d762f1c1242d3f2c29a328526154c13c923077","role":"REVIEWER","approved":false,"status":"UNAPPROVED"},{"user":{"name":"-ci","emailAddress":".....","id":8003,"displayName":"jenkins-ci","active":true,"slug":"-ci","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"lastReviewedCommit":"3947e71bd4e152d6f1b93b63232b32d09fa5562e","role":"REVIEWER","approved":true,"status":"APPROVED"}],"participants":[{"user":{"name":"","emailAddress":"...","id":1857,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"J....","id":3941,"displayName":"C","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"....","id":784,"displayName":"e","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/us"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":"......","id":1483,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"},{"user":{"name":"","emailAddress":".....","id":2862,"displayName":"S","active":true,"slug":"","type":"NORMAL","links":{"self":[{"href":"https://stash_dns/"}]}},"role":"PARTICIPANT","approved":false,"status":"UNAPPROVED"}],"properties":{"mergeResult":{"outcome":"CLEAN","current":true},"resolvedTaskCount":1,"commentCount":86,"openTaskCount":1},"links":{"self":[{"href":"https://stash_dns/projects/XYZ/repos/xxx/pull-requests/1"}]}}],"start":0,"nextPageStart":2}"#;
        let pull_requests: super::PullRequests = serde_json::from_str(response).unwrap();
        let current_epoch = std::time::SystemTime::now();
        //fix date
        let dt = DateTime::parse_from_rfc2822("Sun, 29 Mar 2020 20:36:29 +0000").unwrap();
        let metrics = super::get_metrics_pull_requests_ages_stats(
            &pull_requests,
            dt.timestamp() as u64, /*current_epoch.as_secs()*/
        );

        assert_eq!(37, metrics.1);
        assert_eq!(388, metrics.2);
        assert_eq!(388, metrics.3);
        assert_eq!(213, metrics.4);
    }
}
