use std::{io, fmt};
use std::error::Error;
use rustc_serialize::json;
use nickel::status::StatusCode;
use rusted_cypher::error::{GraphError,Neo4jError};
use r2d2::{GetTimeout};

#[derive(Debug)]
pub enum StoreError {
  NotFound,
  InvalidVersion,
  InvalidId,
  Save,
  Graph(GraphError),
  Neo(Neo4jError),
  Pool(GetTimeout)
}

impl fmt::Display for StoreError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      StoreError::NotFound        => write!(f, "Could not find object"),
      StoreError::InvalidVersion  => write!(f, "The version property is invalid"),
      StoreError::InvalidId       => write!(f, "The ID is not valid"),
      StoreError::Save            => write!(f, "Could not save object"),
      StoreError::Graph(ref err)  => write!(f, "DB communication error: {}", err),
      StoreError::Neo(ref err)    => write!(f, "DB communication error: {}", err.message),
      StoreError::Pool(ref err)   => write!(f, "DB connection pool error: {}", err)
    }
  }
}

impl Error for StoreError {
  fn description(&self) -> &str {
    match *self {
      StoreError::NotFound        => "Could not find object",
      StoreError::InvalidVersion  => "The version property is invalid",
      StoreError::InvalidId       => "The ID is not valid",
      StoreError::Save            => "Could not save object",
      StoreError::Graph(ref err)  => err.description(),
      StoreError::Neo(ref err)    => &err.message,
      StoreError::Pool(ref err)   => &err.description(),
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      StoreError::NotFound        => None,
      StoreError::InvalidVersion  => None,
      StoreError::InvalidId       => None,
      StoreError::Save            => None,
      StoreError::Graph(ref err)  => Some(err),
      StoreError::Neo(_)          => None,
      StoreError::Pool(ref err)   => Some(err)
    }
  }
}

impl From<GraphError> for StoreError {
  fn from(err: GraphError) -> StoreError {
     StoreError::Graph(err)
  }
}

impl From<Neo4jError> for StoreError {
  fn from(err: Neo4jError) -> StoreError {
    StoreError::Neo(err)
  }
}

impl From<GetTimeout> for StoreError {
  fn from(err: GetTimeout) -> StoreError {
    StoreError::Pool(err)
  }
}

#[derive(Debug)]
pub enum ValidationError {
  License,
  Email,
  Url,
}

impl Error for ValidationError {

  fn description(&self) -> &str {
    match *self {
      ValidationError::License  => "Unsupported license",
      ValidationError::Email    => "Invalid email address",
      ValidationError::Url      => "Invalid URL"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      _ => None
    }
  }
}

impl fmt::Display for ValidationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ValidationError::License => write!(f, "Unsupported license"),
      ValidationError::Email   => write!(f, "Invalid email address"),
      ValidationError::Url     => write!(f, "Invalid URL")
    }
  }
}

#[derive(Debug)]
pub enum ParameterError {
  Id,
  Bbox,
  Categories
}

impl fmt::Display for ParameterError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      ParameterError::Id         => write!(f, "Requested ID is invalid"),
      ParameterError::Bbox       => write!(f, "Requested bounding box is invalid"),
      ParameterError::Categories => write!(f, "Requested categories are invalid")
    }
  }
}

impl Error for ParameterError {
  fn description(&self) -> &str {
    match *self {
      ParameterError::Id         => "Requested ID is invalid",
      ParameterError::Bbox       => "Requested bounding box is invalid",
      ParameterError::Categories => "Requested categories are invalid"
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      _ => None
    }
  }
}

#[derive(Debug)]
pub enum AppError {
  Encode(json::EncoderError),
  Parse(json::ParserError),
  //TODO: rename to Store
  Store(StoreError),
  Io(io::Error),
  Parameter(ParameterError),
  Validation(ValidationError)
}

impl fmt::Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      AppError::Encode(ref err)     => write!(f, "Encoding error: {}", err),
      AppError::Parse(ref err)      => write!(f, "Parsing error: {}", err),
      AppError::Store(ref err)      => write!(f, "DB error: {}", err),
      AppError::Io(ref err)         => write!(f, "IO error: {}", err),
      AppError::Parameter(ref err)  => write!(f, "Parameter error: {}", err),
      AppError::Validation(ref err) => write!(f, "Validation error: {}", err),
    }
  }
}

impl Error for AppError {
  fn description(&self) -> &str {
    match *self {
      AppError::Encode(ref err)     => err.description(),
      AppError::Parse(ref err)      => err.description(),
      AppError::Store(ref err)      => err.description(),
      AppError::Io(ref err)         => err.description(),
      AppError::Parameter(ref err)  => err.description(),
      AppError::Validation(ref err) => err.description(),
    }
  }

  fn cause(&self) -> Option<&Error> {
    match *self {
      AppError::Encode(ref err)     => Some(err),
      AppError::Parse(ref err)      => Some(err),
      AppError::Store(ref err)      => Some(err),
      AppError::Io(ref err)         => Some(err),
      AppError::Parameter(ref err)  => Some(err),
      AppError::Validation(ref err) => Some(err),
    }
  }
}

impl From<json::EncoderError> for AppError {
  fn from(err: json::EncoderError) -> AppError {
     AppError::Encode(err)
  }
}

impl From<json::ParserError> for AppError {
  fn from(err: json::ParserError) -> AppError {
    AppError::Parse(err)
  }
}

impl From<StoreError> for AppError {
  fn from(err: StoreError) -> AppError {
    AppError::Store(err)
  }
}

impl From<io::Error> for AppError {
  fn from(err: io::Error) -> AppError {
    AppError::Io(err)
  }
}

impl From<ParameterError> for AppError {
  fn from(err: ParameterError) -> AppError {
     AppError::Parameter(err)
  }
}

impl<'a> From<&'a AppError> for StatusCode {
  fn from(err: &AppError) -> StatusCode {
    match err {
      &AppError::Encode(_)       => StatusCode::InternalServerError,
      &AppError::Parse(_)        => StatusCode::BadRequest,
      &AppError::Io(_)           => StatusCode::BadRequest,
      &AppError::Parameter(_)    => StatusCode::BadRequest,
      &AppError::Validation(_)   => StatusCode::BadRequest,
      &AppError::Store(ref err)      => match err {
        &StoreError::NotFound        => StatusCode::NotFound,
        &StoreError::InvalidVersion  => StatusCode::BadRequest,
        &StoreError::InvalidId       => StatusCode::BadRequest,
        _                            => StatusCode::InternalServerError
      }
    }
  }
}
