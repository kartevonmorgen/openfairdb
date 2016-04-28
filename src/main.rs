// Copyright (c) 2015 - 2016 Markus Kohlhase <mail@markus-kohlhase.de>

#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate nickel;
extern crate rustc_serialize;
extern crate hyper;
extern crate unicase;
extern crate docopt;
#[macro_use]
extern crate rusted_cypher;
extern crate r2d2;
extern crate r2d2_cypher;
extern crate typemap;
extern crate uuid;
extern crate geojson;

mod json;
mod store;
mod error;
mod search;
mod filter;
mod sort;
mod geo;

use nickel::{Nickel, JsonBody, MediaType, QueryString, Request, Response, MiddlewareResult, HttpRouter};
use nickel::status::StatusCode;
use hyper::header::{AccessControlAllowOrigin, AccessControlAllowHeaders, AccessControlAllowMethods};
use hyper::method::Method;
use r2d2_cypher::CypherConnectionManager;
use docopt::Docopt;
use json::{Entry, Category, SearchResult};
use store::Store;
use error::{AppError, ParameterError, StoreError};
use rustc_serialize::json::encode;
use filter::{FilterByCategoryIds, FilterByBoundingBox};
use search::Search;
use sort::SortByDistanceTo;

static VERSION                  : &'static str = "0.0.16";
static POOL_SIZE                : u32 = 5;
static MAX_INVISIBLE_RESULTS    : usize = 5;

const USAGE: &'static str = "
ofdb - openFairDB.

Usage: ofdb [options]
       ofdb (--help | --version)

Options:
  --port PORT           Port [default: 6767].
  --db-url URL          URL to the DB [default: http://neo4j:neo4j@127.0.0.1:7474/db/data].
  --enable-cors         Allow requests from any origin.
  -V --version          Show version.
  -h --help             Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
  flag_port: Option<u16>,
  flag_db_url: Option<String>,
  flag_enable_cors: bool,
  flag_help: bool,
  flag_version: bool
}

#[derive(Debug, Clone)]
struct Data {
  db: r2d2::Pool<CypherConnectionManager>
}

fn enable_cors<'mw>(_req: &mut Request<Data>, mut res: Response<'mw,Data>) -> MiddlewareResult<'mw,Data> {
  res.set(AccessControlAllowOrigin::Any);
  res.set(AccessControlAllowHeaders(vec![
      "Origin".into(),
      "X-Requested-With".into(),
      "Content-Type".into(),
      "Accept".into(),
  ]));

  res.next_middleware()
}

fn main() {

  env_logger::init().unwrap();

  let args: Args = Docopt::new(USAGE)
     .and_then(|d| d.decode())
     .unwrap_or_else(|e| e.exit());

  if args.flag_version {
    print!("{}",VERSION);
    return;
  }

  let db_url  = args.flag_db_url.unwrap();
  let manager = CypherConnectionManager{url:db_url};
  let config  = r2d2::Config::builder().pool_size(POOL_SIZE).build();
  let pool    = r2d2::Pool::new(config, manager).unwrap();
  let data    = Data{db: pool};

  let mut server = Nickel::with_data(data);

  if args.flag_enable_cors {
    server.utilize(enable_cors);
    server.options("/entries/*", middleware!{|_, mut res|
      res.set(AccessControlAllowHeaders(vec!["Content-Type".into()]));
      res.set(AccessControlAllowMethods(vec![Method::Get, Method::Post, Method::Put]));
      StatusCode::Ok
    });
  }

  server.utilize(router! {

    get "/entries/:id" => |req, mut res|{
      match req.param("id")
        .ok_or(ParameterError::Id).map_err(AppError::Parameter)
        .and_then(|s|{
          let ids = s.split(",")
            .map(|x|x.to_string())
            .filter(|id| id != "")
            .collect::<Vec<String>>();
          let data: &Data = res.server_data();
          data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|{

              Entry::all(pool).map_err(AppError::Store).and_then(|entries|{

                match ids.len() {
                  0 => encode(&entries).map_err(AppError::Encode),
                  1 => {
                      entries.iter().find(|x| x.id.clone().is_some() && x.id.clone().unwrap() == ids[0])
                      .ok_or(StoreError::NotFound).map_err(AppError::Store)
                      .and_then(|e| encode(&e).map_err(AppError::Encode))
                  },
                  _ => {
                    let x = entries.iter()
                     .filter(|e| e.id.is_some())
                     .filter(|e| ids.iter().any(|id| e.id == Some(id.clone())))
                     .collect::<Vec<&Entry>>();
                     encode(&x).map_err(AppError::Encode)
                  }
                }

              })
            })
        })
        {
          Ok(x)  => {
            res.set(MediaType::Json);
            (StatusCode::Ok, format!("{}",x))
          },
          Err(ref err) =>
            (StatusCode::from(err), format!("Could not fetch entries: {}", err))
        }
      }

    post "/entries/:id" => |req, res|{
      match req.json_as::<Entry>().map_err(AppError::Io)
        .and_then(|json|{
          let data: &Data = res.server_data();
          data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|
              json.save(pool)
                .map_err(AppError::Store)
                .and_then(|e| e.id
                   .ok_or(StoreError::Save).map_err(AppError::Store))

          )
      })
      {
        Ok(id)  => (StatusCode::Ok, format!("{}",id)),
        Err(ref err) =>
          (StatusCode::from(err), format!("Could not create entry: {}", err))

      }
    }

    put "/entries/:id" => |req, mut res|{
      let entry = req.json_as::<Entry>();
      let data: &Data = res.server_data();
      match req.param("id")
        .ok_or(ParameterError::Id).map_err(AppError::Parameter)
        .and_then(|id| entry.map_err(AppError::Io)
          .and_then(|mut new_data|{
            data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|
              Entry::get(pool, id.to_string())
              .map_err(AppError::Store)
              .and_then(|_|{
                new_data.id = Some(id.to_string());
                new_data.save(pool)
                .map_err(AppError::Store)
                .and_then(|_| Ok(id))
              })
            )
          })
        )
      {
        Ok(id) => {
          (StatusCode::Ok, format!("{}",id))
        }
        Err(ref err) => {
          let msg = format!("Could not save entry: {}", err);
          warn!("{}", msg);
          (StatusCode::from(err), format!("Could not save entry: {}", err))
        }
      }
    }

    get "/categories/:id" => |req, mut res|{
      match req.param("id")
        .ok_or(ParameterError::Id).map_err(AppError::Parameter)
        .and_then(|s|{
          let ids = s.split(",")
            .map(|x|x.to_string())
            .filter(|id| id != "")
            .collect::<Vec<String>>();
          let data: &Data = res.server_data();
          data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|{
              Category::all(pool)
                .map_err(AppError::Store)
                .and_then(|categories|
                  match ids.len() {
                    0 => encode(&categories).map_err(AppError::Encode),
                    1 => {
                        categories.iter().find(|x| x.id.clone().is_some() && x.id.clone().unwrap() == ids[0])
                        .ok_or(StoreError::NotFound).map_err(AppError::Store)
                        .and_then(|e| encode(&e).map_err(AppError::Encode))
                    },
                    _ => {
                      let x = categories.iter()
                       .filter(|e| e.id.is_some())
                       .filter(|e| ids.iter().any(|id| e.id == Some(id.clone())))
                       .collect::<Vec<&Category>>();
                       encode(&x).map_err(AppError::Encode)
                    }
                  }
              )
          })
        })
        {
          Ok(x)  => {
            res.set(MediaType::Json);
            (StatusCode::Ok, format!("{}",x))
          },
          Err(ref err) =>
            (StatusCode::from(err), format!("Could not fetch categories: {}", err))
        }
      }

    get "/search" => |req, mut res| {
      let data: &Data = res.server_data();
      let query = req.query();
      match query
        .get("bbox")
        .ok_or(ParameterError::Bbox).map_err(AppError::Parameter)
        .and_then(|bbox_str| query
        .get("categories")
        .ok_or(ParameterError::Categories).map_err(AppError::Parameter)
        .and_then(|cat_str| data.db.clone().get()
            .map_err(StoreError::Pool)
            .map_err(AppError::Store)
            .and_then(|ref pool|
              Entry::all(pool)
              .map_err(AppError::Store)
              .and_then(|entries|{

                let cat_ids:Vec<String> = cat_str
                  .split(",")
                  .map(|x|x.to_string())
                  .filter(|id| id != "")
                  .collect();

                let bbox:Vec<f64> = bbox_str
                  .split(",")
                  .map(|x| x.parse::<f64>())
                  .filter_map(|x| x.ok())
                  .collect();

                if bbox.len() != 4 {
                  return Err(ParameterError::Bbox).map_err(AppError::Parameter)
                }

                let bbox_center = geo::center(
                    &geo::Coordinate{lat: bbox[0], lng: bbox[1]},
                    &geo::Coordinate{lat: bbox[2], lng: bbox[3]});

                let cat_filtered_entries = &entries
                  .filter_by_category_ids(&cat_ids);

                let mut pre_filtered_entries = match query.get("text"){
                  Some(txt) => cat_filtered_entries.filter_by_search_text(&txt.to_string()),
                  None      => cat_filtered_entries.iter().map(|x|x.clone()).collect()
                };

                pre_filtered_entries.sort_by_distance_to(&bbox_center);

                let visible_results = pre_filtered_entries
                  .filter_by_bounding_box(&bbox)
                  .map_to_ids();

                let invisible_results = pre_filtered_entries
                  .iter()
                  .filter(|e| !visible_results.iter().any(|v| e.id == Some(v.clone()) ))
                  .take(MAX_INVISIBLE_RESULTS)
                  .map(|x|x.clone())
                  .collect::<Vec<_>>()
                  .map_to_ids();

                let search_result = SearchResult{
                  visible   : visible_results,
                  invisible : invisible_results
                };
                encode(&search_result).map_err(AppError::Encode)
              })
            )
      ))
      {
        Ok(x)  => {
          res.set(MediaType::Json);
          (StatusCode::Ok, format!("{}",x))
        },
        Err(ref err) =>
          (StatusCode::from(err), format!("Could not search entries: {}", err))
      }

    }

    get "/server/version" => |_, res| { VERSION }

  });

  server.listen(("127.0.0.1",args.flag_port.unwrap_or(6767)));
}
