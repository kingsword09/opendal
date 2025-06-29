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

use super::core::OpendriveCore;
use chrono::DateTime;
use chrono::Utc;
use http::StatusCode;
use log::debug;
// use services::onedrive::core::OneDriveCore;
// use services::onedrive::core::OneDriveSigner;
use tokio::sync::Mutex;

use crate::raw::normalize_root;
use crate::raw::parse_datetime_from_from_timestamp;
use crate::raw::Access;
use crate::raw::AccessorInfo;
use crate::raw::HttpBody;
use crate::raw::HttpClient;
use crate::raw::OpCreateDir;
use crate::raw::OpRead;
use crate::raw::OpStat;
use crate::raw::RpCreateDir;
use crate::raw::RpRead;
use crate::raw::RpStat;
use crate::services::opendrive::core::OpendriveSigner;
use crate::services::opendrive::error::parse_error;
use crate::services::opendrive::error::parse_i64_error;
use crate::services::OpendriveConfig;
use crate::Scheme;
use crate::*;

impl Configurator for OpendriveConfig {
    type Builder = OpendriveBuilder;
    fn into_builder(self) -> Self::Builder {
        OpendriveBuilder {
            config: self,
            http_client: None,
        }
    }
}

/// Microsoft [OneDrive](https://onedrive.com) backend support.
#[doc = include_str!("docs.md")]
#[derive(Default)]
pub struct OpendriveBuilder {
    config: OpendriveConfig,
    http_client: Option<HttpClient>,
}

impl Debug for OpendriveBuilder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Backend")
            .field("config", &self.config)
            .finish()
    }
}

impl OpendriveBuilder {
    /// Set root path of Opendrive folder.
    pub fn root(mut self, root: &str) -> Self {
        self.config.root = if root.is_empty() {
            None
        } else {
            Some(root.to_string())
        };

        self
    }

    /// Specify the http client that used by this service.
    ///
    /// # Notes
    ///
    /// This API is part of OpenDAL's Raw API. `HttpClient` could be changed
    /// during minor updates.
    #[deprecated(since = "0.53.0", note = "Use `Operator::update_http_client` instead")]
    #[allow(deprecated)]
    pub fn http_client(mut self, http_client: HttpClient) -> Self {
        self.http_client = Some(http_client);
        self
    }

    /// Set the access token for a time limited access to OpenDrive API.
    ///
    /// OpenDrive API uses a typical OAuth 2.0 flow for authentication and authorization.
    /// You can get an access token from OpenDrive's developer portal.
    ///
    /// # Note
    ///
    /// - An access token is short-lived.
    /// - Use a refresh_token if you want to use OpenDrive API for an extended period of time.
    pub fn access_token(mut self, access_token: &str) -> Self {
        self.config.access_token = Some(access_token.to_string());
        self
    }

    /// Set the refresh token for long term access to OpenDrive API.
    ///
    /// OpenDAL will use a refresh token to maintain a fresh access token automatically.
    ///
    /// # Note
    ///
    /// - A refresh token is available through a OAuth 2.0 flow.
    pub fn refresh_token(mut self, refresh_token: &str) -> Self {
        self.config.refresh_token = Some(refresh_token.to_string());
        self
    }

    /// Set the username required for Opendrive login.
    ///
    /// The username is used to identify your Opendrive account.
    /// This should be the email address or username associated with your account.
    ///
    /// # Note
    ///
    /// - Username must be valid and registered with Opendrive
    /// - Required for accessing your Opendrive account and files
    pub fn username(mut self, username: &str) -> Self {
        self.config.username = Some(username.to_string());
        self
    }

    /// Set the passwd required for Opendrive login.
    ///
    /// This password is used to authenticate with the Opendrive service.
    ///
    /// # Note
    ///
    /// - The password must match the one associated with your Opendrive account
    /// - Required for accessing your Opendrive account and files
    pub fn password(mut self, password: &str) -> Self {
        self.config.password = Some(password.to_string());
        self
    }
}

impl Builder for OpendriveBuilder {
    const SCHEME: Scheme = Scheme::Opendrive;
    type Config = OpendriveConfig;

    fn build(self) -> Result<impl Access> {
        let root = normalize_root(&self.config.root.unwrap_or_default());
        debug!("backend use root {root}");

        let info = AccessorInfo::default();
        info.set_scheme(Scheme::Onedrive)
            .set_root(&root)
            .set_native_capability(Capability {
                read: true,
                // read_with_if_none_match: true,

                // write: true,
                // write_with_if_match: true,
                // // OneDrive supports the file size up to 250GB
                // // Read more at https://support.microsoft.com/en-us/office/restrictions-and-limitations-in-onedrive-and-sharepoint-64883a5d-228e-48f5-b3d2-eb39e07630fa#individualfilesize
                // // However, we can't enable this, otherwise OpenDAL behavior tests will try to test creating huge
                // // file up to this size.
                // // write_total_max_size: Some(250 * 1024 * 1024 * 1024),
                // copy: true,
                // rename: true,

                // stat: true,
                // stat_with_if_none_match: true,

                // delete: true,
                // create_dir: true,

                // list: true,
                // list_with_limit: true,
                shared: true,

                ..Default::default()
            });

        // allow deprecated api here for compatibility
        #[allow(deprecated)]
        if let Some(client) = self.http_client {
            info.update_http_client(|_| client);
        }

        let accessor_info = Arc::new(info);
        let mut signer = OpendriveSigner::new(accessor_info.clone());

        let core = Arc::new(OpendriveCore {
            info: accessor_info,
            root,
            signer: Arc::new(Mutex::new(signer)),
        });

        Ok(OpendriveAccessor { core })
    }
}

#[derive(Clone)]
pub struct OpendriveAccessor {
    pub core: Arc<OpendriveCore>,
}

impl Debug for OpendriveAccessor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpendriveAccessor")
            .field("core", &self.core)
            .finish()
    }
}

impl Access for OpendriveAccessor {
    type Reader = HttpBody;
    // type Writer = oio::OneShotWriter<OneDriveWriter>;
    // type Lister = oio::PageLister<OneDriveLister>;
    // type Deleter = oio::OneShotDeleter<OneDriveDeleter>;
    type Writer = ();
    type Lister = ();
    type Deleter = ();

    fn info(&self) -> Arc<AccessorInfo> {
        self.core.info.clone()
    }

    async fn create_dir(&self, path: &str, _args: OpCreateDir) -> Result<RpCreateDir> {
        _ = self.core.opendrive_create_dir(path).await?;
        Ok(RpCreateDir::default())
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        // Try to get file info first
        match self.core.get_folder_id(path).await {
            Ok(folder_id) => {
                let result = self.core.get_folder_info(&folder_id).await?;

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

                // Parse since time once for both time-based conditions
                let last_modified = parse_datetime_from_from_timestamp(
                    result.date_modified.parse().map_err(parse_i64_error)?,
                )
                .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

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

                let metadata = Metadata::new(EntryMode::DIR)
                    .with_last_modified(last_modified)
                    .with_etag(folder_id);

                return Ok(RpStat::new(metadata));
            }
            Err(err) => {
                // If not found as file, try folder
                if matches!(err.kind(), ErrorKind::NotFound) {
                    let file_id = self.core.get_file_id(path).await?;

                    let result = self.core.get_file_info(&file_id).await?;

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

                    // Parse since time once for both time-based conditions
                    let last_modified = parse_datetime_from_from_timestamp(
                        result.date_modified.parse().map_err(parse_i64_error)?,
                    )
                    .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

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

                    let metadata = Metadata::new(EntryMode::FILE)
                        .with_last_modified(last_modified)
                        .with_version(result.version)
                        .with_etag(file_id)
                        .with_content_length(result.size.parse().map_err(parse_i64_error)?);

                    return Ok(RpStat::new(metadata));
                } else {
                    Err(err)
                }
            }
        }
    }

    // async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Reader)> {

    // }
}
