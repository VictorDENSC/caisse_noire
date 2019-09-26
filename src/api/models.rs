use rouille::input::json::JsonError;
use serde::Serialize;

use crate::database::postgres::DbError;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub kind: ErrorKind,
    pub description: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    ServiceUnavailable,
    Unknown,
    NotFound,
    Json,
}

impl ErrorKind {
    pub fn status_code(&self) -> u16 {
        match self {
            ErrorKind::ServiceUnavailable => 500,
            ErrorKind::Unknown => 500,
            ErrorKind::NotFound => 404,
            ErrorKind::Json => 400,
        }
    }
}

impl From<ErrorResponse> for rouille::Response {
    fn from(response: ErrorResponse) -> rouille::Response {
        rouille::Response::json(&response).with_status_code(response.kind.status_code())
    }
}

impl From<DbError> for ErrorResponse {
    fn from(error: DbError) -> Self {
        match error {
            DbError::NotFound => ErrorResponse {
                kind: ErrorKind::NotFound,
                description: String::from("Not found"),
            },
            DbError::Unknown => ErrorResponse {
                kind: ErrorKind::Unknown,
                description: String::from("An internal error occured"),
            },
            DbError::ServiceUnavailable => ErrorResponse {
                kind: ErrorKind::ServiceUnavailable,
                description: String::from("The service is currently unavailable"),
            },
        }
    }
}

impl From<JsonError> for ErrorResponse {
    fn from(error: JsonError) -> ErrorResponse {
        ErrorResponse {
            kind: ErrorKind::Json,
            description: error.to_string(),
        }
    }
}

pub mod test_utils {
    use rouille::Request;
    use serde::Serialize;

    pub struct RequestBuilder;

    impl RequestBuilder {
        fn json_header() -> (String, String) {
            ("Content-Type".to_string(), "application/json".to_string())
        }

        pub fn get(url: String) -> Request {
            Request::fake_http("GET", url, vec![RequestBuilder::json_header()], vec![])
        }

        pub fn post<T>(url: String, data: &T) -> Result<Request, ()>
        where
            T: Serialize,
        {
            match serde_json::to_vec(data) {
                Ok(serialized_data) => Ok(Request::fake_http(
                    "POST",
                    url,
                    vec![RequestBuilder::json_header()],
                    serialized_data,
                )),
                Err(_) => Err(()),
            }
        }
    }

}
