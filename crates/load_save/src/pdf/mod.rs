mod error;
mod merge;
mod types;

use std::{
  collections::HashMap,
  fs::OpenOptions,
  io::{BufReader, Read},
  sync::{mpsc, Arc, Mutex},
  thread,
};

use async_trait::async_trait;
use handlebars::Handlebars;
use lazy_static::lazy_static;
use lopdf::Document;
use serde::Serialize;
use wkhtmltopdf::{Orientation, PageSize, PdfApplication, Size};

use ::config::DATA_DIR;
use ::types::ProviderMapping;

use self::{error::Result, types::PDFCaBr2Document};
use super::{
  error::Result as LoadSaveResult,
  types::{CaBr2Document, Saver},
};

pub use error::PdfError;

type PDFThreadChannels = Arc<Mutex<(mpsc::SyncSender<(String, String)>, mpsc::Receiver<Result<Vec<u8>>>)>>;

lazy_static! {
  pub static ref PROVIDER_MAPPING: Arc<Mutex<ProviderMapping>> = Arc::new(Mutex::new(HashMap::new()));
}

pub struct PDF;

impl PDF {
  pub fn new(provider_mapping: ProviderMapping) -> PDF {
    PROVIDER_MAPPING.lock().unwrap().extend(provider_mapping.into_iter());
    PDF
  }
}

#[async_trait]
impl Saver for PDF {
  async fn save_document(&self, document: CaBr2Document) -> LoadSaveResult<Vec<u8>> {
    lazy_static! {
      static ref PDF_THREAD_CHANNEL: PDFThreadChannels = Arc::new(Mutex::new(init_pdf_application()));
    }

    let title = document.header.document_title.clone();

    // PDF generation may be a long running, cpu intensive task. This informs the runtime to move other waiting tasks
    // to different threads.
    let pages = tokio::task::block_in_place(|| render_doc(document.into()))?;

    let channels = PDF_THREAD_CHANNEL.lock().unwrap();

    let mut pdfs = Vec::with_capacity(2);
    for page in pages {
      channels
        .0
        .send((page, title.clone()))
        .expect("sending data to pdf thread failed");

      let pdf: Vec<u8> = channels.1.recv().expect("receiving data from pdf thread failed")?;
      pdfs.push(pdf);
    }

    let mut documents = Vec::with_capacity(pdfs.len());
    for pdf in pdfs {
      match Document::load_mem(&pdf) {
        Ok(doc) => documents.push(doc),
        Err(_) => return Err(PdfError::PdfMergeError("loading pdf failed".into()).into()),
      }
    }

    let mut merged_pdf = merge::merge_pdfs(documents)?;

    let mut buf = Vec::new();
    merged_pdf.save_to(&mut buf)?;

    Ok(buf)
  }
}

const PDF_TEMPLATE_PAGES: [&str; 2] = ["first", "second"];

/// render_doc get CaBr2Document and return html
fn render_doc(document: PDFCaBr2Document) -> Result<Vec<String>> {
  #[derive(Debug, Serialize)]
  struct Context<'a> {
    stylesheet: &'a String,
    document: &'a PDFCaBr2Document,
  }

  lazy_static! {
    static ref REG: Arc<Mutex<Option<(String, Handlebars<'static>)>>> = Arc::new(Mutex::new(None));
  }

  let mut reg = REG.lock().unwrap();

  if reg.is_none() {
    *reg = Some(init_handlebars()?);
  }

  let reg = reg.as_ref().unwrap();
  let context = Context {
    stylesheet: &reg.0,
    document: &document,
  };
  Ok(vec![
    reg.1.render(PDF_TEMPLATE_PAGES[0], &context)?,
    reg.1.render(PDF_TEMPLATE_PAGES[1], &context)?,
  ])
}

#[inline]
fn init_handlebars() -> Result<(String, Handlebars<'static>)> {
  let mut reg = Handlebars::new();
  let mut template_path = DATA_DIR.clone();
  template_path.push("templates");

  for name in PDF_TEMPLATE_PAGES.iter() {
    let mut filename = template_path.clone();
    filename.push(name);
    let filename = filename.with_extension("html");

    log::trace!("template path: {:?}", filename);
    reg.register_template_file(name, filename)?;
  }

  let mut buf = String::new();
  template_path.push("styles.css");
  log::trace!("styles_ path: {:?}", template_path);
  let file = OpenOptions::new().read(true).open(template_path)?;
  let mut reader = BufReader::new(file);
  reader.read_to_string(&mut buf)?;

  reg.register_helper("ghs_symbols", Box::new(handlebar_helpers::ghs_symbols));
  reg.register_helper("h_p_phrases_numbers", Box::new(handlebar_helpers::h_p_phrases_numbers));
  reg.register_helper("h_p_phrases", Box::new(handlebar_helpers::h_p_phrases));
  reg.register_helper("value_or_dash", Box::new(handlebar_helpers::value_or_dash));
  reg.register_helper("providers", Box::new(handlebar_helpers::providers));

  Ok((buf, reg))
}

type PDFChannels = (mpsc::SyncSender<(String, String)>, mpsc::Receiver<Result<Vec<u8>>>);

fn init_pdf_application() -> PDFChannels {
  let (tauri_tx, pdf_rx) = mpsc::sync_channel(0);
  let (pdf_tx, tauri_rx) = mpsc::sync_channel(0);

  /* #region  pdf thread */

  thread::spawn(move || {
    log::debug!("[pdf_thread]: initializing pdf application");
    let pdf_app = match PdfApplication::new() {
      Ok(app) => app,
      Err(e) => {
        log::error!("[pdf_thread]: initialization of pdf application failed");
        pdf_tx
          .send(Err(PdfError::Wkhtml(e)))
          .expect("[pdf_thread]: pdf thread could not send data");
        return;
      }
    };

    loop {
      log::trace!("[pdf_thread]: waiting for html to convert");
      let (html, title) = pdf_rx.recv().expect("[pdf_thread]: pdf thread could not receive data");
      log::trace!("[pdf_thread]: got html");

      // needed for rust to resolve types
      let title: String = title;

      let mut buf = Vec::with_capacity(256_000); // just some random default value to minimize allocations

      let result = match pdf_app
        .builder()
        .page_size(PageSize::A4)
        .orientation(Orientation::Portrait)
        .margin(Size::Millimeters(15))
        .title(&title)
        .build_from_html(&html)
      {
        Ok(mut pdfout) => match pdfout.read_to_end(&mut buf) {
          Ok(_) => Ok(buf),
          Err(e) => Err(e.into()),
        },
        Err(e) => Err(e.into()),
      };

      log::trace!("[pdf_thread]: sending result");
      pdf_tx
        .send(result)
        .expect("[pdf_thread]: pdf thread could not send data");
      log::trace!("[pdf_thread]: finished");
    }
  });

  /* #endregion */

  (tauri_tx, tauri_rx)
}

/// Custom helpers for handlebars
mod handlebar_helpers {
  use std::{
    collections::BTreeSet,
    sync::{Arc, Mutex},
  };

  use handlebars::{Handlebars, JsonRender, RenderError};
  use lazy_static::lazy_static;

  use config::GHSSymbols;

  use super::{types::PDFSubstanceData, PROVIDER_MAPPING};

  lazy_static! {
    static ref GHS_SYMBOLS: Arc<Mutex<GHSSymbols>> = Arc::new(Mutex::new(get_hazard_symbols().unwrap_or_default()));
  }

  fn get_hazard_symbols() -> Result<GHSSymbols, impl std::error::Error> {
    // calling `block_on` is possible, because we set the current thread to block at the beginning of the PDF generation.
    tokio::runtime::Handle::current().block_on(config::get_hazard_symbols())
  }

  /// Inlines the actual ghs-symbol-images from their keys as base64-encodes pngs
  pub fn ghs_symbols(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
  ) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();
    out.write("<img class='ghs' src=\"")?;
    out.write(
      GHS_SYMBOLS
        .lock()
        .unwrap()
        .get(param.value().as_str().unwrap_or_default())
        .unwrap_or(&String::from(""))
        .as_str(),
    )?;
    out.write("\" alt=\"")?;
    out.write(&param.value().render())?; // alt content
    out.write("\" />")?;
    if let Some(index) = param.context_path().unwrap().last() {
      let index: u8 = index.parse()?;
      if index > 0u8 && index % 3 == 2 {
        out.write("<br/>")?;
      }
    }
    Ok(())
  }

  /// Writes numbers of h- or p-phrases
  pub fn h_p_phrases_numbers(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
  ) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();
    let phrases: Vec<(String, String)> = serde_json::from_value(param.value().clone())?;
    out.write(&phrases.into_iter().map(|p| p.0).collect::<Vec<String>>().join("-"))?;
    Ok(())
  }

  /// handlebar helper: write h- or p-phrases in the following style:
  ///
  /// - number1: text1
  /// - number2: text2
  /// - ...
  pub fn h_p_phrases(
    h: &handlebars::Helper,
    _: &Handlebars,
    ctx: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
  ) -> handlebars::HelperResult {
    #[inline]
    /// Returns a `Vec<String>` where the number and text are concatenated with a non-breaking space
    fn map_phrases(phrases: Vec<(String, String)>) -> Vec<String> {
      phrases.into_iter().map(|p| format!("{}:&nbsp;{}", p.0, p.1)).collect()
    }

    /// Helper enum for code below
    enum PhraseType {
      H,
      P,
    }

    let param = h.param(0).unwrap();
    let phrases_selector = match param.value().as_str().unwrap() {
      "h" => PhraseType::H,
      "p" => PhraseType::P,
      _ => return Err(RenderError::new(format!("unknown phrase type: {}", param.value()))),
    };

    let substances: Vec<PDFSubstanceData> =
      match serde_json::from_value(ctx.data()["document"]["substanceData"].clone()) {
        Ok(substances) => substances,
        Err(err) => return Err(RenderError::new(format!("json deserialize error: {:?}", err))),
      };

    let phrases: BTreeSet<String> = substances
      .into_iter()
      .map(|s| {
        map_phrases(match phrases_selector {
          PhraseType::H => s.h_phrases.data,
          PhraseType::P => s.p_phrases.data,
        })
      })
      .flatten()
      .collect();

    for p in phrases.iter() {
      out.write(p)?;
      out.write("<br/>")?;
    }
    Ok(())
  }

  /// Writes a `-` if the value is null, otherwise the value itself is written
  pub fn value_or_dash(
    h: &handlebars::Helper,
    _: &Handlebars,
    _: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
  ) -> handlebars::HelperResult {
    let param = h.param(0).unwrap();
    match param.value() {
      handlebars::JsonValue::Null => out.write("-")?,
      _ => out.write(&param.render())?,
    };
    Ok(())
  }

  /// Writes the `Set` of providers separated by `,`
  pub fn providers(
    _: &handlebars::Helper,
    _: &Handlebars,
    ctx: &handlebars::Context,
    _: &mut handlebars::RenderContext,
    out: &mut dyn handlebars::Output,
  ) -> handlebars::HelperResult {
    let substances: Vec<PDFSubstanceData> =
      match serde_json::from_value(ctx.data()["document"]["substanceData"].clone()) {
        Ok(substances) => substances,
        Err(err) => return Err(RenderError::new(format!("json deserialize error: {:?}", err))),
      };

    let providers: BTreeSet<String> = substances.into_iter().map(|s| s.source.provider).collect();

    let provider_mapping = PROVIDER_MAPPING.lock().unwrap();
    // kill empty string from empty substance lines
    for (i, provider) in providers
      .iter()
      .filter(|p| !(p.is_empty() || p.as_str() == "custom"))
      .map(|p| provider_mapping.get(p).unwrap())
      .enumerate()
    {
      if i > 0 {
        out.write(", ")?;
      }
      out.write(provider)?;
    }
    Ok(())
  }
}
