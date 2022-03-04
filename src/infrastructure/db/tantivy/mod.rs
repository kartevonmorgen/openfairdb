use crate::core::{
    db::{
        EventAndPlaceIndexer, EventIndexer, IdIndex, IdIndexer, IndexQuery, IndexQueryMode,
        IndexedPlace, Indexer, PlaceIndex, PlaceIndexer,
    },
    entities::{
        Address, AvgRatingValue, AvgRatings, Category, Contact, Event, Id, Place, RatingContext,
        ReviewStatus, ReviewStatusPrimitive,
    },
    util::{
        geo::{LatCoord, LngCoord, MapPoint},
        time::Timestamp,
    },
};

use anyhow::{bail, Result as Fallible};
use num_traits::ToPrimitive;
use parking_lot::Mutex;
use std::{ops::Bound, path::Path, sync::Arc};
use strum::IntoEnumIterator as _;
use tantivy::{
    collector::TopDocs,
    fastfield::FastFieldReader as _,
    query::{BooleanQuery, Occur, Query, QueryParser, RangeQuery, TermQuery},
    schema::*,
    tokenizer::{LowerCaser, RawTokenizer, RemoveLongFilter, SimpleTokenizer, TextAnalyzer},
    DocAddress, DocId, Document, Index, IndexReader, IndexWriter, ReloadPolicy, Score,
    SegmentReader,
};

const OVERALL_INDEX_HEAP_SIZE_IN_BYTES: usize = 50_000_000;

const PLACE_KIND_FLAG: i64 = 1;
const EVENT_KIND_FLAG: i64 = 2;
const ALL_KINDS_MASK: i64 = PLACE_KIND_FLAG | EVENT_KIND_FLAG;

fn get_category_kind_flag(category: &Category) -> i64 {
    if category.id.as_str() == Category::ID_EVENT {
        EVENT_KIND_FLAG
    } else {
        PLACE_KIND_FLAG
    }
}

// Shared fields for both places and events
struct IndexedFields {
    kind: Field,
    id: Field,
    status: Field,
    lat: Field,
    lng: Field,
    ts_min: Field, // minimum time stamp with second precision, e.g. event start
    ts_max: Field, // maximum time stamp with second precision, e.g. event end
    title: Field,
    description: Field,
    address_street: Field,
    address_city: Field,
    address_zip: Field,
    address_country: Field,
    address_state: Field,
    contact_name: Field,
    tag: Field,
    ratings_diversity: Field,
    ratings_fairness: Field,
    ratings_humanity: Field,
    ratings_renewable: Field,
    ratings_solidarity: Field,
    ratings_transparency: Field,
    total_rating: Field,
}

impl IndexedFields {
    fn build_schema() -> (Self, Schema) {
        let id_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(ID_TOKENIZER)
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored();
        let tag_options = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer(TAG_TOKENIZER)
                    .set_index_option(IndexRecordOption::WithFreqs),
            )
            .set_stored();
        // Common options for indexing text fields
        let indexed_text_options = TextOptions::default().set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(TEXT_TOKENIZER)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        );
        // Text fields that are returned as part of the search result
        // additionally need to be stored explicitly
        let stored_text_options = indexed_text_options.clone().set_stored();
        let mut schema_builder = SchemaBuilder::default();
        let fields = Self {
            kind: schema_builder.add_i64_field("kind", INDEXED),
            id: schema_builder.add_text_field("id", id_options),
            status: schema_builder.add_i64_field("status", INDEXED | STORED),
            lat: schema_builder.add_f64_field("lat", INDEXED | STORED),
            lng: schema_builder.add_f64_field("lon", INDEXED | STORED),
            ts_min: schema_builder.add_i64_field("ts_min", INDEXED | STORED),
            ts_max: schema_builder.add_i64_field("ts_max", INDEXED | STORED),
            title: schema_builder.add_text_field("tit", stored_text_options.clone()),
            description: schema_builder.add_text_field("dsc", stored_text_options),
            contact_name: schema_builder.add_text_field("cnt_name", indexed_text_options.clone()),
            address_street: schema_builder
                .add_text_field("adr_street", indexed_text_options.clone()),
            address_city: schema_builder.add_text_field("adr_city", indexed_text_options.clone()),
            address_zip: schema_builder.add_text_field("adr_zip", indexed_text_options.clone()),
            address_country: schema_builder
                .add_text_field("adr_country", indexed_text_options.clone()),
            address_state: schema_builder.add_text_field("adr_state", indexed_text_options),
            tag: schema_builder.add_text_field("tag", tag_options),
            ratings_diversity: schema_builder.add_f64_field("rat_diversity", STORED),
            ratings_fairness: schema_builder.add_f64_field("rat_fairness", STORED),
            ratings_humanity: schema_builder.add_f64_field("rat_humanity", STORED),
            ratings_renewable: schema_builder.add_f64_field("rat_renewable", STORED),
            ratings_solidarity: schema_builder.add_f64_field("rat_solidarity", STORED),
            ratings_transparency: schema_builder.add_f64_field("rat_transparency", STORED),
            total_rating: schema_builder.add_u64_field("rat_total", STORED | FAST),
        };
        (fields, schema_builder.build())
    }

    fn read_indexed_place(&self, doc: &Document) -> IndexedPlace {
        let mut lat: Option<LatCoord> = Default::default();
        let mut lng: Option<LngCoord> = Default::default();
        let mut place = IndexedPlace::default();
        place.tags.reserve(32);
        for field_value in doc.field_values() {
            match field_value {
                fv if fv.field() == self.status => {
                    place.status = fv
                        .value()
                        .i64_value()
                        .and_then(|v| ReviewStatus::try_from(v as ReviewStatusPrimitive));
                }
                fv if fv.field() == self.lat => {
                    debug_assert!(lat.is_none());
                    lat = fv.value().f64_value().map(LatCoord::from_deg);
                }
                fv if fv.field() == self.lng => {
                    debug_assert!(lng.is_none());
                    lng = fv.value().f64_value().map(LngCoord::from_deg);
                }
                fv if fv.field() == self.id => {
                    debug_assert!(place.id.is_empty());
                    if let Some(id) = fv.value().text() {
                        place.id = id.into();
                    } else {
                        error!("Invalid id value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.title => {
                    debug_assert!(place.title.is_empty());
                    if let Some(title) = fv.value().text() {
                        place.title = title.into();
                    } else {
                        error!("Invalid title value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.description => {
                    debug_assert!(place.description.is_empty());
                    if let Some(description) = fv.value().text() {
                        place.description = description.into();
                    } else {
                        error!("Invalid description value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.tag => {
                    if let Some(tag) = fv.value().text() {
                        place.tags.push(tag.into());
                    } else {
                        error!("Invalid tag value: {:?}", fv.value());
                    }
                }
                fv if fv.field() == self.ratings_diversity => {
                    debug_assert!(place.ratings.diversity == Default::default());
                    place.ratings.diversity =
                        fv.value().f64_value().map(Into::into).unwrap_or_default();
                }
                fv if fv.field() == self.ratings_fairness => {
                    debug_assert!(place.ratings.fairness == Default::default());
                    place.ratings.fairness =
                        fv.value().f64_value().map(Into::into).unwrap_or_default();
                }
                fv if fv.field() == self.ratings_humanity => {
                    debug_assert!(place.ratings.humanity == Default::default());
                    place.ratings.humanity =
                        fv.value().f64_value().map(Into::into).unwrap_or_default();
                }
                fv if fv.field() == self.ratings_renewable => {
                    debug_assert!(place.ratings.renewable == Default::default());
                    place.ratings.renewable =
                        fv.value().f64_value().map(Into::into).unwrap_or_default();
                }
                fv if fv.field() == self.ratings_solidarity => {
                    debug_assert!(place.ratings.solidarity == Default::default());
                    place.ratings.solidarity =
                        fv.value().f64_value().map(Into::into).unwrap_or_default();
                }
                fv if fv.field() == self.ratings_transparency => {
                    debug_assert!(place.ratings.transparency == Default::default());
                    place.ratings.transparency =
                        fv.value().f64_value().map(Into::into).unwrap_or_default();
                }
                fv if fv.field() == self.total_rating => (),
                // Address fields are currently not stored
                //fv if fv.field() == self.address_street => (),
                //fv if fv.field() == self.address_city => (),
                //fv if fv.field() == self.address_zip => (),
                //fv if fv.field() == self.address_country => (),
                //fv if fv.field() == self.address_state => (),
                fv => {
                    error!("Unexpected field value: {:?}", fv);
                }
            }
        }
        if let (Some(lat), Some(lng)) = (lat, lng) {
            place.pos = MapPoint::new(lat, lng);
        } else {
            error!("Invalid position: lat = {:?}, lng = {:?}", lat, lng);
        }
        place
    }
}

pub(crate) struct TantivyIndex {
    fields: IndexedFields,
    index_reader: IndexReader,
    index_writer: IndexWriter,
    text_query_parser: QueryParser,
}

const ID_TOKENIZER: &str = "raw";
const TAG_TOKENIZER: &str = "tag";
const TEXT_TOKENIZER: &str = "default";

const MAX_TOKEN_LEN: usize = 40;

fn register_tokenizers(index: &Index) {
    // Predefined tokenizers
    debug_assert!(index.tokenizers().get(ID_TOKENIZER).is_some());
    debug_assert!(index.tokenizers().get(TEXT_TOKENIZER).is_some());
    // Custom tokenizer(s)
    debug_assert!(index.tokenizers().get(TAG_TOKENIZER).is_none());
    let tag_tokenizer = TextAnalyzer::from(RawTokenizer)
        .filter(LowerCaser)
        .filter(RemoveLongFilter::limit(MAX_TOKEN_LEN));
    index.tokenizers().register(TAG_TOKENIZER, tag_tokenizer);
    let text_tokenizer = TextAnalyzer::from(SimpleTokenizer)
        .filter(LowerCaser)
        .filter(RemoveLongFilter::limit(MAX_TOKEN_LEN));
    index.tokenizers().register(TEXT_TOKENIZER, text_tokenizer);
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

#[derive(Copy, Clone, Debug)]
enum TopDocsMode {
    Score,
    Rating,
    ScoreBoostedByRating,
}

impl TantivyIndex {
    #[allow(dead_code)]
    pub fn create_in_ram() -> Fallible<Self> {
        let no_path: Option<&Path> = None;
        Self::create(no_path)
    }

    pub fn create<P: AsRef<Path>>(path: Option<P>) -> Fallible<Self> {
        let (fields, schema) = IndexedFields::build_schema();

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
                fields.address_state,
                fields.contact_name,
            ],
        );
        Ok(Self {
            fields,
            index_reader,
            index_writer,
            text_query_parser,
        })
    }

    fn build_query(
        &self,
        query_mode: IndexQueryMode,
        query: &IndexQuery,
    ) -> (BooleanQuery, TopDocsMode) {
        let mut sub_queries: Vec<(Occur, Box<dyn Query>)> = Vec::with_capacity(1 + 2 + 1 + 1 + 1);

        if !query.ids.is_empty() {
            let ids_query: Box<dyn Query> = if query.ids.len() > 1 {
                debug!("Query multiple ids: {:?}", query.ids);
                let mut id_queries: Vec<(Occur, Box<dyn Query>)> =
                    Vec::with_capacity(query.ids.len());
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
                let id_term = Term::from_field_text(self.fields.id, id);
                Box::new(TermQuery::new(id_term, IndexRecordOption::Basic))
            };
            sub_queries.push((Occur::Must, ids_query));
        }

        // Status
        if let Some(ref status) = query.status {
            // NOTE(2019-12-17, Tantivy v0.11.1): A boolean query that contains
            // only MustNot sub-queries does not work as expected, especially
            // for events that do not have a status (yet). Therefore we need to
            // also query for all other variants that are not explicitly excluded
            // with a Should occurrence.
            //
            // Desired implementation (...that didn't work as intended):
            // let exclude_status: Vec<_> = if status.is_empty() {
            //     // Exclude all invisible/inexistent entries
            //     ReviewStatus::iter().filter(|s| !s.exists()).collect()
            // } else {
            //     // Exclude all entries with a different status than requested
            //     ReviewStatus::iter()
            //         .filter(|s1| status.iter().any(|s2| s1 != s2))
            //         .collect()
            // };
            // debug_assert!(!exclude_status.is_empty());
            // let exclude_status_queries: Vec<_> = exclude_status
            //     .into_iter()
            //     .map(|status| {
            //         let status = status.to_i64().unwrap();
            //         let boxed_query: Box<dyn Query> = Box::new(RangeQuery::new_i64_bounds(
            //             self.fields.status,
            //             Bound::Included(status),
            //             Bound::Included(status),
            //         ));
            //         (Occur::MustNot, boxed_query)
            //     })
            //     .collect();
            // sub_queries.push((Occur::Must, Box::new(BooleanQuery::from(exclude_status_queries))));
            let status: Vec<_> = if status.is_empty() {
                ReviewStatus::iter()
                    .map(|s| {
                        if s.exists() {
                            (Occur::Should, s)
                        } else {
                            (Occur::MustNot, s)
                        }
                    })
                    .collect()
            } else {
                ReviewStatus::iter()
                    .map(|s1| {
                        if status.iter().any(|s2| s1 == *s2) {
                            (Occur::Should, s1)
                        } else {
                            (Occur::MustNot, s1)
                        }
                    })
                    .collect()
            };
            let status_queries: Vec<_> = status
                .into_iter()
                .map(|(occur, status)| {
                    let status = status.to_i64().unwrap();
                    let boxed_query: Box<dyn Query> = Box::new(RangeQuery::new_i64_bounds(
                        self.fields.status,
                        Bound::Included(status),
                        Bound::Included(status),
                    ));
                    (occur, boxed_query)
                })
                .collect();
            sub_queries.push((Occur::Must, Box::new(BooleanQuery::from(status_queries))));
        }

        // Bbox (include)
        if let Some(ref bbox) = query.include_bbox {
            debug!("Query bbox (include): {}", bbox);
            debug_assert!(bbox.is_valid());
            debug_assert!(!bbox.is_empty());
            let lat_query = RangeQuery::new_f64_bounds(
                self.fields.lat,
                Bound::Included(bbox.southwest().lat().to_deg()),
                Bound::Included(bbox.northeast().lat().to_deg()),
            );
            // Latitude query: Always inclusive
            sub_queries.push((Occur::Must, Box::new(lat_query)));
            // Longitude query: Either inclusive or exclusive (wrap around)
            if bbox.southwest().lng() <= bbox.northeast().lng() {
                // regular (inclusive)
                let lng_query = RangeQuery::new_f64_bounds(
                    self.fields.lng,
                    Bound::Included(bbox.southwest().lng().to_deg()),
                    Bound::Included(bbox.northeast().lng().to_deg()),
                );
                sub_queries.push((Occur::Must, Box::new(lng_query)));
            } else {
                // inverse (exclusive)
                let lng_query = RangeQuery::new_f64_bounds(
                    self.fields.lng,
                    Bound::Excluded(bbox.northeast().lng().to_deg()),
                    Bound::Excluded(bbox.southwest().lng().to_deg()),
                );
                sub_queries.push((Occur::MustNot, Box::new(lng_query)));
            }
        }

        // Inverse Bbox (exclude)
        if let Some(ref bbox) = query.exclude_bbox {
            debug!("Query bbox (exclude): {}", bbox);
            debug_assert!(bbox.is_valid());
            debug_assert!(!bbox.is_empty());
            let lat_query = RangeQuery::new_f64_bounds(
                self.fields.lat,
                Bound::Included(bbox.southwest().lat().to_deg()),
                Bound::Included(bbox.northeast().lat().to_deg()),
            );
            // Latitude query: Always exclusive
            sub_queries.push((Occur::MustNot, Box::new(lat_query)));
            // Longitude query: Either exclusive or inclusive (wrap around)
            if bbox.southwest().lng() <= bbox.northeast().lng() {
                // regular (exclusive)
                let lng_query = RangeQuery::new_f64_bounds(
                    self.fields.lng,
                    Bound::Included(bbox.southwest().lng().to_deg()),
                    Bound::Included(bbox.northeast().lng().to_deg()),
                );
                sub_queries.push((Occur::MustNot, Box::new(lng_query)));
            } else {
                // inverse (inclusive)
                let lng_query = RangeQuery::new_f64_bounds(
                    self.fields.lng,
                    Bound::Excluded(bbox.northeast().lng().to_deg()),
                    Bound::Excluded(bbox.southwest().lng().to_deg()),
                );
                sub_queries.push((Occur::Must, Box::new(lng_query)));
            }
        }

        let merged_tags = Category::merge_ids_into_tags(
            &query
                .categories
                .iter()
                .map(|c| Id::from(*c))
                .collect::<Vec<_>>(),
            query.hash_tags.clone(),
        );
        let (tags, categories) = Category::split_from_tags(merged_tags);

        // Categories (= mapped to predefined tags + separate sub-query + kind)
        let mut kinds_mask = 0i64;
        // Special handling of place categories for backwards compatibility
        let place_categories =
            categories
                .into_iter()
                .fold(Vec::with_capacity(2), |mut place_categories, category| {
                    let kind_flag = get_category_kind_flag(&category);
                    kinds_mask |= kind_flag;
                    if kind_flag == PLACE_KIND_FLAG {
                        place_categories.push(category);
                    }
                    place_categories
                });
        if !place_categories.is_empty() {
            let categories_query: Box<dyn Query> = if place_categories.len() > 1 {
                debug!("Query multiple place categories: {:?}", place_categories);
                let mut category_queries: Vec<(Occur, Box<dyn Query>)> =
                    Vec::with_capacity(place_categories.len());
                for category in &place_categories {
                    let tag_term = Term::from_field_text(self.fields.tag, &category.tag);
                    let tag_query = TermQuery::new(tag_term, IndexRecordOption::Basic);
                    category_queries.push((Occur::Should, Box::new(tag_query)));
                }
                Box::new(BooleanQuery::from(category_queries))
            } else {
                let category = &place_categories[0];
                debug!("Query single place category: {:?}", category);
                let tag_term = Term::from_field_text(self.fields.tag, &category.tag);
                Box::new(TermQuery::new(tag_term, IndexRecordOption::Basic))
            };
            sub_queries.push((Occur::Must, categories_query));
        }
        // Select the requested partitions by kind
        if kinds_mask != 0 {
            if kinds_mask.count_ones() == 1 {
                // must match one kind
                let kind_term = Term::from_field_i64(self.fields.kind, kinds_mask);
                let query = TermQuery::new(kind_term, IndexRecordOption::Basic);
                sub_queries.push((Occur::Must, Box::new(query)));
            } else {
                // most not match all other kinds
                let inverse_mask = ALL_KINDS_MASK ^ kinds_mask;
                let num_terms = inverse_mask.count_ones() as usize;
                let mut kind_terms = Vec::with_capacity(num_terms);
                let mut single_kind_mask = 1i64;
                while kind_terms.len() < num_terms {
                    if inverse_mask & single_kind_mask == single_kind_mask {
                        kind_terms.push(Term::from_field_i64(self.fields.kind, single_kind_mask));
                    }
                    single_kind_mask <<= 1;
                }
                let query = BooleanQuery::new_multiterms_query(kind_terms);
                sub_queries.push((Occur::MustNot, Box::new(query)));
            };
        }

        // Hash tags (mandatory)
        for tag in &tags {
            debug!("Query hash tag (mandatory): {}", tag);
            debug_assert!(!tag.trim().is_empty());
            let tag_term = Term::from_field_text(self.fields.tag, &tag.to_lowercase());
            let tag_query = TermQuery::new(tag_term, IndexRecordOption::Basic);
            sub_queries.push((Occur::Must, Box::new(tag_query)));
        }

        let mut text_and_tags_queries: Vec<(Occur, Box<dyn Query>)> =
            Vec::with_capacity(1 + query.text_tags.len());

        // Text
        if let Some(text) = &query.text {
            debug!("Query text: {}", text);
            debug_assert!(!text.trim().is_empty());
            let text = text.to_lowercase();
            match self.text_query_parser.parse_query(&text) {
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

        // ts_min
        let ts_min_lb = query
            .ts_min_lb
            .map(|x| Bound::Included(x.into_inner()))
            .unwrap_or(Bound::Unbounded);
        let ts_min_ub = query
            .ts_min_ub
            .map(|x| Bound::Included(x.into_inner()))
            .unwrap_or(Bound::Unbounded);
        if (ts_min_lb, ts_min_ub) != (Bound::Unbounded, Bound::Unbounded) {
            let ts_min_query = RangeQuery::new_i64_bounds(self.fields.ts_min, ts_min_lb, ts_min_ub);
            sub_queries.push((Occur::Must, Box::new(ts_min_query)));
        }

        // ts_max
        let ts_max_lb = query
            .ts_max_lb
            .map(|x| Bound::Included(x.into_inner()))
            .unwrap_or(Bound::Unbounded);
        let ts_max_ub = query
            .ts_max_ub
            .map(|x| Bound::Included(x.into_inner()))
            .unwrap_or(Bound::Unbounded);
        if (ts_max_lb, ts_max_ub) != (Bound::Unbounded, Bound::Unbounded) {
            let ts_max_query = RangeQuery::new_i64_bounds(self.fields.ts_max, ts_max_lb, ts_max_ub);
            sub_queries.push((Occur::Must, Box::new(ts_max_query)));
        }

        // Boosting the score by the rating does only make sense if the
        // query actually contains search terms or tags. Otherwise the
        // results are sorted only by their rating, e.g. if the query
        // contains just the bounding box or ids.
        if text_and_tags_queries.is_empty() {
            let mode = match query_mode {
                IndexQueryMode::WithRating => TopDocsMode::Rating,
                IndexQueryMode::WithoutRating => TopDocsMode::Score,
            };
            (sub_queries.into(), mode)
        } else {
            sub_queries.push((
                Occur::Must,
                Box::new(BooleanQuery::from(text_and_tags_queries)),
            ));
            let mode = match query_mode {
                IndexQueryMode::WithRating => TopDocsMode::ScoreBoostedByRating,
                IndexQueryMode::WithoutRating => TopDocsMode::Score,
            };
            (sub_queries.into(), mode)
        }
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    fn query_documents<D>(
        &self,
        query_mode: IndexQueryMode,
        query: &IndexQuery,
        limit: usize,
        mut doc_collector: D,
    ) -> Fallible<D>
    where
        D: DocumentCollector,
    {
        if limit <= 0 {
            bail!("Invalid limit: {}", limit);
        }

        let (search_query, top_docs_mode) = self.build_query(query_mode, query);
        let searcher = self.index_reader.searcher();
        // TODO: Try to combine redundant code from different search strategies
        match top_docs_mode {
            TopDocsMode::Score => {
                let collector = TopDocs::with_limit(limit);
                let top_docs = searcher.search(&search_query, &collector)?;
                for (_, doc_addr) in top_docs {
                    match searcher.doc(doc_addr) {
                        Ok(doc) => {
                            doc_collector.collect_document(doc_addr, doc);
                        }
                        Err(err) => {
                            warn!("Failed to load document {:?}: {}", doc_addr, err);
                        }
                    }
                }
                Ok(doc_collector)
            }
            TopDocsMode::Rating => {
                let collector =
                    TopDocs::with_limit(limit).order_by_u64_field(self.fields.total_rating);
                searcher.search(&search_query, &collector)?;
                let top_docs = searcher.search(&search_query, &collector)?;
                for (_, doc_addr) in top_docs {
                    match searcher.doc(doc_addr) {
                        Ok(doc) => {
                            doc_collector.collect_document(doc_addr, doc);
                        }
                        Err(err) => {
                            warn!("Failed to load document {:?}: {}", doc_addr, err);
                        }
                    }
                }
                Ok(doc_collector)
            }
            TopDocsMode::ScoreBoostedByRating => {
                let collector = {
                    let total_rating_field = self.fields.total_rating;
                    TopDocs::with_limit(limit).tweak_score(move |segment_reader: &SegmentReader| {
                        let total_rating_reader = segment_reader
                            .fast_fields()
                            .u64(total_rating_field)
                            .unwrap();

                        move |doc: DocId, original_score: Score| {
                            let total_rating =
                                f64::from(u64_to_avg_rating(total_rating_reader.get(doc)));
                            let boost_factor =
                                if total_rating < f64::from(AvgRatingValue::default()) {
                                    // Negative ratings result in a boost factor < 1
                                    (total_rating - f64::from(AvgRatingValue::min()))
                                        / (f64::from(AvgRatingValue::default())
                                            - f64::from(AvgRatingValue::min()))
                                } else {
                                    // Default rating results in a boost factor of 1
                                    // Positive ratings result in a boost factor > 1
                                    // The total rating is scaled by the number of different rating context
                                    // variants to achieve better results by emphasizing the rating factor.
                                    1.0 + f64::from(RatingContext::total_count())
                                        * (total_rating - f64::from(AvgRatingValue::default()))
                                };
                            // Transform the original score by log2() to narrow the range. Otherwise
                            // the rating boost factor is not powerful enough to promote highly
                            // rated entries over entries that received a much higher score.
                            debug_assert!(original_score >= 0.0);
                            let unboosted_score = (1.0 + original_score).log2();
                            unboosted_score * (boost_factor as f32)
                        }
                    })
                };
                let top_docs = searcher.search(&search_query, &collector)?;
                for (_, doc_addr) in top_docs {
                    match searcher.doc(doc_addr) {
                        Ok(doc) => {
                            doc_collector.collect_document(doc_addr, doc);
                        }
                        Err(err) => {
                            warn!("Failed to load document {:?}: {}", doc_addr, err);
                        }
                    }
                }
                Ok(doc_collector)
            }
        }
    }
}

trait DocumentCollector {
    fn collect_document(&mut self, doc_addr: DocAddress, doc: Document);
}

struct IdCollector {
    id_field: Field,
    collected_ids: Vec<Id>,
}

impl IdCollector {
    fn with_capacity(id_field: Field, capacity: usize) -> Self {
        Self {
            id_field,
            collected_ids: Vec::with_capacity(capacity),
        }
    }
}

impl From<IdCollector> for Vec<Id> {
    fn from(from: IdCollector) -> Self {
        from.collected_ids
    }
}

impl DocumentCollector for IdCollector {
    fn collect_document(&mut self, doc_addr: DocAddress, doc: Document) {
        if let Some(id) = doc.get_first(self.id_field).and_then(Value::text) {
            self.collected_ids.push(Id::from(id));
        } else {
            error!(
                "Document ({:?}) has no id field ({:?}) value",
                doc_addr, self.id_field
            );
        }
    }
}

struct IndexedPlaceCollector<'a> {
    fields: &'a IndexedFields,
    collected_places: Vec<IndexedPlace>,
}

impl<'a> IndexedPlaceCollector<'a> {
    fn with_capacity(fields: &'a IndexedFields, capacity: usize) -> Self {
        Self {
            fields,
            collected_places: Vec::with_capacity(capacity),
        }
    }
}

impl<'a> From<IndexedPlaceCollector<'a>> for Vec<IndexedPlace> {
    fn from(from: IndexedPlaceCollector<'a>) -> Self {
        from.collected_places
    }
}

impl<'a> DocumentCollector for IndexedPlaceCollector<'a> {
    fn collect_document(&mut self, _doc_addr: DocAddress, doc: Document) {
        self.collected_places
            .push(self.fields.read_indexed_place(&doc));
    }
}

impl IdIndex for TantivyIndex {
    fn query_ids(
        &self,
        query_mode: IndexQueryMode,
        query: &IndexQuery,
        limit: usize,
    ) -> Fallible<Vec<Id>> {
        let collector = IdCollector::with_capacity(self.fields.id, limit);
        self.query_documents(query_mode, query, limit, collector)
            .map(Into::into)
    }
}

impl Indexer for TantivyIndex {
    fn flush_index(&mut self) -> Fallible<()> {
        self.index_writer.commit()?;
        // Manually reload the reader to ensure that all committed changes
        // become visible immediately.
        self.index_reader.reload()?;
        Ok(())
    }
}

impl IdIndexer for TantivyIndex {
    fn remove_by_id(&self, id: &Id) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, id.as_str());
        self.index_writer.delete_term(id_term);
        Ok(())
    }
}

impl PlaceIndexer for TantivyIndex {
    fn add_or_update_place(
        &self,
        place: &Place,
        status: ReviewStatus,
        ratings: &AvgRatings,
    ) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, place.id.as_str());
        self.index_writer.delete_term(id_term);
        let mut doc = Document::default();
        doc.add_i64(self.fields.kind, PLACE_KIND_FLAG);
        if let Some(status) = status.to_i64() {
            doc.add_i64(self.fields.status, status);
        }
        doc.add_text(self.fields.id, &place.id);
        doc.add_f64(self.fields.lat, place.location.pos.lat().to_deg());
        doc.add_f64(self.fields.lng, place.location.pos.lng().to_deg());
        doc.add_text(self.fields.title, &place.title);
        doc.add_text(self.fields.description, &place.description);
        if let Some(ref address) = place.location.address {
            let Address {
                street,
                city,
                zip,
                country,
                state,
            } = address;
            if let Some(street) = street {
                doc.add_text(self.fields.address_street, street);
            }
            if let Some(city) = city {
                doc.add_text(self.fields.address_city, city);
            }
            if let Some(zip) = zip {
                doc.add_text(self.fields.address_zip, zip);
            }
            if let Some(country) = country {
                doc.add_text(self.fields.address_country, country);
            }
            if let Some(state) = state {
                doc.add_text(self.fields.address_country, state);
            }
        }
        if let Some(ref contact) = place.contact {
            let Contact { name, .. } = contact;
            if let Some(contact_name) = name {
                doc.add_text(self.fields.contact_name, contact_name);
            }
        }
        for tag in &place.tags {
            doc.add_text(self.fields.tag, tag);
        }
        doc.add_u64(self.fields.total_rating, avg_rating_to_u64(ratings.total()));
        doc.add_f64(self.fields.ratings_diversity, ratings.diversity.into());
        doc.add_f64(self.fields.ratings_fairness, ratings.fairness.into());
        doc.add_f64(self.fields.ratings_humanity, ratings.humanity.into());
        doc.add_f64(self.fields.ratings_renewable, ratings.renewable.into());
        doc.add_f64(self.fields.ratings_solidarity, ratings.solidarity.into());
        doc.add_f64(
            self.fields.ratings_transparency,
            ratings.transparency.into(),
        );
        self.index_writer.add_document(doc);
        Ok(())
    }
}

impl EventIndexer for TantivyIndex {
    fn add_or_update_event(&self, event: &Event) -> Fallible<()> {
        let id_term = Term::from_field_text(self.fields.id, event.id.as_str());
        self.index_writer.delete_term(id_term);
        let mut doc = Document::default();
        doc.add_i64(self.fields.kind, EVENT_KIND_FLAG);
        doc.add_text(self.fields.id, &event.id);
        if let Some(ref location) = event.location {
            doc.add_f64(self.fields.lat, location.pos.lat().to_deg());
            doc.add_f64(self.fields.lng, location.pos.lng().to_deg());
            if let Some(address) = &location.address {
                let Address {
                    street,
                    city,
                    zip,
                    country,
                    state,
                } = address;
                if let Some(street) = street {
                    doc.add_text(self.fields.address_street, street);
                }
                if let Some(city) = city {
                    doc.add_text(self.fields.address_city, city);
                }
                if let Some(zip) = zip {
                    doc.add_text(self.fields.address_zip, zip);
                }
                if let Some(country) = country {
                    doc.add_text(self.fields.address_country, country);
                }
                if let Some(state) = state {
                    doc.add_text(self.fields.address_country, state);
                }
            }
        }
        doc.add_i64(
            self.fields.ts_min,
            Timestamp::from(event.start).into_inner(),
        );
        if let Some(end) = event.end {
            debug_assert!(event.start <= end);
            doc.add_i64(self.fields.ts_max, Timestamp::from(end).into_inner());
        }
        doc.add_text(self.fields.title, &event.title);
        if let Some(ref description) = event.description {
            doc.add_text(self.fields.description, description);
        }
        if let Some(ref contact) = event.contact {
            let Contact { name, .. } = contact;
            if let Some(contact_name) = name {
                doc.add_text(self.fields.contact_name, contact_name);
            }
        }
        for tag in &event.tags {
            doc.add_text(self.fields.tag, tag);
        }
        self.index_writer.add_document(doc);
        Ok(())
    }
}

impl PlaceIndex for TantivyIndex {
    fn query_places(&self, query: &IndexQuery, limit: usize) -> Fallible<Vec<IndexedPlace>> {
        let collector = IndexedPlaceCollector::with_capacity(&self.fields, limit);
        self.query_documents(IndexQueryMode::WithRating, query, limit, collector)
            .map(Into::into)
    }
}

impl EventAndPlaceIndexer for TantivyIndex {}

#[derive(Clone)]
pub struct SearchEngine(Arc<Mutex<Box<dyn EventAndPlaceIndexer + Send>>>);

impl SearchEngine {
    #[allow(dead_code)]
    pub fn init_in_ram() -> Fallible<SearchEngine> {
        let index = TantivyIndex::create_in_ram()?;
        Ok(SearchEngine(Arc::new(Mutex::new(Box::new(index)))))
    }

    pub fn init_with_path<P: AsRef<Path>>(path: Option<P>) -> Fallible<SearchEngine> {
        let index = TantivyIndex::create(path)?;
        Ok(SearchEngine(Arc::new(Mutex::new(Box::new(index)))))
    }
}

impl Indexer for SearchEngine {
    fn flush_index(&mut self) -> Fallible<()> {
        let mut inner = self.0.lock();
        inner.flush_index()
    }
}

impl IdIndex for SearchEngine {
    fn query_ids(
        &self,
        mode: IndexQueryMode,
        query: &IndexQuery,
        limit: usize,
    ) -> Fallible<Vec<Id>> {
        let inner = self.0.lock();
        inner.query_ids(mode, query, limit)
    }
}

impl IdIndexer for SearchEngine {
    fn remove_by_id(&self, id: &Id) -> Fallible<()> {
        let inner = self.0.lock();
        inner.remove_by_id(id)
    }
}

impl PlaceIndex for SearchEngine {
    fn query_places(&self, query: &IndexQuery, limit: usize) -> Fallible<Vec<IndexedPlace>> {
        let inner = self.0.lock();
        inner.query_places(query, limit)
    }
}

impl PlaceIndexer for SearchEngine {
    fn add_or_update_place(
        &self,
        place: &Place,
        status: ReviewStatus,
        ratings: &AvgRatings,
    ) -> Fallible<()> {
        let inner = self.0.lock();
        inner.add_or_update_place(place, status, ratings)
    }
}

impl EventIndexer for SearchEngine {
    fn add_or_update_event(&self, event: &Event) -> Fallible<()> {
        let inner = self.0.lock();
        inner.add_or_update_event(event)
    }
}

impl EventAndPlaceIndexer for SearchEngine {}
