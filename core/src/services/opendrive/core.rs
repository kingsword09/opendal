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
use http::Response;
use reqwest::Client;
use serde_json::json;
use tokio::sync::Mutex;

use crate::raw::*;
use crate::services::opendrive::error::new_proxy_request_build_error;
use crate::services::opendrive::error::parse_numeric_types_error;
use crate::services::opendrive::model::*;
use crate::*;

pub mod constants {
    // Opendrive base URL.
    pub(crate) const OPENDRIVE_BASE_URL: &str = "https://dev.opendrive.com/api/v1";

    // Partner ID. “OpenDrive” is default partner value
    pub(crate) const OPENDRIVE_CLIENT_ID: &str = "opendrive";

    // OAUTH 2.0 Session Id
    pub(crate) const OPENDRIVE_SESSION_ID: &str = "OAUTH";

    // opendrive root:/ folder id
    pub(crate) const OPENDRIVE_SLASH_ROOT_FOLDER_ID: &str = "0";
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

    fn parse_file_metadata(&self, file: OpendriveGetFileInfo) -> Result<Metadata> {
        // Parse since time once for both time-based conditions
        let last_modified = parse_datetime_from_from_timestamp(
            file.date_modified
                .parse()
                .map_err(parse_numeric_types_error)?,
        )
        .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

        let metadata = Metadata::new(EntryMode::FILE)
            .with_last_modified(last_modified)
            .with_version(file.version)
            .with_etag(file.file_id)
            .with_content_length(file.size.parse().map_err(parse_numeric_types_error)?);

        Ok(metadata)
    }

    async fn parse_folder_metadata(&self, folder: OpendriveGetFolderInfo) -> Result<Metadata> {
        let last_modified = parse_datetime_from_from_timestamp(
            folder
                .date_modified
                .parse()
                .map_err(parse_numeric_types_error)?,
        )
        .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

        let metadata = Metadata::new(EntryMode::DIR)
            .with_last_modified(last_modified)
            .with_etag(folder.folder_id);

        Ok(metadata)
    }

    pub(crate) async fn parse_id_by_metadata(
        &self,
        path: &str,
        metadata: Metadata,
    ) -> Result<String> {
        match metadata.etag() {
            Some(etag) => Ok(etag.to_string()),
            None => {
                if metadata.is_file() {
                    Ok(self.get_file_id(path).await?)
                } else {
                    Ok(self.get_folder_id(path).await?)
                }
            }
        }
    }

    async fn get_folder_id(&self, path: &str) -> Result<String> {
        if path == "/" {
            return Ok(constants::OPENDRIVE_SLASH_ROOT_FOLDER_ID.to_string());
        }

        let url = format!("{}/folder/idbypath.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "path": path,
            "session_id": constants::OPENDRIVE_SESSION_ID
        });

        let req = Request::post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveGetFolderIdResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetFolderIdResponse::Success(result) => Ok(result.folder_id),
                OpendriveGetFolderIdResponse::Fail(result) => {
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn get_folder_info(&self, folder_id: &str) -> Result<OpendriveGetFolderInfo> {
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
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn get_file_id(&self, path: &str) -> Result<String> {
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
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn get_file_info(&self, file_id: &str) -> Result<OpendriveGetFileInfo> {
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
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn create_folder(
        &self,
        name: &str,
        path: &str,
        folder_id: Option<String>,
    ) -> Result<String> {
        let url = format!("{}/folder.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = match folder_id {
            Some(folder_id) => json!({
                "folder_name": name,
                "folder_sub_parent": folder_id,
                "session_id": constants::OPENDRIVE_SESSION_ID,
            }),
            None => json!({
                "folder_name": name,
                "session_id": constants::OPENDRIVE_SESSION_ID,
            }),
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
                    if result.error.code == 409 {
                        let folder_id = self.get_folder_id(path).await?;
                        return Ok(folder_id);
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn get_list_info(&self, folder_id: &str) -> Result<OpendriveGetListInfo> {
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

        let parsed_res: Result<OpendriveGetListInfoResponse, serde_json::Error> =
            serde_json::from_reader(res.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveGetListInfoResponse::Success(result) => Ok(result),
                OpendriveGetListInfoResponse::Fail(result) => {
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn recursive_list(
        &self,
        folder: OpendriveGetFolderInfo,
        parent: &str,
    ) -> Result<(Vec<Entry>, Vec<Entry>)> {
        let mut files = vec![];
        let mut folders = vec![];
        let res = self.get_list_info(&folder.folder_id).await?;

        for info in res.files {
            let metadata = self.parse_file_metadata(info.clone())?;
            let path = build_abs_path(parent, &info.name);
            files.push(Entry::new(path, metadata));
        }

        for info in res.folders {
            let metadata = self.parse_folder_metadata(info.clone()).await?;
            let path = build_abs_path(parent, &info.name);
            folders.push(Entry::new(path, metadata));
        }

        Ok((files, folders))
    }

    fn parse_response_unit(&self, resp: Response<Buffer>) -> Result<()> {
        let parsed_res: Result<OpendriveSuccessIgnoreResponse, serde_json::Error> =
            serde_json::from_reader(resp.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveSuccessIgnoreResponse::Success => Ok(()),
                OpendriveSuccessIgnoreResponse::Fail(result) => {
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    } else if result.error.code == 409 {
                        return Err(Error::new(ErrorKind::AlreadyExists, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn rename_folder(&self, from: &str, to: &str) -> Result<()> {
        let from = build_abs_path(&self.root, from);
        let to = build_abs_path(&self.root, to);

        if from == build_abs_path(&self.root, "") || to == build_abs_path(&self.root, "") {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "renaming root directory is not supported",
            ));
        }

        let folder_id = self.get_folder_id(&from).await?;
        let folder_name = get_basename(&to);

        let url = format!("{}/folder/rename.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "folder_id": &folder_id,
            "folder_name": &folder_name
        });

        let req = Request::post(&url)
            .extension(Operation::Rename)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn rename_file(&self, from: &str, to: &str) -> Result<()> {
        let from = build_abs_path(&self.root, from);
        let to = build_abs_path(&self.root, to);

        let file_id = self.get_file_id(&from).await?;
        let file_name = get_basename(&to);

        let url = format!("{}/file/rename.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "file_id": &file_id,
            "new_file_name": &file_name
        });

        let req = Request::post(&url)
            .extension(Operation::Rename)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn copy_folder(&self, from: &str, to: &str) -> Result<()> {
        let from = build_abs_path(&self.root, from);
        let to = build_abs_path(&self.root, to);

        let dst_folder_path = get_parent(&to);
        let folder_name = get_basename(&to);

        let folder_id = self.get_folder_id(&from).await?;
        let dst_folder_id = self.get_folder_id(dst_folder_path).await?;

        let url = format!("{}/folder/move_copy.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "folder_id": &folder_id,
            "dst_folder_id": &dst_folder_id,
            "move": false,
            "copy_recursive": true,
            "new_folder_name": &folder_name
        });

        let req = Request::post(&url)
            .extension(Operation::Copy)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn copy_file(&self, from: &str, to: &str) -> Result<()> {
        let from = build_abs_path(&self.root, from);
        let to = build_abs_path(&self.root, to);

        let dst_folder_path = get_parent(&to);
        let file_name = get_basename(&to);

        let file_id = self.get_file_id(&from).await?;
        let dst_folder_id = self.get_folder_id(dst_folder_path).await?;

        let url = format!("{}/file/move_copy.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "src_file_id": &file_id,
            "dst_folder_id": &dst_folder_id,
            "move": false,
            "new_file_name": &file_name
        });

        let req = Request::post(&url)
            .extension(Operation::Copy)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn trash_folder(&self, folder_id: &str) -> Result<()> {
        let url = format!("{}/folder/trash.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "folder_id": folder_id
        });

        let req = Request::post(&url)
            .extension(Operation::Delete)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn trash_file(&self, file_id: &str) -> Result<()> {
        let url = format!("{}/file/trash.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "file_id": file_id
        });

        let req = Request::post(&url)
            .extension(Operation::Delete)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn remove_trash_folder(&self, folder_id: &str) -> Result<()> {
        let url = format!("{}/folder/remove.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "folder_id": folder_id
        });

        let req = Request::post(&url)
            .extension(Operation::Delete)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn remove_trash_file(&self, file_id: &str) -> Result<()> {
        let url = format!("{}/file/remove.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "file_id": file_id
        });

        let req = Request::post(&url)
            .extension(Operation::Delete)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn check_if_file_exists(&self, path: &str) -> Result<bool> {
        let path = build_abs_path(&self.root, path);

        let parent = get_parent(&path);
        let file_name = get_basename(&path);

        let folder_id = self.get_folder_id(parent).await?;

        let url = format!(
            "{}/upload/checkfileexistsbyname.json/{}",
            constants::OPENDRIVE_BASE_URL,
            &folder_id
        );
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "name": vec![file_name]
        });

        let req = Request::post(&url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveCheckIfExistsResponse, serde_json::Error> =
            serde_json::from_reader(resp.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveCheckIfExistsResponse::Success(result) => {
                    if !result.result.is_empty() {
                        Ok(true)
                    } else {
                        Ok(false)
                    }
                }
                OpendriveCheckIfExistsResponse::Fail(result) => {
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    pub(crate) async fn create_file(&self, path: &str) -> Result<OpendriveCreateFileInfo> {
        let parent = get_parent(path);
        let file_name = get_basename(path);

        let folder_id = self.get_folder_id(parent).await?;

        let url = format!("{}/upload/create_file.json", constants::OPENDRIVE_BASE_URL);
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "folder_id": &folder_id,
            "file_name": &file_name,
            // (1) - file info will be returned if file already exists,
            // (0) - error 409 will be returned if file already exists.
            "open_if_exists": 1
        });

        let req = Request::post(&url)
            .extension(Operation::Write)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveCreateFileResponse, serde_json::Error> =
            serde_json::from_reader(resp.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveCreateFileResponse::Success(result) => Ok(result),
                OpendriveCreateFileResponse::Fail(result) => {
                    if result.error.code == 409 {
                        return Err(Error::new(ErrorKind::AlreadyExists, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn open_file_upload(
        &self,
        file_id: &str,
        file_size: u64,
    ) -> Result<OpendriveOpenFileUploadInfo> {
        let url = format!(
            "{}/upload/open_file_upload.json",
            constants::OPENDRIVE_BASE_URL
        );
        let url = self.sign(&url).await?;

        let body = json!({
            "session_id": constants::OPENDRIVE_SESSION_ID,
            "file_id": file_id,
            "file_size": file_size
        });

        let req = Request::post(&url)
            .extension(Operation::Write)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveOpenFileUploadResponse, serde_json::Error> =
            serde_json::from_reader(resp.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveOpenFileUploadResponse::Success(result) => Ok(result),
                OpendriveOpenFileUploadResponse::Fail(result) => {
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
                }
            },
            Err(err) => Err(new_json_deserialize_error(err)),
        }
    }

    async fn upload_file_chunk(
        &self,
        file_id: &str,
        file_name: &str,
        temp_location: &str,
        offset: usize,
        chunk: Buffer,
    ) -> Result<()> {
        let url = format!(
            "{}/upload/upload_file_chunk.json",
            constants::OPENDRIVE_BASE_URL
        );
        let url = self.sign(&url).await?;

        let req = Request::post(&url).extension(Operation::Write);

        let chunk_size = chunk.len();
        let file_part = FormDataPart::new("file")
            .header(
                header::CONTENT_DISPOSITION,
                format!("form-data; name=\"file\"; filename=\"{file_name}\"")
                    .parse()
                    .unwrap(),
            )
            .content(chunk);

        let multipart = Multipart::new()
            .part(FormDataPart::new("session_id").content(constants::OPENDRIVE_SESSION_ID))
            .part(FormDataPart::new("file_id").content(file_id.to_string()))
            .part(FormDataPart::new("temp_location").content(temp_location.to_string()))
            .part(FormDataPart::new("chunk_offset").content(offset.to_string()))
            .part(FormDataPart::new("chunk_size").content(chunk_size.to_string()))
            .part(file_part);

        let req = multipart.apply(req)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn upload_file_chunk_second(
        &self,
        file_id: &str,
        file_name: &str,
        chunk: Buffer,
    ) -> Result<()> {
        let url = format!(
            "{}/upload/upload_file_chunk2.json/{}/{}",
            constants::OPENDRIVE_BASE_URL,
            constants::OPENDRIVE_SESSION_ID,
            file_id
        );
        let url = self.sign(&url).await?;

        let req = Request::post(&url).extension(Operation::Write);

        let file_part = FormDataPart::new("file")
            .header(
                header::CONTENT_DISPOSITION,
                format!("form-data; name=\"file\"; filename=\"{file_name}\"")
                    .parse()
                    .unwrap(),
            )
            .content(chunk);

        let multipart = Multipart::new().part(file_part);

        let req = multipart.apply(req)?;

        let resp = self.info.http_client().send(req).await?;

        self.parse_response_unit(resp)
    }

    async fn close_file_upload(
        &self,
        file_id: &str,
        file_size: u64,
        temp_location: Option<String>,
    ) -> Result<OpendriveCloseFileUploadInfo> {
        let url = format!(
            "{}/upload/close_file_upload.json",
            constants::OPENDRIVE_BASE_URL
        );
        let url = self.sign(&url).await?;

        let body = match temp_location {
            Some(temp_location) => json!({
                "session_id": constants::OPENDRIVE_SESSION_ID,
                "file_id": file_id.to_string(),
                "file_size": file_size.to_string(),
                "temp_location": temp_location
            }),
            None => json!({
                "session_id": constants::OPENDRIVE_SESSION_ID,
                "file_id": file_id.to_string(),
                "file_size": file_size.to_string(),
            }),
        };

        let req = Request::post(&url)
            .extension(Operation::Write)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Buffer::from(body.to_string()))
            .map_err(new_request_build_error)?;

        let resp = self.info.http_client().send(req).await?;

        let parsed_res: Result<OpendriveCloseFileUploadResponse, serde_json::Error> =
            serde_json::from_reader(resp.body().clone().reader());

        match parsed_res {
            Ok(parsed_res) => match parsed_res {
                OpendriveCloseFileUploadResponse::Success(result) => Ok(result),
                OpendriveCloseFileUploadResponse::Fail(result) => {
                    if result.error.code == 404 {
                        return Err(Error::new(ErrorKind::NotFound, result.error.message));
                    }

                    Err(Error::new(ErrorKind::Unexpected, result.error.message))
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

    pub(crate) async fn stat(&self, path: &str, args: Option<OpStat>) -> Result<Metadata> {
        match self.get_folder_id(path).await {
            Ok(folder_id) => {
                let result = self.get_folder_info(&folder_id).await?;

                // Parse since time once for both time-based conditions
                let last_modified = parse_datetime_from_from_timestamp(
                    result
                        .date_modified
                        .parse()
                        .map_err(parse_numeric_types_error)?,
                )
                .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

                if let Some(args) = args {
                    // Check if_match condition
                    if let Some(if_match) = &args.if_match() {
                        if if_match != &folder_id {
                            return Err(Error::new(
                                ErrorKind::ConditionNotMatch,
                                "doesn't match the condition if_match",
                            ));
                        }
                    }

                    // Check if_none_match condition
                    if let Some(if_none_match) = &args.if_none_match() {
                        if if_none_match == &folder_id {
                            return Err(Error::new(
                                ErrorKind::ConditionNotMatch,
                                "doesn't match the condition if_none_match",
                            ));
                        }
                    }

                    // Check modified_since condition
                    if let Some(modified_since) = &args.if_modified_since() {
                        if !last_modified.gt(modified_since) {
                            return Err(Error::new(
                                ErrorKind::ConditionNotMatch,
                                "doesn't match the condition if_modified_since",
                            ));
                        }
                    }

                    // Check unmodified_since condition
                    if let Some(unmodified_since) = &args.if_unmodified_since() {
                        if !last_modified.le(unmodified_since) {
                            return Err(Error::new(
                                ErrorKind::ConditionNotMatch,
                                "doesn't match the condition if_unmodified_since",
                            ));
                        }
                    }
                }

                Ok(self.parse_folder_metadata(result).await?)
            }
            Err(err) => {
                // If not found as file, try folder
                if matches!(err.kind(), ErrorKind::NotFound) {
                    let file_id = self.get_file_id(path).await?;

                    let result = self.get_file_info(&file_id).await?;

                    // Parse since time once for both time-based conditions
                    let last_modified = parse_datetime_from_from_timestamp(
                        result
                            .date_modified
                            .parse()
                            .map_err(parse_numeric_types_error)?,
                    )
                    .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

                    if let Some(args) = args {
                        // Check if_match condition
                        if let Some(if_match) = &args.if_match() {
                            if if_match != &file_id {
                                return Err(Error::new(
                                    ErrorKind::ConditionNotMatch,
                                    "doesn't match the condition if_match",
                                ));
                            }
                        }

                        // Check if_none_match condition
                        if let Some(if_none_match) = &args.if_none_match() {
                            if if_none_match == &file_id {
                                return Err(Error::new(
                                    ErrorKind::ConditionNotMatch,
                                    "doesn't match the condition if_none_match",
                                ));
                            }
                        }

                        // Check modified_since condition
                        if let Some(modified_since) = &args.if_modified_since() {
                            if !last_modified.gt(modified_since) {
                                return Err(Error::new(
                                    ErrorKind::ConditionNotMatch,
                                    "doesn't match the condition if_modified_since",
                                ));
                            }
                        }

                        // Check unmodified_since condition
                        if let Some(unmodified_since) = &args.if_unmodified_since() {
                            if !last_modified.le(unmodified_since) {
                                return Err(Error::new(
                                    ErrorKind::ConditionNotMatch,
                                    "doesn't match the condition if_unmodified_since",
                                ));
                            }
                        }

                        // Check unmodified_since condition
                        if let Some(version) = &args.version() {
                            if version != &result.version {
                                return Err(Error::new(
                                    ErrorKind::ConditionNotMatch,
                                    "doesn't match the condition version",
                                ));
                            }
                        }
                    }

                    Ok(self.parse_file_metadata(result)?)
                } else {
                    Err(err)
                }
            }
        }
    }

    pub(crate) async fn read(&self, path: &str, args: OpRead) -> Result<Buffer> {
        let metadata = self.stat(path, None).await?;

        if metadata.is_dir() {
            return Err(Error::new(ErrorKind::IsADirectory, "path should be a file"));
        }

        let file_id = self.parse_id_by_metadata(path, metadata).await?;

        let url = format!(
            "{}/download/file.json/{}",
            constants::OPENDRIVE_BASE_URL,
            &file_id
        );
        let url = self.sign(&url).await?;
        let mut url = QueryPairsWriter::new(&url);

        let range = args.range();
        url = url.push("session_id", constants::OPENDRIVE_SESSION_ID);
        url = url.push("offset", &range.offset().to_string());

        let req = Request::get(url.finish().to_string())
            .extension(Operation::Read)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header(header::RANGE, range.to_header())
            .body(Buffer::new())
            .map_err(new_request_build_error)?;

        let res = self.info.http_client().send(req).await?;

        let mut body = res.into_body();
        if let Some(size) = range.size() {
            let slice_end = body.len().min(size as usize);
            body = body.slice(0..slice_end);
        }

        Ok(body)
    }

    pub(crate) async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let path = build_abs_path(&self.root, from);

        let metadata = self.stat(&path, None).await?;

        if metadata.is_file() {
            self.rename_file(from, to).await
        } else {
            self.rename_folder(from, to).await
        }
    }

    pub(crate) async fn copy(&self, from: &str, to: &str) -> Result<()> {
        let metadata = self.stat(from, None).await?;

        if metadata.is_file() {
            self.copy_file(from, to).await
        } else {
            self.copy_folder(from, to).await
        }
    }

    pub(crate) async fn delete(&self, path: &str, version: Option<&str>) -> Result<()> {
        let path = build_abs_path(&self.root, path);
        let metadata = self.stat(&path, None).await?;

        if version.is_some() && metadata.version() != version {
            return Err(Error::new(ErrorKind::ConditionNotMatch, "version mismatch"));
        }

        if metadata.is_file() {
            let file_id = self.parse_id_by_metadata(&path, metadata).await?;

            self.trash_file(&file_id).await?;
            self.remove_trash_file(&file_id).await
        } else {
            let folder_id = self.parse_id_by_metadata(&path, metadata).await?;

            self.trash_folder(&folder_id).await?;
            self.remove_trash_folder(&folder_id).await
        }
    }

    pub(crate) async fn list(&self, path: &str, args: OpList) -> Result<Vec<Entry>> {
        let path = build_abs_path(&self.root, path);

        let folder_id = if path == build_abs_path(&self.root, "") {
            "0"
        } else {
            &self.get_folder_id(&path).await?
        };

        let mut entry_list = vec![];

        let info = self.get_folder_info(folder_id).await?;
        let metadata = self.parse_folder_metadata(info.clone()).await?;
        entry_list.push(Entry::new(path.clone(), metadata));

        let (files, folders) = self.recursive_list(info, &path).await?;
        entry_list.extend(files);

        // Only process folders recursively if requested
        if args.recursive() {
            let mut folders_to_process = folders.clone();
            entry_list.extend(folders);

            // Process folders that have children
            while let Some(folder_entry) = folders_to_process.pop() {
                if let Ok(folder_info) = self
                    .get_folder_info(folder_entry.metadata().etag().unwrap_or(""))
                    .await
                {
                    if folder_info.child_folders.unwrap_or_default() > 0 {
                        let (child_files, child_folders) = self
                            .recursive_list(folder_info, folder_entry.path())
                            .await?;
                        entry_list.extend(child_files);
                        entry_list.extend(child_folders.clone());
                        folders_to_process.extend(child_folders);
                    }
                }
            }
        } else {
            entry_list.extend(folders);
        }

        Ok(entry_list)
    }

    pub(crate) async fn write_prepare(
        &self,
        path: &str,
        op: &OpWrite,
    ) -> Result<OpendriveCreateFileInfo> {
        let if_exists = self.check_if_file_exists(path).await?;

        if if_exists {
            if op.if_not_exists() {
                return Err(Error::new(ErrorKind::AlreadyExists, "file already exists"));
            }
        } else {
            let result = self.get_folder_id(path).await;

            if result.is_ok() {
                return Err(Error::new(
                    ErrorKind::IsADirectory,
                    "directory does not support write operations",
                ));
            }
        }

        let result = self.create_file(path).await?;
        self.close_file_upload(&result.file_id, 0, Some(result.clone().temp_location))
            .await?;

        Ok(result)
    }

    pub(crate) async fn write_once(
        &self,
        path: &str,
        chunk: Buffer,
        op: &OpWrite,
    ) -> Result<OpendriveGetFileInfo> {
        let info = self.write_prepare(path, op).await?;

        let file_size = chunk.len() as u64;

        self.open_file_upload(&info.file_id, file_size).await?;

        let file_name = get_basename(path);
        self.upload_file_chunk_second(&info.file_id, file_name, chunk)
            .await?;

        let result = self
            .close_file_upload(&info.file_id, file_size, Some(info.temp_location))
            .await?;

        Ok(OpendriveGetFileInfo {
            file_id: info.file_id,
            name: file_name.to_string(),
            size: result.size,
            version: result.version,
            date_modified: result.date_modified,
        })
    }

    pub(crate) async fn write_append(
        &self,
        path: &str,
        file_size: u64,
        offset: u64,
        chunk: Buffer,
        op: &OpWrite,
    ) -> Result<OpendriveGetFileInfo> {
        let info = self.write_prepare(path, op).await?;
        let file_id = info.file_id;

        let info = self.open_file_upload(&file_id, file_size).await?;

        let file_name = get_basename(path);
        self.upload_file_chunk(
            &file_id,
            file_name,
            &info.temp_location,
            offset as usize,
            chunk,
        )
        .await?;

        let info = self
            .close_file_upload(&file_id, file_size, Some(info.temp_location))
            .await?;

        Ok(OpendriveGetFileInfo {
            file_id,
            name: file_name.to_string(),
            size: info.size,
            version: info.version,
            date_modified: info.date_modified,
        })
    }
}

// OpenDrive is supporting simplified OAuth 2.0 standard for authorization (Resource Owner Password Credentials Flow).
pub struct OpendriveSigner {
    pub username: String,
    pub password: String,
    pub refresh_token: String,
    pub access_token: String,
    pub expires_in: DateTime<Utc>,

    pub auth_http_client: Client,
}

impl OpendriveSigner {
    pub fn new(auth_http_client: Client, username: &str, password: &str) -> Self {
        OpendriveSigner {
            auth_http_client,
            username: username.to_string(),
            password: password.to_string(),
            refresh_token: "".to_string(),
            access_token: "".to_string(),
            expires_in: DateTime::<Utc>::MIN_UTC,
        }
    }

    async fn refresh_tokens(&mut self, oauth_body: serde_json::Value) -> Result<()> {
        let url = format!("{}/oauth2/grant.json", constants::OPENDRIVE_BASE_URL);

        let request = self
            .auth_http_client
            .post(url)
            .header("Content-Type", "application/json")
            .body(oauth_body.to_string())
            .build()
            .map_err(new_proxy_request_build_error)?;
        let resp = self
            .auth_http_client
            .execute(request)
            .await
            .map_err(new_proxy_request_build_error)?;
        let resp_body = resp.text().await.map_err(new_proxy_request_build_error)?;

        let data: Result<OAuthTokenResponse, serde_json::Error> = serde_json::from_str(&resp_body);

        match data {
            Ok(data) => match data {
                OAuthTokenResponse::Success(data) => {
                    self.access_token = data.access_token;
                    self.refresh_token = data.refresh_token;
                    self.expires_in = Utc::now()
                        + chrono::TimeDelta::try_seconds(data.expires_in)
                            .expect("expires_in must be valid seconds")
                        - chrono::TimeDelta::minutes(2); // assumes 2 mins graceful transmission for implementation simplicity

                    Ok(())
                }
                OAuthTokenResponse::Fail(err) => {
                    if err.error.code == 401 {
                        return Err(Error::new(ErrorKind::ConfigInvalid, "invalid credentials"));
                    }

                    Err(Error::new(ErrorKind::Unexpected, err.error.message))
                }
            },
            Err(err) => Err(Error::new(ErrorKind::Unexpected, err.to_string())),
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
