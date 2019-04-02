use crate::core::{
    db::{EntryIndex, EntryIndexQuery, EntryIndexer, IndexedEntry},
    entities::{AvgRatingValue, AvgRatings, Entry},
    util::geo::{LatCoord, LngCoord, MapPoint, RawCoord},
};

use failure::{bail, Fallible};
use std::{
    ops::Bound,
    path::Path,
    sync::{Arc, Mutex},
};
use tantivy::{
    collector::TopDocs,
    query::{BooleanQuery, Occur, Query, QueryParser, RangeQuery, TermQuery},
    schema::*,
    tokenizer::{LowerCaser, RawTokenizer, Tokenizer},
    DocAddress, Document, Index, IndexReader, IndexWriter, ReloadPolicy,
};

const OVERALL_INDEX_HEAP_SIZE_IN_BYTES: usize = 50_000_000;

struct IndexedEntryFields {
    id: Field,
    lat: Field,
    lng: Field,
    title: Field,
    description: Field,
    address_street: Field,
    address_city: Field,
    address_zip: Field,
    address_country: Field,
    category: Field,
    tag: Field,
    ratings_diversity: Field,
    ratings_fairness: Field,
    ratings_humanity: Field,
    ratings_renewable: Field,
    ratings_solidarity: Field,
    ratings_transparency: Field,
    total_rating: Field,
}

impl IndexedEntryFields {
    fn build_schema() -> (Self, Schema) {
        let id_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(ID_TOKENIZER)
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored();
        let category_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(ID_TOKENIZER)
                    .set_index_option(IndexRecordOption::WithFreqs),
            )
            .set_stored();
        let tag_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(TAG_TOKENIZER)
                    .set_index_option(IndexRecordOption::WithFreqs),
            )
            .set_stored();
        let address_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(TEXT_TOKENIZER)
                    .set_index_option(IndexRecordOption::WithFreqs),
            )
            // Address fields currently don't need to be stored
            //.set_stored()
            ;
        let text_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(TEXT_TOKENIZER)
                    .set_index_option(IndexRecordOption::WithFreqsAndPositions),
            )
            .set_stored();
        let mut schema_builder = SchemaBuilder::default();
        let fields = Self {
            id: schema_builder.add_text_field("id", id_options),
            lat: schema_builder.add_i64_field("lat", INDEXED | STORED),
            lng: schema_builder.add_i64_field("lng", INDEXED | STORED),
            title: schema_builder.add_text_field("title", text_options.clone()),
            description: schema_builder.add_text_field("description", text_options.clone()),
            address_street: schema_builder
                .add_text_field("address_street", address_options.clone()),
            address_city: schema_builder.add_text_field("address_city", address_options.clone()),
            address_zip: schema_builder.add_text_field("address_zip", address_options.clone()),
            address_country: schema_builder
                .add_text_field("address_country", address_options.clone()),
            category: schema_builder.add_text_field("category", category_options.clone()),
            tag: schema_builder.add_text_field("tag", tag_options.clone()),
            ratings_diversity: schema_builder.add_u64_field("ratings_diversity", STORED),
            ratings_fairness: schema_builder.add_u64_field("ratings_fairness", STORED),
            ratings_humanity: schema_builder.add_u64_field("ratings_humanity", STORED),
            ratings_renewable: schema_builder.add_u64_field("ratings_renewable", STORED),
            ratings_solidarity: schema_builder.add_u64_field("ratings_solidarity", STORED),
            ratings_transparency: schema_builder.add_u64_field("ratings_transparency", STORED),
            total_rating: schema_builder.add_u64_field("total_rating", STORED | FAST),
        };
        (fields, schema_builder.build())
    }

    fn read_document(&self, doc: &Document) -> IndexedEntry {
        let mut lat: Option<LatCoord> = Default::default();
        let mut lng: Option<LngCoord> = Default::default();
        let mut entry = IndexedEntry::default();
        entry.categories.reserve(4);
        entry.tags.reserve(32);
        for field_value in doc.field_values() {
            match field_value {
                fv if fv.field() == self.lat => {
                    debug_assert!(lat.is_none());
                    let raw_val = fv.value().i64_value();
                    debug_assert!(raw_val >= LatCoord::min().to_raw().into());
                    debug_assert!(raw_val <= LatCoord::max().to_raw().into());
                    lat = Some(LatCoord::from_raw(raw_val as RawCoord));
                }
                fv if fv.field() == self.lng => {
                    debug_assert!(lng.is_none());
                    let raw_val = fv.value().i64_value();
                    debug_assert!(raw_val >= LngCoord::min().to_raw().into());
                    debug_assert!(raw_val <= LngCoord::max().to_raw().into());
                    lng = Some(LngCoord::from_raw(raw_val as RawCoord));
                }
                fv if fv.field() == self.id => {
                    debug_assert!(entry.id.is_empty());
                    if let Some(id) = fv.value().text() {
                        entry.id = id.into();
                    } else {
                        error!("Invalid id value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.title => {
                    debug_assert!(entry.title.is_empty());
                    if let Some(title) = fv.value().text() {
                        entry.title = title.into();
                    } else {
                        error!("Invalid title value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.description => {
                    debug_assert!(entry.description.is_empty());
                    if let Some(description) = fv.value().text() {
                        entry.description = description.into();
                    } else {
                        error!("Invalid description value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.category => {
                    if let Some(category) = fv.value().text() {
                        entry.categories.push(category.into());
                    } else {
                        error!("Invalid category value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.tag => {
                    if let Some(tag) = fv.value().text() {
                        entry.tags.push(tag.into());
                    } else {
                        error!("Invalid tag value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.ratings_diversity => {
                    debug_assert!(entry.ratings.diversity == Default::default());
                    entry.ratings.diversity = u64_to_avg_rating(fv.value().u64_value());
                }
                fv if fv.field() == self.ratings_fairness => {
                    debug_assert!(entry.ratings.fairness == Default::default());
                    entry.ratings.fairness = u64_to_avg_rating(fv.value().u64_value());
                }
                fv if fv.field() == self.ratings_humanity => {
                    debug_assert!(entry.ratings.humanity == Default::default());
                    entry.ratings.humanity = u64_to_avg_rating(fv.value().u64_value());
                }
                fv if fv.field() == self.ratings_renewable => {
                    debug_assert!(entry.ratings.renewable == Default::default());
                    entry.ratings.renewable = u64_to_avg_rating(fv.value().u64_value());
                }
                fv if fv.field() == self.ratings_solidarity => {
                    debug_assert!(entry.ratings.solidarity == Default::default());
                    entry.ratings.solidarity = u64_to_avg_rating(fv.value().u64_value());
                }
                fv if fv.field() == self.ratings_transparency => {
                    debug_assert!(entry.ratings.transparency == Default::default());
                    entry.ratings.transparency = u64_to_avg_rating(fv.value().u64_value());
                }
                fv if fv.field() == self.total_rating => (),
                // Address fields are currently not stored
                //fv if fv.field() == self.address_street => (),
                //fv if fv.field() == self.address_city => (),
                //fv if fv.field() == self.address_zip => (),
                //fv if fv.field() == self.address_country => (),
                fv => {
                    error!("Unexpected field value: {:?}", fv);
                }
            }
        }
        if let (Some(lat), Some(lng)) = (lat, lng) {
            entry.pos = MapPoint::new(lat, lng);
        } else {
            error!("Invalid position: lat = {:?}, lng = {:?}", lat, lng);
        }
        entry
    }
}

pub(crate) struct TantivyEntryIndex {
    fields: IndexedEntryFields,
    index: Index,
    index_reader: IndexReader,
    index_writer: IndexWriter,
    text_query_parser: QueryParser,
}

const ID_TOKENIZER: &str = "raw";
const TAG_TOKENIZER: &str = "tag";
const TEXT_TOKENIZER: &str = "default";

fn register_tokenizers(index: &Index) {
    // Predefined tokenizers
    debug_assert!(index.tokenizers().get(ID_TOKENIZER).is_some());
    debug_assert!(index.tokenizers().get(TEXT_TOKENIZER).is_some());
    // Custom tokenizer(s)
    debug_assert!(index.tokenizers().get(TAG_TOKENIZER).is_none());
    index
        .tokenizers()
        .register(TAG_TOKENIZER, RawTokenizer.filter(LowerCaser));
}

fn f64_to_u64(val: f64, min: f64, max: f64) -> u64 {
    debug_assert!(val >= min);
    debug_assert!(val <= max);
    debug_assert!(min < max);
    if (val - max).abs() <= std::f64::EPSILON {
        u64::max_value()
    } else if (val - min).abs() <= std::f64::EPSILON {
        0u64
    } else {
        let norm = (val.max(min).min(max) - min) / (max - min);
        let mapped = u64::max_value() as f64 * norm;
        mapped.round() as u64
    }
}

fn u64_to_f64(val: u64, min: f64, max: f64) -> f64 {
    debug_assert!(min < max);
    if val == u64::max_value() {
        max
    } else if val == 0 {
        min
    } else {
        min + val as f64 * ((max - min) / u64::max_value() as f64)
    }
}

fn avg_rating_to_u64(avg_rating: AvgRatingValue) -> u64 {
    f64_to_u64(
        avg_rating.into(),
        AvgRatingValue::min().into(),
        AvgRatingValue::max().into(),
    )
}

fn u64_to_avg_rating(val: u64) -> AvgRatingValue {
    u64_to_f64(
        val,
        AvgRatingValue::min().into(),
        AvgRatingValue::max().into(),
    )
    .into()
}

impl TantivyEntryIndex {
    pub fn create_in_ram() -> Fallible<Self> {
        let no_path: Option<&Path> = None;
        Self::create(no_path)
    }

    pub fn create<P: AsRef<Path>>(path: Option<P>) -> Fallible<Self> {
        let (fields, schema) = IndexedEntryFields::build_schema();

        // TODO: Open index from existing directory
        let index = if let Some(path) = path {
            info!(
                "Creating full-text search index in directory: {}",
                path.as_ref().to_string_lossy()
            );
            Index::create_in_dir(path, schema)?
        } else {
            warn!("Creating full-text search index in RAM");
            Index::create_in_ram(schema)
        };

        register_tokenizers(&index);

        // Prefer to manually reload the index reader during `flush()`
        // to ensure that all committed changes become visible immediately.
        // Otherwise ReloadPolicy::OnCommit will delay the changes and
        // many tests would fail without modification.
        let index_reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()?;
        let index_writer = index.writer(OVERALL_INDEX_HEAP_SIZE_IN_BYTES)?;
        let text_query_parser = QueryParser::for_index(
            &index,
            vec![
                fields.title,
                fields.description,
                fields.address_street,
                fields.address_city,
                fields.address_zip,
                fields.address_country,
            ],
        );
        Ok(Self {
            fields,
            index,
            index_reader,
            index_writer,
            text_query_parser,
        })
    }

    fn build_query(&self, query: &EntryIndexQuery) -> BooleanQuery {
        let mut sub_queries: Vec<(Occur, Box<Query>)> = Vec::with_capacity(1 + 2 + 1 + 1 + 1);

        if !query.ids.is_empty() {
            let ids_query: Box<Query> = if query.ids.len() > 1 {
                debug!("Query multiple ids: {:?}", query.ids);
                let mut id_queries: Vec<(Occur, Box<Query>)> = Vec::with_capacity(query.ids.len());
                for id in &query.ids {
                    debug_assert!(!id.trim().is_empty());
                    let id_term = Term::from_field_text(self.fields.id, id);
                    let id_query = TermQuery::new(id_term, IndexRecordOption::Basic);
                    id_queries.push((Occur::Should, Box::new(id_query)));
                }
                Box::new(BooleanQuery::from(id_queries))
            } else {
                let id = &query.ids[0];
                debug!("Query single id: {:?}", id);
                debug_assert!(!id.trim().is_empty());
                let id_term = Term::from_field_text(self.fields.id, &id);
                Box::new(TermQuery::new(id_term, IndexRecordOption::Basic))
            };
            sub_queries.push((Occur::Must, ids_query));
        }

        // Bbox (include)
        if let Some(ref bbox) = query.include_bbox {
            debug!("Query bbox (include): {}", bbox);
            debug_assert!(bbox.is_valid());
            debug_assert!(!bbox.is_empty());
            let lat_query = RangeQuery::new_i64_bounds(
                self.fields.lat,
                Bound::Included(i64::from(bbox.south_west().lat().to_raw())),
                Bound::Included(i64::from(bbox.north_east().lat().to_raw())),
            );
            // Latitude query: Always inclusive
            sub_queries.push((Occur::Must, Box::new(lat_query)));
            // Longitude query: Either inclusive or exclusive (wrap around)
            if bbox.south_west().lng() <= bbox.north_east().lng() {
                // regular (inclusive)
                let lng_query = RangeQuery::new_i64_bounds(
                    self.fields.lng,
                    Bound::Included(i64::from(bbox.south_west().lng().to_raw())),
                    Bound::Included(i64::from(bbox.north_east().lng().to_raw())),
                );
                sub_queries.push((Occur::Must, Box::new(lng_query)));
            } else {
                // inverse (exclusive)
                let lng_query = RangeQuery::new_i64_bounds(
                    self.fields.lng,
                    Bound::Excluded(i64::from(bbox.north_east().lng().to_raw())),
                    Bound::Excluded(i64::from(bbox.south_west().lng().to_raw())),
                );
                sub_queries.push((Occur::MustNot, Box::new(lng_query)));
            }
        }

        // Inverse Bbox (exclude)
        if let Some(ref bbox) = query.exclude_bbox {
            debug!("Query bbox (exclude): {}", bbox);
            debug_assert!(bbox.is_valid());
            debug_assert!(!bbox.is_empty());
            let lat_query = RangeQuery::new_i64_bounds(
                self.fields.lat,
                Bound::Included(i64::from(bbox.south_west().lat().to_raw())),
                Bound::Included(i64::from(bbox.north_east().lat().to_raw())),
            );
            // Latitude query: Always exclusive
            sub_queries.push((Occur::MustNot, Box::new(lat_query)));
            // Longitude query: Either exclusive or inclusive (wrap around)
            if bbox.south_west().lng() <= bbox.north_east().lng() {
                // regular (exclusive)
                let lng_query = RangeQuery::new_i64_bounds(
                    self.fields.lng,
                    Bound::Included(i64::from(bbox.south_west().lng().to_raw())),
                    Bound::Included(i64::from(bbox.north_east().lng().to_raw())),
                );
                sub_queries.push((Occur::MustNot, Box::new(lng_query)));
            } else {
                // inverse (inclusive)
                let lng_query = RangeQuery::new_i64_bounds(
                    self.fields.lng,
                    Bound::Excluded(i64::from(bbox.north_east().lng().to_raw())),
                    Bound::Excluded(i64::from(bbox.south_west().lng().to_raw())),
                );
                sub_queries.push((Occur::Must, Box::new(lng_query)));
            }
        }

        // Categories
        if !query.categories.is_empty() {
            let categories_query: Box<Query> = if query.categories.len() > 1 {
                debug!("Query multiple categories: {:?}", query.categories);
                let mut category_queries: Vec<(Occur, Box<Query>)> =
                    Vec::with_capacity(query.categories.len());
                for category in &query.categories {
                    debug_assert!(!category.trim().is_empty());
                    let category_term =
                        Term::from_field_text(self.fields.category, &category.to_lowercase());
                    let category_query = TermQuery::new(category_term, IndexRecordOption::Basic);
                    category_queries.push((Occur::Should, Box::new(category_query)));
                }
                Box::new(BooleanQuery::from(category_queries))
            } else {
                let category = &query.categories[0];
                debug!("Query single category: {}", category);
                debug_assert!(!category.trim().is_empty());
                let category_term =
                    Term::from_field_text(self.fields.category, &category.to_lowercase());
                Box::new(TermQuery::new(category_term, IndexRecordOption::Basic))
            };
            sub_queries.push((Occur::Must, categories_query));
        }

        // Hash tags (mandatory)
        for tag in &query.hash_tags {
            debug!("Query hash tag (mandatory): {}", tag);
            debug_assert!(!tag.trim().is_empty());
            let tag_term = Term::from_field_text(self.fields.tag, &tag.to_lowercase());
            let tag_query = TermQuery::new(tag_term, IndexRecordOption::Basic);
            sub_queries.push((Occur::Must, Box::new(tag_query)));
        }

        let mut text_and_tags_queries: Vec<(Occur, Box<Query>)> =
            Vec::with_capacity(1 + query.text_tags.len());

        // Text
        if let Some(text) = &query.text {
            debug!("Query text: {}", text);
            debug_assert!(!text.trim().is_empty());
            match self.text_query_parser.parse_query(&text.to_lowercase()) {
                Ok(text_query) => {
                    if query.hash_tags.is_empty() && query.text_tags.is_empty() {
                        sub_queries.push((Occur::Must, Box::new(text_query)));
                    } else {
                        text_and_tags_queries.push((Occur::Should, Box::new(text_query)));
                    }
                }
                Err(err) => {
                    warn!("Failed to parse query text '{}': {:?}", text, err);
                }
            }
        }

        // Text tags (optional)
        for tag in &query.text_tags {
            debug!("Query text tag (optional): {}", tag);
            debug_assert!(!tag.trim().is_empty());
            let tag_term = Term::from_field_text(self.fields.tag, &tag.to_lowercase());
            let tag_query = TermQuery::new(tag_term, IndexRecordOption::Basic);
            text_and_tags_queries.push((Occur::Should, Box::new(tag_query)));
        }

        if !text_and_tags_queries.is_empty() {
            sub_queries.push((
                Occur::Must,
                Box::new(BooleanQuery::from(text_and_tags_queries)),
            ));
        }

        BooleanQuery::from(sub_queries)
    }
}

impl EntryIndexer for TantivyEntryIndex {
    fn add_or_update_entry(&mut self, entry: &Entry, ratings: &AvgRatings) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, &entry.id);
        self.index_writer.delete_term(id_term);
        let mut doc = Document::default();
        doc.add_text(self.fields.id, &entry.id);
        doc.add_i64(
            self.fields.lat,
            i64::from(entry.location.pos.lat().to_raw()),
        );
        doc.add_i64(
            self.fields.lng,
            i64::from(entry.location.pos.lng().to_raw()),
        );
        doc.add_text(self.fields.title, &entry.title);
        doc.add_text(self.fields.description, &entry.description);
        if let Some(street) = entry
            .location
            .address
            .as_ref()
            .and_then(|address| address.street.as_ref())
        {
            doc.add_text(self.fields.address_street, street);
        }
        if let Some(city) = entry
            .location
            .address
            .as_ref()
            .and_then(|address| address.city.as_ref())
        {
            doc.add_text(self.fields.address_city, city);
        }
        if let Some(zip) = entry
            .location
            .address
            .as_ref()
            .and_then(|address| address.zip.as_ref())
        {
            doc.add_text(self.fields.address_zip, zip);
        }
        if let Some(country) = entry
            .location
            .address
            .as_ref()
            .and_then(|address| address.country.as_ref())
        {
            doc.add_text(self.fields.address_country, country);
        }
        for category in &entry.categories {
            doc.add_text(self.fields.category, category);
        }
        for tag in &entry.tags {
            doc.add_text(self.fields.tag, tag);
        }
        doc.add_u64(self.fields.total_rating, avg_rating_to_u64(ratings.total()));
        doc.add_u64(
            self.fields.ratings_diversity,
            avg_rating_to_u64(ratings.diversity),
        );
        doc.add_u64(
            self.fields.ratings_fairness,
            avg_rating_to_u64(ratings.fairness),
        );
        doc.add_u64(
            self.fields.ratings_humanity,
            avg_rating_to_u64(ratings.humanity),
        );
        doc.add_u64(
            self.fields.ratings_renewable,
            avg_rating_to_u64(ratings.renewable),
        );
        doc.add_u64(
            self.fields.ratings_solidarity,
            avg_rating_to_u64(ratings.solidarity),
        );
        doc.add_u64(
            self.fields.ratings_transparency,
            avg_rating_to_u64(ratings.transparency),
        );
        self.index_writer.add_document(doc);
        Ok(())
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, id);
        self.index_writer.delete_term(id_term);
        Ok(())
    }

    fn flush(&mut self) -> Fallible<()> {
        self.index_writer.commit()?;
        // Manually reload the reader to ensure that all committed changes
        // become visible immediately.
        self.index_reader.reload()?;
        Ok(())
    }
}

impl EntryIndex for TantivyEntryIndex {
    fn query_entries(&self, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<IndexedEntry>> {
        if limit <= 0 {
            bail!("Invalid limit: {}", limit);
        }

        let searcher = self.index_reader.searcher();
        // TODO (2019-02-26): Ideally we would like to order the results by
        // (score * rating) instead of only (rating). Currently Tantivy doesn't
        // support this kind of collector.
        let collector = TopDocs::with_limit(limit).order_by_field(self.fields.total_rating);
        let top_docs_by_rating: Vec<(u64, DocAddress)> =
            searcher.search(&self.build_query(query), &collector)?;
        let mut entries = Vec::with_capacity(top_docs_by_rating.len());
        for (_total_rating, doc_addr) in top_docs_by_rating {
            match searcher.doc(doc_addr) {
                Ok(ref doc) => {
                    entries.push(self.fields.read_document(doc));
                }
                Err(err) => {
                    warn!("Failed to load document {:?}: {}", doc_addr, err);
                }
            }
        }
        Ok(entries)
    }
}

#[derive(Clone)]
pub struct SearchEngine(Arc<Mutex<Box<dyn EntryIndexer + Send>>>);

impl SearchEngine {
    pub fn init_in_ram() -> Fallible<SearchEngine> {
        let entry_index = TantivyEntryIndex::create_in_ram()?;
        Ok(SearchEngine(Arc::new(Mutex::new(Box::new(entry_index)))))
    }

    pub fn init_with_path<P: AsRef<Path>>(path: Option<P>) -> Fallible<SearchEngine> {
        let entry_index = TantivyEntryIndex::create(path)?;
        Ok(SearchEngine(Arc::new(Mutex::new(Box::new(entry_index)))))
    }
}

impl EntryIndex for SearchEngine {
    fn query_entries(&self, query: &EntryIndexQuery, limit: usize) -> Fallible<Vec<IndexedEntry>> {
        let entry_index = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        entry_index.query_entries(query, limit)
    }
}

impl EntryIndexer for SearchEngine {
    fn add_or_update_entry(&mut self, entry: &Entry, ratings: &AvgRatings) -> Fallible<()> {
        let mut inner = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner.add_or_update_entry(entry, ratings)
    }

    fn remove_entry_by_id(&mut self, id: &str) -> Fallible<()> {
        let mut inner = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner.remove_entry_by_id(id)
    }

    fn flush(&mut self) -> Fallible<()> {
        let mut inner = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        inner.flush()
    }
}
