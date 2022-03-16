extern crate failure;
extern crate rusqlite;
extern crate time;
extern crate uuid;

use uuid::Uuid;

use rusqlite::NO_PARAMS;
use rusqlite::{params, Connection, Result};

//use rustbreak::{FileDatabase, deser::Ron};
use log::{debug, error, info};

use serde::{Deserialize, Serialize};

#[path = "../../settings/mod.rs"]
mod settings;
use settings::Settings;

lazy_static! {
    static ref SETTINGS: settings::Settings = Settings::new().unwrap();
}

/**
 * "public" part
 */
pub trait DomainRepo {
    fn list_all_domains(
        &self,
        config: &super::super::settings::Database,
    ) -> Result<Vec<DomainItem>>;

    fn add_domain(
        &self,
        config: &super::super::settings::Database,
        name: &str,
        description: &str,
        owner: &str,
    ) -> Result<Uuid>;

    fn get_domain(&self, config: &super::super::settings::Database, id: Uuid)
        -> Result<DomainItem>;

    fn delete_domain(&self, config: &super::super::settings::Database, id: Uuid) -> Result<()>;
}

//
pub struct DomainItem {
    pub name: std::string::String,
    pub id: Uuid,
    pub description: String,
    pub owner: String,
}

pub enum DomainImplType {
    YamlBasedDomainRepo,
    DbBasedDomainRepo,
}

pub struct DomainImplFactory;
impl DomainImplFactory {
    pub fn get_impl() -> Box<dyn DomainRepo> {
        let domain_type = &SETTINGS.domain_repo_type.domain_impl;
        info!("Using Domain Impl {}", &domain_type);

        match domain_type.as_str() {
            "YamlBasedDomainRepo" => Box::new(YamlBasedDomainRepo {}),
            "DbBasedDomainRepo" => Box::new(DbBasedDomainRepo {}),
            _ => Box::new(DbBasedDomainRepo {}),
        }
    }

    pub fn is_read_only() -> bool {
        let domain_type = &SETTINGS.domain_repo_type.domain_impl;

        match domain_type.as_str() {
            "YamlBasedDomainRepo" => true,
            "DbBasedDomainRepo" => false,
            _ => true,
        }
    }
}

/**
 * implementation based on yaml file
 */
pub struct YamlBasedDomainRepo {
    // domain_catalog: YamlBasedDomainCatalog,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct YamlBasedDomainCatalog {
    #[serde(rename(serialize = "softwaredomains", deserialize = "softwaredomains"))]
    software_domains: Vec<YamlBasedDomainCatalogItem>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct YamlBasedDomainCatalogItem {
    id: String,
    #[serde(rename(serialize = "subdomains", deserialize = "subdomains"))]
    subdomains: Option<Vec<YamlBasedDomainCatalogItem>>,
}

// impl Default for YamlBasedDomainRepo {
//     fn default() -> Self {
//         Self {}
//     }
// }

impl YamlBasedDomainRepo {
    fn get_domain_catalog() -> YamlBasedDomainCatalog {
        //TODO find a way to not reload from file every time this method is called

        let f = match std::fs::File::open(&SETTINGS.domain_repo_type.domain_catalog_path) {
            Ok(f) => {
                info!(
                    "has loaded Domain Catalog from Yaml file {:?}",
                    SETTINGS.domain_repo_type.domain_catalog_path
                );
                f
            }
            Err(err) => {
                error!(
                    "failed loading Yml Catalog from {:?} - {:?}",
                    SETTINGS.domain_repo_type.domain_catalog_path, err
                );
                panic!(
                    "failed loading Yml Catalog from {:?} - {:?}",
                    SETTINGS.domain_repo_type.domain_catalog_path, err
                );
            }
        };

        let yaml_struct = serde_yaml::from_reader(f);
        match yaml_struct {
            Ok(yaml) => yaml,
            Err(err) => {
                panic!("Unable to load Yaml Struct from file - {:?}", err);
            }
        }
    }

    fn get_flat_list(software_domains: &Vec<YamlBasedDomainCatalogItem>) -> Vec<String> {
        let mut buf: Vec<String> = Vec::new();

        for domain in software_domains {
            let mut domain_str = String::new();
            domain_str.push_str("/");
            domain_str.push_str(domain.id.as_str());
            debug!("got domain {:?}", &domain_str);

            buf.push(String::from(&domain_str));

            match &domain.subdomains {
                None => {
                    debug!("no subdomain");
                    buf.push(domain_str);
                }
                Some(subdomains) => {
                    let buf_2 = YamlBasedDomainRepo::get_flat_list(&subdomains);

                    for item in buf_2 {
                        let mut s = String::new();
                        s.push_str(domain_str.as_str());
                        s.push_str(item.as_str());

                        buf.push(s);
                    }
                }
            }
        }

        buf
    }
}

impl DomainRepo for YamlBasedDomainRepo {
    fn list_all_domains(
        &self,
        _config: &super::super::settings::Database,
    ) -> Result<Vec<DomainItem>> {
        let mut tuples = Vec::new();

        let catalog: YamlBasedDomainCatalog = YamlBasedDomainRepo::get_domain_catalog();

        let tmp = YamlBasedDomainRepo::get_flat_list(&catalog.software_domains);
        debug!("list of all (flat) domains {:?}", tmp);

        for domain in tmp {
            let domain = DomainItem {
                id: Uuid::new_v4(),
                name: String::from(domain),
                description: String::from("N/A"),
                owner: String::from("N/A"),
            };
            tuples.push(domain);
        }

        Ok(tuples)
    }

    fn add_domain(
        &self,
        _config: &super::super::settings::Database,
        _name: &str,
        _description: &str,
        _owner: &str,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();

        Ok(id)
    }

    fn get_domain(
        &self,
        _config: &super::super::settings::Database,
        _id: Uuid,
    ) -> Result<DomainItem> {
        info!("OLIV get_domain ");

        let id = Uuid::new_v4();
        Ok(DomainItem {
            name: String::from("name"),
            id: id,
            description: String::from("description"),
            owner: String::from("owner"),
        })
    }

    fn delete_domain(&self, _config: &super::super::settings::Database, _id: Uuid) -> Result<()> {
        Ok(())
    }
}

/**
 *implementation based on DB
 */
pub struct DbBasedDomainRepo {}

impl DomainRepo for DbBasedDomainRepo {
    //
    fn list_all_domains(
        &self,
        config: &super::super::settings::Database,
    ) -> Result<Vec<DomainItem>> {
        let mut db_path = String::from(&config.rusqlite_path);
        db_path.push_str("/apis-catalog-all.db");
        {
            debug!("Reading all domains from Domain_Database [{:?}]", db_path);
        }

        let conn = Connection::open(db_path)?;

        let mut stmt = conn.prepare("SELECT id, name, description, owner FROM domains")?;
        let mut rows = stmt.query(NO_PARAMS)?;

        let mut tuples = Vec::new();
        while let Some(row) = rows.next()? {
            let id = row.get("id")?;
            let name = row.get("name")?;
            let descripton = row.get("description")?;
            let owner = row.get("owner")?;
            let domain = DomainItem {
                id: id,
                name: name,
                description: descripton,
                owner: owner,
            };

            tuples.push(domain);
        }

        Ok(tuples)
    }

    fn add_domain(
        &self,
        config: &super::super::settings::Database,
        name: &str,
        description: &str,
        owner: &str,
    ) -> Result<Uuid> {
        let mut db_path = String::from(&config.rusqlite_path);
        db_path.push_str("/apis-catalog-all.db");
        {
            debug!(
                "Creating domain [{}] into Domain_Database [{:?}]",
                name, db_path
            );
        }

        let conn = Connection::open(db_path)?;

        let id = Uuid::new_v4();
        conn.execute(
            "INSERT INTO domains (id, name, description, owner) VALUES (?1, ?2, ?3, ?4)",
            params![id, name, description, owner],
        )?;

        conn.close().unwrap();
        Ok(id)
    }

    fn get_domain(
        &self,
        config: &super::super::settings::Database,
        id: Uuid,
    ) -> Result<DomainItem> {
        let mut db_path = String::from(&config.rusqlite_path);
        db_path.push_str("/apis-catalog-all.db");
        {
            debug!("Get domain [{}] into Domain_Database [{:?}]", id, db_path);
        }

        let conn = Connection::open(db_path)?;

        let mut stmt = conn.prepare("SELECT id, name, description FROM domains WHERE id = ?1")?;
        let row = stmt.query_row(params![id], |row| {
            Ok(DomainItem {
                name: row.get(1)?,
                id: row.get(0)?,
                description: row.get(2)?,
                owner: row.get(3)?,
            })
        })?;

        Ok(row)
    }

    fn delete_domain(&self, config: &super::super::settings::Database, id: Uuid) -> Result<()> {
        let mut db_path = String::from(&config.rusqlite_path);
        db_path.push_str("/apis-catalog-all.db");
        {
            debug!(
                "Delete domain [{}] into Domain_Database [{:?}]",
                id, db_path
            );
        }

        let conn = Connection::open(db_path)?;

        let mut stmt = conn.prepare("DELETE FROM domains where id = ?1")?;
        stmt.execute(params![id])?;

        Ok(())
    }
}
