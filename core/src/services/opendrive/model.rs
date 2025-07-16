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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub enum OAuthGrantType {
    #[serde(rename = "password")]
    Password,
    #[serde(rename = "refresh_token")]
    RefreshToken,
}

#[derive(Debug, Deserialize)]
pub struct OAuthTokenResponseBody {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OAuthTokenResponse {
    Success(OAuthTokenResponseBody),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveCreateDirResponse {
    Success(OpendriveCreateDirSuccess),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
pub struct OpendriveCreateDirSuccess {
    #[serde(rename = "FolderID")]
    pub folder_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "DateCreated")]
    pub date_created: u32,
    #[serde(rename = "DirUpdateTime")]
    pub dir_update_time: u32,
    #[serde(rename = "DateModified")]
    pub date_modified: u32,
}

#[derive(Debug, Deserialize)]
pub struct OpendriveDeserializeFailError {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct OpendriveDeserializeFail {
    pub error: OpendriveDeserializeFailError,
}

#[derive(Debug, Deserialize)]
pub struct OpendriveGetFolderId {
    #[serde(rename = "FolderId")]
    pub folder_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveGetFolderIdResponse {
    Success(OpendriveGetFolderId),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
pub struct OpendriveGetFileId {
    #[serde(rename = "FileId")]
    pub file_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveGetFileIdResponse {
    Success(OpendriveGetFileId),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpendriveGetFileInfo {
    #[serde(rename = "FileId")]
    pub file_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "DateModified")]
    pub date_modified: String,
    // #[serde(rename = "Date")]
    // pub date: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveGetFileInfoResponse {
    Success(OpendriveGetFileInfo),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpendriveGetFolderInfo {
    #[serde(rename = "FolderID")]
    pub folder_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "DateModified")]
    pub date_modified: String,
    #[serde(rename = "ChildFolders")]
    pub child_folders: Option<usize>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveGetFolderInfoResponse {
    Success(OpendriveGetFolderInfo),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
pub struct OpendriveGetListInfo {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ParentFolderID")]
    pub parent_folder_id: String,
    #[serde(rename = "Folders")]
    pub folders: Vec<OpendriveGetFolderInfo>,
    #[serde(rename = "Files")]
    pub files: Vec<OpendriveGetFileInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveGetListInfoResponse {
    Success(OpendriveGetListInfo),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveSuccessIgnoreResponse {
    Success,
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
pub struct OpendriveCheckIfExists {
    pub result: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveCheckIfExistsResponse {
    Success(OpendriveCheckIfExists),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
pub struct OpendriveOpenFileUploadInfo {
    #[serde(rename = "TempLocation")]
    pub temp_location: String,

    #[serde(rename = "RequireCompression")]
    pub require_compression: bool,

    #[serde(rename = "RequireHash")]
    pub require_hash: bool,

    #[serde(rename = "RequireHashOnly")]
    pub require_hash_only: bool,

    #[serde(rename = "SpeedLimit")]
    pub speed_limit: u64,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveOpenFileUploadResponse {
    Success(OpendriveOpenFileUploadInfo),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Deserialize)]
pub struct OpendriveCloseFileUploadInfo {
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "DateModified")]
    pub date_modified: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveCloseFileUploadResponse {
    Success(OpendriveCloseFileUploadInfo),
    Fail(OpendriveDeserializeFail),
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpendriveCreateFileInfo {
    #[serde(rename = "FileId")]
    pub file_id: String,
    #[serde(rename = "Name")]
    pub file_name: String,
    #[serde(rename = "Size")]
    pub size: String,
    #[serde(rename = "TempLocation")]
    pub temp_location: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OpendriveCreateFileResponse {
    Success(OpendriveCreateFileInfo),
    Fail(OpendriveDeserializeFail),
}
