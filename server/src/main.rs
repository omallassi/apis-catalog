extern crate log;
//extern crate env_logger;
extern crate uuid;

extern crate config;

mod app;

mod shared;
use shared::settings::*;

use actix_files::NamedFile;
use actix_web::{middleware::Logger, web, App, HttpServer};
use actix_web::{HttpRequest, Result, Error};
use std::path::PathBuf;

use log::info;

#[macro_use]
extern crate lazy_static;

/**
 * To server static pages
 */
async fn index(_req: HttpRequest) -> Result<NamedFile, Error> {
    //info!("Requested Resource [{:?}]", req.match_info().query("filename") );

    let mut path: PathBuf = PathBuf::from(&SETTINGS.server.static_resources_path);
    path.push("index.html");

    // match req.match_info().query("filename").parse::<PathBuf>() {
    //     Ok(val) => {
    //         if(val.eq(&PathBuf::from(""))){
    //             path.push("index.html");
    //         }
    //         else{
    //             path.push(val);
    //         }
    //     }
    //     Err(_err) => {
    //         path.push("index.html");
    //     }
    // }

    info!("Loading page [{:?}]", path);

    Ok(NamedFile::open(path)?)
}


#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;


/**
 *
 */
#[actix_web::main]
async fn main() {

    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    //TODO std::env::set_var("RUST_LOG", "actix_web=info");
    //env_logger::init();

    // let colors = fern::colors::ColoredLevelConfig::new()
    //     .debug(fern::colors::Color::Blue)
    //     .info(fern::colors::Color::Green)
    //     .warn(fern::colors::Color::Yellow)
    //     .error(fern::colors::Color::Red);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] - [{}] - [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                //colors.color(record.level()),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("logs.log").unwrap())
        .apply()
        .unwrap();

    //do this call to warm up internal caches
    crate::app::catalogs::resfresh_caches_and_indexes(true);

    /*
     * HTTP Server
     */
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
            //deployment related endpoints
            .service(app::deployments::add_deployment)
            .service(app::deployments::get_deployments)
            .service(
                web::resource("/v1/deployments/{api}")
                    .route(web::get().to(app::deployments::get_deployments_for_api)),
            ) //TODO rework url
            //domain related APIs
            .service(app::domains::get_domains)
            .service(app::domains::get_domains_stats)
            .service(app::domains::create_domain)
            .service(app::domains::get_domains_errors)
            .service(web::scope("/v1/domains").service(
                web::resource("/{id}").route(web::delete().to(app::domains::delete_domain)),
            ))
            //APIs and Specs related endpoints
            //.route("/v1/endpoints", web::get().to(app::apis::get_endpoints))
            //.service(web::resource("/v1/endpoints/{api}").route(web::get().to(app::apis::get_endpoints)),)
            .service(app::apis::list_all_reviews)
            .service(app::apis::get_all_specs)
            .service(app::apis::create_api)
            .service(app::apis::list_all_apis)
            .service(
                web::scope("/v1/apis")
                    .service(web::resource("/{api}").route(web::get().to(app::apis::get_api_by_id)))
                    .service(
                        web::resource("/{api}/status")
                            .route(web::post().to(app::apis::update_api_status_by_id)),
                    )
                    .service(
                        web::resource("/{api}/tier")
                            .route(web::post().to(app::apis::update_api_tier_by_id)),
                    ),
            )
            //end related endpoints
            .service(app::envs::create_env)
            .service(app::envs::list_env)
            .service(web::resource("/v1/envs/{id}").route(web::get().to(app::envs::get_env)))
            //Tier related endpoints
            .service(app::tiers::create_tier)
            .service(app::tiers::get_tiers)
            //metrics related endpoints
            .service(app::metrics::get_all_metrics)
            .service(app::apis::get_oldest_pr)
            .service(app::apis::get_merged_pr)
            .service(app::metrics::refresh_metrics)
            .service(app::systems::get_all_systems)
            .service(app::systems::get_all_domains_per_system_and_layer)
            .service(app::domains::get_all_domains_for_all_catalogs)
            .service(app::catalogs::get_catalog_by_id)
            .service(app::catalogs::get_all_catalog)
            .service(app::catalogs::refresh_all_catalogs)
            .service(app::search::search_specs)
            .service(app::config::get_config_for_ui)
            .service(app::apis::get_all_errors)
            //Static resources mapping
            .service(actix_files::Files::new("/",  &SETTINGS.server.static_resources_path)
                .redirect_to_slash_directory()
                //.show_files_listing()
                .index_file("index.html"))
            //.route("/", web::get().to(index))
            //.route("/static/", web::get().to(index))
            //.route("/domains", web::get().to(index))
            //.route("/reviews", web::get().to(index))
            //.route("/apis", web::get().to(index))
            //.route("/envs", web::get().to(index))
            //.route("/layers", web::get().to(index))
            
            //keep it at last position (in URLs mappings)
            // .service(
            //     Files::new("/", &SETTINGS.server.static_resources_path).index_file("index.html"),
            // )
    })
    .workers(12)
    .bind(&SETTINGS.server.bind_adress)
    .unwrap()
    .run()
    .await
    .unwrap();
}
