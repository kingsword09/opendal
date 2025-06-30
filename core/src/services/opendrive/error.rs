// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use bytes::Buf;
use http::Response;

use crate::services::opendrive::model::OpendriveDeserializeFailError;
use crate::{Buffer, Error, ErrorKind};

/// Parse error response into Error.
pub(super) fn parse_error(response: Response<Buffer>) -> Error {
    let result: Result<OpendriveDeserializeFailError, serde_json::Error> =
        serde_json::from_reader(response.body().clone().reader());

    let (kind, message) = match result {
        Ok(result) => match result.code {
            400 => (ErrorKind::Unexpected, result.message),
            401 => (ErrorKind::PermissionDenied, result.message),
            404 => (ErrorKind::NotFound, result.message),
            409 => (ErrorKind::AlreadyExists, result.message),
            _ => (ErrorKind::Unexpected, result.message),
        },
        Err(_) => (
            ErrorKind::Unexpected,
            "unexpected error occurred during deserialization".to_string(),
        ),
    };

    Error::new(kind, message)
}

pub(super) fn parse_numeric_types_error(err: impl std::error::Error) -> Error {
    Error::new(ErrorKind::Unexpected, format!("parse Numeric types error: {}", err))
}
