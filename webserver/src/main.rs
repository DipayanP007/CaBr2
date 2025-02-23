mod impls;

use std::fs;

use warp::Filter;

use crate::impls::{
  config,
  load_save::{self, CACHE_FOLDER, DOWNLOAD_FOLDER},
  search,
};

#[tokio::main]
pub async fn main() {
  // must be initialized first
  logger::setup_logger().await.unwrap();

  search::init().await;
  load_save::init(search::get_provider_mapping().await).await;

  // create tmp folders
  handle_result(fs::create_dir_all(DOWNLOAD_FOLDER));
  handle_result(fs::create_dir(CACHE_FOLDER));

  let search_available_providers = warp::path("availableProviders")
    .and(warp::path::end())
    .and(warp::get())
    .and_then(search::handle_available_providers);

  let search_suggestions = warp::path("suggestions")
    .and(warp::post())
    .and(warp::body::json())
    .and_then(search::handle_suggestions);

  let search_results = warp::path("results")
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json())
    .and_then(search::handle_results);

  let search_substances = warp::path("substances")
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json())
    .and_then(search::handle_substances);

  let search = warp::path("search")
    .and(search_available_providers.or(search_suggestions.or(search_results.or(search_substances))));

  let config_programversion = warp::path("programVersion")
    .and(warp::path::end())
    .and(warp::get())
    .and_then(config::handle_program_version);

  let config_hazard_symbols = warp::path("hazardSymbols")
    .and(warp::path::end())
    .and(warp::get())
    .and_then(config::handle_hazard_symbols);

  let config_available_languages = warp::path("availableLanguages")
    .and(warp::path::end())
    .and(warp::get())
    .and_then(config::handle_available_languages);

  let config_localized_strings = warp::path("localizedStrings")
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json())
    .and_then(config::handle_localized_strings);

  let config = warp::path("config")
    .and(config_programversion.or(config_hazard_symbols.or(config_available_languages.or(config_localized_strings))));

  let load_save_available_document_types = warp::path("availableDocumentTypes")
    .and(warp::path::end())
    .and(warp::get())
    .and_then(load_save::handle_available_document_types);

  let load_save_load_document = warp::path("loadDocument")
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json())
    .and_then(load_save::handle_load_document);

  let load_save_save_document = warp::path("saveDocument")
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json())
    .and_then(load_save::handle_save_document);

  let load_save = warp::path("loadSave").and(
    load_save_available_document_types
      .or(load_save_load_document)
      .or(load_save_save_document),
  );

  let downloads_folder = warp::path("download").and(warp::fs::dir(DOWNLOAD_FOLDER));

  let cors;
  let address;
  #[cfg(not(debug_assertions))]
  {
    cors = warp::cors()
      .allow_origin("https://app.cabr2.de")
      .allow_methods(vec!["GET", "POST"])
      .allow_headers(vec!["content-type"]);
    address = ([0, 0, 0, 0], 80);
  }
  #[cfg(debug_assertions)]
  {
    cors = warp::cors()
      .allow_origin("http://localhost:4200")
      .allow_methods(vec!["GET", "POST"])
      .allow_headers(vec!["content-type"]);
    address = ([127, 0, 0, 1], 3030);
  }

  let api = warp::path("api").and(warp::path("v1"));
  let routes = api.and(search.or(config.or(load_save))).with(cors).or(downloads_folder);

  /*
  /api/v1/..

  POST ../search/suggestions { provider, searchArgument: { searchType, pattern } }
  POST ../search/results { provider, searchArguments: { exact, arguments: []{ searchType, pattern } } }
  POST ../search/substances { provider, identifier }

  GET ../config/programVersion
  GET ../config/hazardSymbols
  POST ../config/promptHtml { name }

  GET ../config/availableLanguages
  POST ../config/localizedStrings { language }

  GET ../loadSave/availableDocumentTypes
  POST ../loadSave/saveDocument { fileType, document } // document: CaBr2Document
  POST ../loadSave/loadDocument { fileType, document } // document: String
  */

  log::info!("Starting cleanup thread...");
  tokio::spawn(load_save::cleanup_thread());

  log::info!("server starting...");
  // On debug builds it runs on `http://localhost:3030`,
  // on release builds it runs on port 80 and listens on every interface.
  warp::serve(routes).run(address).await;
}

/// handles a directory creation result, if the folder was created or already existed
/// everything is ok otherwise it logs the error and panics
fn handle_result(res: std::io::Result<()>) {
  res.unwrap_or_else(|err| match err.kind() {
    std::io::ErrorKind::AlreadyExists => {}
    _ => {
      log::error!("{:?}", err);
      panic!("")
    }
  });
}
