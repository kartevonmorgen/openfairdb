use crate::core::{entities::Entry, db::{EntryGateway, EntryIndexer, EntryIndex, EntryIndexQuery}, util::geo::{LatCoord, LngCoord}};

use failure::Fallible;
//use std::path::Path;
//use tempdir::TempDir;
use std::ops::Bound;
use tantivy::{Index, IndexWriter, Document, DocAddress, Score, tokenizer::{Tokenizer, LowerCaser, RawTokenizer}, query::{Occur, Query, TermQuery, RangeQuery, BooleanQuery, QueryParser}, collector::{Count, TopDocs}, schema::*};

struct TantivyEntryFields {
    id: Field,
    title: Field,
    description: Field,
    category: Field,
    lat: Field,
    lng: Field,
    tag: Field,
}

pub(crate) struct TantivyEntryIndex {
    fields: TantivyEntryFields,
    index: Index,
    writer: IndexWriter,
    text_query_parser: QueryParser,
}

impl TantivyEntryIndex {
    pub fn create() -> Fallible<Self> {
        let id_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("id")
                    .set_index_option(IndexRecordOption::Basic)
            )
            .set_stored();
        let category_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("id")
                    .set_index_option(IndexRecordOption::WithFreqs)
            );
        let tag_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("tag")
                    .set_index_option(IndexRecordOption::WithFreqs)
            );
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions)
            );
        let mut schema_builder = SchemaBuilder::default();
        let id = schema_builder.add_text_field("id", id_options);
        let lat = schema_builder.add_i64_field("lat", INT_INDEXED);
        let lng = schema_builder.add_i64_field("lng", INT_INDEXED);
        let title = schema_builder.add_text_field("title", text_options.clone());
        let description = schema_builder.add_text_field("desc", text_options);
        let category = schema_builder.add_text_field("cat", category_options.clone());
        let tag = schema_builder.add_text_field("tag", tag_options);
        let schema = schema_builder.build();
        // TODO: Create index on file system, either persistent or temporary
        let index = Index::create_in_ram(schema.clone());
        index.tokenizers().register("id", RawTokenizer);
        index.tokenizers().register("tag", RawTokenizer.filter(LowerCaser));

        let writer = index.writer(50_000_000)?;
        let text_query_parser = QueryParser::for_index(&index, vec![title, description]);
        Ok(Self {
            fields: TantivyEntryFields {
                id,
                lat,
                lng,
                title,
                description,
                category,
                tag,
            },
            index,
            writer,
            text_query_parser,
        })
    }
}

impl EntryIndexer for TantivyEntryIndex {
    fn add_or_update_entry(&mut self, entry: &Entry) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, &entry.id);
        self.writer.delete_term(id_term);
        let mut doc = Document::default();
        doc.add_text(self.fields.id, &entry.id);
        doc.add_i64(self.fields.lat, i64::from(LatCoord::from_deg(entry.location.lat).to_raw()));
        doc.add_i64(self.fields.lng, i64::from(LngCoord::from_deg(entry.location.lng).to_raw()));
        doc.add_text(self.fields.title, &entry.title);
        doc.add_text(self.fields.description, &entry.description);
        for category in &entry.categories {
            doc.add_text(self.fields.category, category);
        }
        for tag in &entry.tags {
            doc.add_text(self.fields.tag, tag);
        }
        self.writer.add_document(doc);
        Ok(())
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, id);
        self.writer.delete_term(id_term);
        Ok(())
    }

    fn flush(&mut self) -> Fallible<()> {
        self.writer.commit()?;
        self.index.load_searchers()?;
        Ok(())
    }
}

// Upper bound for the expected number of results for which
// memory is pre-allocated.
const MAX_LIMIT: usize = 100;

impl EntryIndex for TantivyEntryIndex {
    fn query_entries(&self, entries: &EntryGateway, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<Entry>> {
        let mut sub_queries: Vec<(Occur, Box<Query>)> = Vec::with_capacity(2 + 2 + query.categories.len() + query.tags.len());
        if let Some(ref bbox) = query.bbox {
            debug_assert!(bbox.is_valid());
            debug_assert!(!bbox.is_empty());
            let lat_query = RangeQuery::new_i64_bounds(
                self.fields.lat,
                Bound::Included(i64::from(bbox.south_west().lat().to_raw())),
                Bound::Included(i64::from(bbox.north_east().lat().to_raw())));
            sub_queries.push((Occur::Must, Box::new(lat_query)));
            if bbox.south_west().lng() <= bbox.north_east().lng() {
                // regular (inclusive)
                let lng_query = RangeQuery::new_i64_bounds(
                    self.fields.lng,
                    Bound::Included(i64::from(bbox.south_west().lng().to_raw())),
                    Bound::Included(i64::from(bbox.north_east().lng().to_raw())));
                sub_queries.push((Occur::Must, Box::new(lng_query)));
            } else {
                // inverse (exclusive)
                let lng_query = RangeQuery::new_i64_bounds(
                    self.fields.lng,
                    Bound::Excluded(i64::from(bbox.north_east().lng().to_raw())),
                    Bound::Excluded(i64::from(bbox.south_west().lng().to_raw())));
                sub_queries.push((Occur::MustNot, Box::new(lng_query)));
            }
        }
        if let Some(ref text) = query.text {
            debug_assert!(!text.trim().is_empty());
            match self.text_query_parser.parse_query(&text.to_lowercase()) {
                Ok(query) => {
                    sub_queries.push((Occur::Should, Box::new(query)));
                }
                Err(err) => {
                    warn!("Failed to parse query text '{}': {:?}", text, err);
                }
            }
        }
        for category in &query.categories {
            debug_assert!(!category.trim().is_empty());
            let category_term = Term::from_field_text(self.fields.category, &category.to_lowercase());
            let category_query = TermQuery::new(category_term, IndexRecordOption::Basic);
            sub_queries.push((Occur::Must, Box::new(category_query)));
        }
        for tag in &query.tags {
            debug_assert!(!tag.trim().is_empty());
            let tag_term = Term::from_field_text(self.fields.tag, &tag.to_lowercase());
            let tag_query = TermQuery::new(tag_term, IndexRecordOption::Basic);
            sub_queries.push((Occur::Should, Box::new(tag_query)));
        }
        let query = BooleanQuery::from(sub_queries);
        let searcher = self.index.searcher();
        let (_doc_count, top_docs): (usize, Vec<(Score, DocAddress)>) =
            searcher.search(&query, &(Count, TopDocs::with_limit(limit.min(MAX_LIMIT))))?;
        let mut top_entries = Vec::with_capacity(top_docs.len());
        for (_score, doc_addr) in top_docs {
            match searcher.doc(doc_addr) {
                Ok(doc) => {
                    if let Some(id) = doc.get_first(self.fields.id).and_then(Value::text) {
                        match entries.get_entry(id) {
                            Ok(entry) => {
                                top_entries.push(entry);
                            }
                            Err(err) => {
                                warn!("Entry {} not found: {}", id, err);
                            }
                        }
                    } else {
                        warn!("Mising entry id in document {:?}", doc_addr);
                    }
                }
                Err(err) => {
                    warn!("Failed to load document {:?}: {}", doc_addr, err);
                }
            }
        }
        Ok(top_entries)
    }
}