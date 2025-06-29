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

use std::fmt::Debug;
use std::fmt::Formatter;
use std::sync::Arc;

use bytes::Buf;
use chrono::DateTime;
use chrono::Utc;
use http::header;
use http::Request;
use http::StatusCode;
use serde_json::json;
use tokio::sync::Mutex;

use super::error::parse_error;
use crate::raw::*;
use crate::services::opendrive::model::OAuthGrantType;
use crate::services::opendrive::model::OAuthTokenResponseBody;
use crate::services::opendrive::model::OpendriveCreateDirResponse;
use crate::services::opendrive::model::OpendriveGetFileIdResponse;
use crate::services::opendrive::model::OpendriveGetFileInfo;
use crate::services::opendrive::model::OpendriveGetFileInfoResponse;
use crate::services::opendrive::model::OpendriveGetFolderIdResponse;
use crate::services::opendrive::model::OpendriveGetFolderInfo;
use crate::services::opendrive::model::OpendriveGetFolderInfoResponse;
use crate::*;

pub mod constants {
    // Opendrive base URL.
    pub(crate) const OPENDRIVE_BASE_URL: &str = "https://dev.opendrive.com/api/v1";

    // Partner ID. “OpenDrive” is default partner value
    pub(crate) const OPENDRIVE_CLIENT_ID: &str = "opendrive";

    // OAUTH 2.0 Session Id
    pub(crate) const OPENDRIVE_SESSION_ID: &str = "OAUTH";
}

pub struct OpendriveCore {
    pub info: Arc<AccessorInfo>,
    pub root: String,
    pub signer: Arc<Mutex<OpendriveSigner>>,
}

impl Debug for OpendriveCore {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpendriveCore")
            .field("root", &self.info.root())
            .finish_non_exhaustive()
    }
}

// organizes a few core module functions
impl OpendriveCore {
    pub(crate) async fn sign(&self, url: &str) -> Result<String> {
        let mut signer = self.signer.lock().await;
        signer.sign(url).await
    }

    pub(crate) async fn get_folder_id(&self, path: &str) -> Result<String> {
        let url = format!("{}/folder/idbypath.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "path": path,
            "session_id": constants::OPENDRIVE_SESSION_ID
        });

        let req = Request::post(url)
            .header(header::CONTENT_TYPE, "applicaton/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveGetFolderIdResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetFolderIdResponse::Success(result) => Ok(result.folder_id),
                OpendriveGetFolderIdResponse::Fail(result) => {
                    if result.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    pub(crate) async fn get_folder_info(&self, folder_id: &str) -> Result<OpendriveGetFolderInfo> {
        let url = format!(
            "{}/folder/info.json/{}/{}",
            constants::OPENDRIVE_BASE_URL,
            constants::OPENDRIVE_SESSION_ID,
            folder_id
        );
        let url = self.sign(&url).await?;

        let req = Request::get(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::new())
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveGetFolderInfoResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetFolderInfoResponse::Success(result) => Ok(result),
                OpendriveGetFolderInfoResponse::Fail(result) => {
                    if result.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    pub(crate) async fn get_file_id(&self, path: &str) -> Result<String> {
        let url = format!("{}/file/idbypath.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "path": path.trim_start_matches('/'),
            "session_id": constants::OPENDRIVE_SESSION_ID,
        });

        let req = Request::post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveGetFileIdResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetFileIdResponse::Success(result) => Ok(result.file_id),
                OpendriveGetFileIdResponse::Fail(result) => {
                    if result.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    pub(crate) async fn get_file_info(&self, file_id: &str) -> Result<OpendriveGetFileInfo> {
        let url = format!(
            "{}/file/info.json/{}",
            constants::OPENDRIVE_BASE_URL,
            file_id
        );
        let url = self.sign(&url).await?;
        let mut url = QueryPairsWriter::new(&url);
        url = url.push("session_id", constants::OPENDRIVE_SESSION_ID);

        let req = Request::get(url.finish().to_string())
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::new())
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveGetFileInfoResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetFileInfoResponse::Success(result) => Ok(result),
                OpendriveGetFileInfoResponse::Fail(result) => {
                    if result.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    pub(crate) async fn create_folder(
        &self,
        name: &str,
        path: &str,
        folder_id: Option<String>,
    ) -> Result<String> {
        let url = format!("{}/folder.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = if folder_id.is_some() {
            json!({
                "folder_name": name,
                "folder_sub_parent": folder_id.unwrap(),
                "session_id": constants::OPENDRIVE_SESSION_ID,
            })
        } else {
            json!({
                "folder_name": name,
                "session_id": constants::OPENDRIVE_SESSION_ID,
            })
        };

        let req = Request::post(url)
            .extension(Operation::CreateDir)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveCreateDirResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveCreateDirResponse::Success(result) => Ok(result.folder_id),
                OpendriveCreateDirResponse::Fail(result) => {
                    if result.code == 409 {
                        let folder_id = self.get_folder_id(path).await?;
                        return Ok(folder_id);
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }
}

// `services-opendrive` rest api guide
// Read more at https://www.opendrive.com/wp-content/uploads/guides/OpenDrive_API_guide.pdf
impl OpendriveCore {
    /// Create a directory
    ///
    /// When creating a folder, OpenDrive returns error code 409 if the folder already exists.
    pub(crate) async fn opendrive_create_dir(&self, path: &str) -> Result<()> {
        if path == "/" {
            return Ok(());
        }

        let path = path.trim_start_matches('/').trim_end_matches('/');

        let mut current_path = String::new();
        let mut parent_folder_id = None;

        // Iterate through each folder in the path
        for folder in path.split('/') {
            current_path.push('/');
            current_path.push_str(folder);

            // Create folder and get its ID
            let folder_id = self
                .create_folder(folder, &current_path, parent_folder_id)
                .await?;
            parent_folder_id = Some(folder_id);
        }

        Ok(())
    }

    pub(crate) async fn list(&self, folder_id: &str, args: OpList) -> Result<String> {
        let url = format!(
            "{}/folder/list.json/{}/{}",
            constants::OPENDRIVE_BASE_URL,
            constants::OPENDRIVE_SESSION_ID,
            folder_id
        );
        let url = self.sign(&url).await?;

        let req = Request::get(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::new())
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveGetFileIdResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetFileIdResponse::Success(result) => Ok(result.file_id),
                OpendriveGetFileIdResponse::Fail(result) => {
                    if result.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }
}

// OpenDrive is supporting simplified OAuth 2.0 standard for authorization (Resource Owner Password Credentials Flow).
pub struct OpendriveSigner {
    pub info: Arc<AccessorInfo>, // to use `http_client`

    pub username: String,
    pub password: String,
    pub refresh_token: String,
    pub access_token: String,
    pub expires_in: DateTime<Utc>,
}

impl OpendriveSigner {
    pub fn new(info: Arc<AccessorInfo>) -> Self {
        OpendriveSigner {
            info,

            username: "".to_string(),
            password: "".to_string(),
            refresh_token: "".to_string(),
            access_token: "".to_string(),
            expires_in: DateTime::<Utc>::MIN_UTC,
        }
    }

    async fn refresh_tokens(&mut self, oauth_body: serde_json::Value) -> Result<()> {
        let url = format!("{}/oauth2/grant.json", constants::OPENDRIVE_BASE_URL);

        let request = Request::post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(oauth_body.to_string()))
            .map_err(new_request_build_error)?;

        let response = self.info.http_client().send(request).await?;
        match response.status() {
            StatusCode::OK => {
                let resp_body = response.into_body();
                let data: OAuthTokenResponseBody = serde_json::from_reader(resp_body.reader())
                    .map_err(new_json_deserialize_error)?;
                self.access_token = data.access_token;
                self.refresh_token = data.refresh_token;
                self.expires_in = Utc::now()
                    + chrono::TimeDelta::try_seconds(data.expires_in)
                        .expect("expires_in must be valid seconds")
                    - chrono::TimeDelta::minutes(2); // assumes 2 mins graceful transmission for implementation simplicity
                Ok(())
            }
            _ => Err(parse_error(response)),
        }
    }

    /// Sign a request.
    pub async fn sign(&mut self, url: &str) -> Result<String> {
        if !self.access_token.is_empty() && self.expires_in > Utc::now() {
            let mut url = QueryPairsWriter::new(url);
            url = url.push("access_token", &self.access_token);
            return Ok(url.finish());
        }

        let oauth_body = if self.access_token.is_empty() {
            json!({
                "grant_type": OAuthGrantType::Password,
                "client_id": constants::OPENDRIVE_CLIENT_ID,
                "username": self.username,
                "password": self.password
            })
        } else {
            json!({
                "grant_type": OAuthGrantType::RefreshToken,
                "client_id": constants::OPENDRIVE_CLIENT_ID,
                "refresh_token": self.refresh_token
            })
        };

        self.refresh_tokens(oauth_body).await?;

        let mut url = QueryPairsWriter::new(url);
        url = url.push("access_token", &self.access_token);

        Ok(url.finish())
    }
}
