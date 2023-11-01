extern crate tantivy;
use crate::app::dao::catalog::spec::SpecItem;
use std::time::Instant;
use std::path::Path;
use std::fs;
use tantivy::schema::*;
use tantivy::directory::MmapDirectory;
use tantivy::query::QueryParser;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tantivy::collector::TopDocs;
use tantivy::doc;
use serde::{Serialize, Deserialize};
use log::{debug, info, warn, error};

pub fn build_index(index_path: &str, specs: &Vec<SpecItem>) -> tantivy::Result<()> {
    info!("Building Index in folder [{}]", index_path);

    let now = Instant::now();

    if let Err(err) = fs::create_dir_all(&index_path) {
        error!("Failed to create directory {:?} - {:?}", &index_path, err);
    }

    let index_path = Path::new(index_path);


    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("audience", TEXT | STORED);
    schema_builder.add_text_field("domain", TEXT | STORED);
    schema_builder.add_text_field("systems", TEXT | STORED);
    schema_builder.add_text_field("layer", TEXT | STORED);
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("operations", TEXT | STORED);
    schema_builder.add_text_field("summary", TEXT);
    schema_builder.add_text_field("description", TEXT);
    schema_builder.add_text_field("catalog_id", TEXT | STORED);
    schema_builder.add_text_field("spec_path", TEXT | STORED);
    schema_builder.add_text_field("version", TEXT | STORED);
    //TODO schema_builder.add_text_field("system", TEXT);
    let schema = schema_builder.build();

    let mmap_directory = MmapDirectory::open(index_path)?;
    let index = Index::open_or_create(mmap_directory, schema.clone())?; // should use open_or_create to not overwrite existing index.
    let mut index_writer = index.writer(100_000_000)?; //multi threaded behind the scene # of thread < 8

    index_writer.delete_all_documents()?;
    index_writer.commit()?;

    let audience = schema.get_field("audience").unwrap();
    let domain = schema.get_field("domain").unwrap();
    let systems = schema.get_field("systems").unwrap();
    let layer = schema.get_field("layer").unwrap();
    let path = schema.get_field("path").unwrap();
    let operations = schema.get_field("operations").unwrap();
    let summary = schema.get_field("summary").unwrap();
    let description = schema.get_field("description").unwrap();
    let catalog_id = schema.get_field("catalog_id").unwrap();
    let spec_path = schema.get_field("spec_path").unwrap();
    let spec_version = schema.get_field("version").unwrap();

    //  will index all specs
    for spec in specs {
        let systems_as_text = &spec.get_systems().join(" ");
        let paths = &spec.get_paths();
        for path_item in paths.iter() {

            let mut ope_summary = String::from("");
            let mut ope_description = String::from("");
            let mut ope_methods = String::from("");

            for path_method in &path_item.methods{
                ope_summary.push_str( path_method.summary.as_str() );
                ope_summary.push_str( " " );
                ope_description.push_str( path_method.description.as_str()  );
                ope_description.push_str( " " );
                ope_methods.push_str( path_method.method.as_str() );
                ope_methods.push_str( " " );
            }
            //add the doc to the index 
            index_writer.add_document(doc!(
                audience => String::from( &spec.get_audience() ),
                domain => String::from( SpecItem::get_domain(&spec.spec_handler) ), 
                systems => String::from(systems_as_text),
                layer => String::from( &spec.get_layer() ),
                path => String::from(&path_item.path),
                operations => ope_methods,
                summary => ope_summary,
                description => ope_description,
                catalog_id => String::from(spec.get_catalog_id()), 
                spec_path => String::from(spec.get_file_path()),
                spec_version => String::from(spec.get_version()),
            )).ok();
        }
    }
    index_writer.commit()?;
    info!("Indexing Time [{}] milli seconds", now.elapsed().as_millis());

    Ok(())
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SearchResult {
    pub audience: [String; 1],
    pub domain: [String; 1],
    pub systems: [String; 1],
    pub layer: [String; 1],
    pub path: [String; 1],
    pub operations: [String; 1],
    pub catalog_id: [String; 1],
    pub spec_path: [String; 1],
    pub version: [String; 1],
}

pub fn search(index_path: &str, query_as_string: String, limit: usize) -> tantivy::Result<Vec<SearchResult>> {
    info!("Searching [{}] based on Index in folder [{}]", query_as_string, index_path);

    let index_path = Path::new(&index_path);
    let mmap_directory = MmapDirectory::open(index_path)?;
    //println!("file exist {}", Index::exists(&mmap_directory) );
    let index = Index::open(mmap_directory)?;
    //
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("audience", TEXT | STORED);
    schema_builder.add_text_field("domain", TEXT | STORED);
    schema_builder.add_text_field("systems", TEXT | STORED);
    schema_builder.add_text_field("layer", TEXT | STORED);
    schema_builder.add_text_field("path", TEXT | STORED);
    schema_builder.add_text_field("operations", TEXT | STORED);
    schema_builder.add_text_field("summary", TEXT);
    schema_builder.add_text_field("description", TEXT);
    schema_builder.add_text_field("catalog_id", TEXT | STORED);
    schema_builder.add_text_field("spec_path", TEXT | STORED);
    schema_builder.add_text_field("version", TEXT | STORED);
    
    let schema = schema_builder.build();

    let audience = schema.get_field("audience").unwrap();
    let domain = schema.get_field("domain").unwrap();
    let systems = schema.get_field("systems").unwrap();
    let layer = schema.get_field("layer").unwrap();
    let path = schema.get_field("path").unwrap();
    let operations = schema.get_field("operations").unwrap();
    let summary = schema.get_field("summary").unwrap();
    let description = schema.get_field("description").unwrap();
    let catalog_id = schema.get_field("catalog_id").unwrap();
    let spec_path = schema.get_field("spec_path").unwrap();
    let spec_version = schema.get_field("version").unwrap();

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![audience, domain, systems, layer, path, operations, summary, description, catalog_id, spec_path, spec_version]);
    let query = match query_parser.parse_query(&query_as_string){
        Ok(e) => e,
        Err(why) => panic!("Search | Error while parsing {:?}", why),
    };
    let docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

    //
    let mut results = std::vec::Vec::new();
    for (_score, doc) in docs {
        let retrieved_doc = searcher.doc(doc)?;
        debug!("Found doc [{}]",schema.to_json(&retrieved_doc));

        let doc_as_json = schema.to_json(&retrieved_doc);
        let search_result: SearchResult = serde_json::from_str(&doc_as_json).unwrap();
        results.push(search_result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use std::env;

    #[test]
    fn test_build_and_search_index() {
        let mut dir = env::temp_dir();
        dir.push("apis-catalog-test");

        println!("** test - will use {:?} dir for index", dir);

        let binding = &dir.into_os_string();
        let index_path = binding.to_str().unwrap();

        let specs = crate::app::dao::catalog::tests::get_mocked_specs();

        let _ = super::build_index(index_path, &specs);

        let mut returned_value = super::search(index_path, String::from("resource"), 10).unwrap();
        assert_eq!(returned_value.len(), 1);

        returned_value = super::search(index_path, String::from("resource AND summary:perRRRRr"), 10).unwrap();
        assert_eq!(returned_value.len(), 0);

    }
}