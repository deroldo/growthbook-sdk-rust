use std::env::VarError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;

use reqwest::Response;

use crate::model::Flag;

#[derive(Debug)]
pub enum GrowthbookErrorCode {
    GenericError,
    SerdeDeserialize,
    ParseError,
    MissingEnvironmentVariable,
    GrowthbookGateway,
    GrowthbookGatewayDeserialize,
    InvalidResponseValueType,
}

#[derive(Debug)]
pub struct GrowthbookError {
    pub code: GrowthbookErrorCode,
    pub message: String,
}

impl GrowthbookError {
    pub fn new(code: GrowthbookErrorCode, message: &str) -> Self {
        GrowthbookError {
            code,
            message: String::from(message),
        }
    }

    pub fn invalid_response_value_type(flag: Flag, expected_type: &str) -> Self {
        let value = match flag {
            Flag::BooleanFlag(it) => it.enabled.to_string(),
            Flag::StringFlag(it) => it.value,
            Flag::InvalidFlag() => String::from("'INVALID TYPE'"),
        };

        GrowthbookError {
            code: GrowthbookErrorCode::InvalidResponseValueType,
            message: format!("Invalid value={value} for expected type={expected_type}"),
        }
    }
}

impl Display for GrowthbookError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for GrowthbookError {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<Box<dyn Error>> for GrowthbookError {
    fn from(error: Box<dyn Error>) -> Self {
        Self {
            code: GrowthbookErrorCode::GenericError,
            message: error.to_string(),
        }
    }
}

impl From<reqwest_middleware::Error> for GrowthbookError {
    fn from(error: reqwest_middleware::Error) -> Self {
        Self {
            code: GrowthbookErrorCode::GrowthbookGateway,
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for GrowthbookError {
    fn from(error: reqwest::Error) -> Self {
        Self {
            code: GrowthbookErrorCode::GrowthbookGatewayDeserialize,
            message: error.to_string(),
        }
    }
}

impl From<VarError> for GrowthbookError {
    fn from(error: VarError) -> Self {
        Self {
            code: GrowthbookErrorCode::MissingEnvironmentVariable,
            message: error.to_string(),
        }
    }
}

impl From<ParseIntError> for GrowthbookError {
    fn from(error: ParseIntError) -> Self {
        Self {
            code: GrowthbookErrorCode::ParseError,
            message: error.to_string(),
        }
    }
}

impl From<Response> for GrowthbookError {
    fn from(response: Response) -> Self {
        Self {
            code: GrowthbookErrorCode::GrowthbookGateway,
            message: format!("Failed to get features. StatusCode={}", response.status()),
        }
    }
}
