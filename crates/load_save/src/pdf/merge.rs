use std::collections::BTreeMap;

use lopdf::{Document, Object, ObjectId};

use super::error::{PdfError, Result};

/// this code is basically the example from the
/// [lopdf repo](https://github.com/J-F-Liu/lopdf/blob/master/examples/merge.rs)
/// with minor changes to make it work as library.
pub fn merge_pdfs(documents: Vec<Document>) -> Result<Document> {
  // Define a starting max_id (will be used as start index for object_ids)
  let mut max_id = 1;

  // Collect all Documents Objects grouped by a map
  let mut documents_pages = BTreeMap::new();
  let mut documents_objects = BTreeMap::new();

  for mut document in documents {
    document.renumber_objects_with(max_id);

    max_id = document.max_id + 1;

    documents_pages.extend(
      document
        .get_pages()
        .into_iter()
        .map(|(_, object_id)| (object_id, document.get_object(object_id).unwrap().to_owned()))
        .collect::<BTreeMap<ObjectId, Object>>(),
    );
    documents_objects.extend(document.objects);
  }

  // Initialize a new empty document
  let mut document = Document::with_version("1.5");

  // Catalog and Pages are mandatory
  let mut catalog_object: Option<(ObjectId, Object)> = None;
  let mut pages_object: Option<(ObjectId, Object)> = None;

  // Process all objects except "Page" type
  for (object_id, object) in documents_objects.iter() {
    // We have to ignore "Page" (as are processed later), "Outlines" and "Outline" objects
    // All other objects should be collected and inserted into the main Document
    match object.type_name().unwrap_or("") {
      "Catalog" => {
        // Collect a first "Catalog" object and use it for the future "Pages"
        catalog_object = Some((
          if let Some((id, _)) = catalog_object {
            id
          } else {
            *object_id
          },
          object.clone(),
        ));
      }
      "Pages" => {
        // Collect and update a first "Pages" object and use it for the future "Catalog"
        // We have also to merge all dictionaries of the old and the new "Pages" object
        if let Ok(dictionary) = object.as_dict() {
          let mut dictionary = dictionary.clone();
          if let Some((_, ref object)) = pages_object {
            if let Ok(old_dictionary) = object.as_dict() {
              dictionary.extend(old_dictionary);
            }
          }

          pages_object = Some((
            if let Some((id, _)) = pages_object {
              id
            } else {
              *object_id
            },
            Object::Dictionary(dictionary),
          ));
        }
      }
      "Page" => {}     // Ignored, processed later and separately
      "Outlines" => {} // Ignored, not supported yet
      "Outline" => {}  // Ignored, not supported yet
      _ => {
        document.objects.insert(*object_id, object.clone());
      }
    }
  }

  // If no "Pages" found abort
  if pages_object.is_none() {
    return Err(PdfError::PdfMergeError("Pages root not found.".into()));
  }

  // Iter over all "Page" and collect with the parent "Pages" created before
  for (object_id, object) in documents_pages.iter() {
    if let Ok(dictionary) = object.as_dict() {
      let mut dictionary = dictionary.clone();
      dictionary.set("Parent", pages_object.as_ref().unwrap().0);

      document.objects.insert(*object_id, Object::Dictionary(dictionary));
    }
  }

  // If no "Catalog" found abort
  if catalog_object.is_none() {
    return Err(PdfError::PdfMergeError("Catalog root not found.".into()));
  }

  let catalog_object = catalog_object.unwrap();
  let pages_object = pages_object.unwrap();

  // Build a new "Pages" with updated fields
  if let Ok(dictionary) = pages_object.1.as_dict() {
    let mut dictionary = dictionary.clone();

    // Set new pages count
    dictionary.set("Count", documents_pages.len() as u32);

    // Set new "Kids" list (collected from documents pages) for "Pages"
    dictionary.set(
      "Kids",
      documents_pages
        .into_iter()
        .map(|(object_id, _)| Object::Reference(object_id))
        .collect::<Vec<_>>(),
    );

    document.objects.insert(pages_object.0, Object::Dictionary(dictionary));
  }

  // Build a new "Catalog" with updated fields
  if let Ok(dictionary) = catalog_object.1.as_dict() {
    let mut dictionary = dictionary.clone();
    dictionary.set("Pages", pages_object.0);
    dictionary.remove(b"Outlines"); // Outlines not supported in merged PDFs

    document
      .objects
      .insert(catalog_object.0, Object::Dictionary(dictionary));
  }

  document.trailer.set("Root", catalog_object.0);

  // Update the max internal ID as wasn't updated before due to direct objects insertion
  document.max_id = document.objects.len() as u32;

  // Reorder all new Document objects
  document.renumber_objects();
  document.compress();

  Ok(document)
}
