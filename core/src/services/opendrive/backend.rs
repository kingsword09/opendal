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
use log::debug;
use reqwest::Client;
use reqwest::Proxy;
use tokio::sync::Mutex;

use crate::raw::*;
use crate::services::opendrive::core::constants;
use crate::services::opendrive::core::OpendriveSigner;
use crate::services::opendrive::delete::OpendriveDeleter;
use crate::services::opendrive::error::new_proxy_request_build_error;
use crate::services::opendrive::lister::OpendriveLister;
use crate::services::opendrive::writer::OpendriveWriter;
use crate::services::opendrive::writer::OpendriveWriters;
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

/// [Opendrive](https://www.opendrive.com/) backend support.
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
        // let root = self.config.root.unwrap_or_default();
        // let root = "/".to_string();
        println!("backend use root before {:?}", &self.config.root);
        let root = normalize_root(&self.config.root.unwrap_or_default());
        println!("backend use root {root}");
        debug!("backend use root {root}");

        let username = match self.config.username {
            Some(username) => username,
            None => {
                return Err(Error::new(ErrorKind::ConfigInvalid, "username is empty")
                    .with_operation("Builder::build")
                    .with_context("service", Scheme::Opendrive))
            }
        };

        let password = match self.config.password {
            Some(password) => password,
            None => {
                return Err(Error::new(ErrorKind::ConfigInvalid, "password is empty")
                    .with_operation("Builder::build")
                    .with_context("service", Scheme::Opendrive))
            }
        };

        let info = AccessorInfo::default();
        info.set_scheme(Scheme::Opendrive)
            .set_root(&root)
            .set_native_capability(Capability {
                create_dir: true,
                copy: true,
                rename: true,

                delete: true,
                delete_with_version: true,

                read: true,
                read_with_if_match: true,
                read_with_if_none_match: true,
                read_with_if_modified_since: true,
                read_with_if_unmodified_since: true,
                read_with_version: true,

                stat: true,
                stat_with_if_match: true,
                stat_with_if_none_match: true,
                stat_with_if_modified_since: true,
                stat_with_if_unmodified_since: true,
                stat_with_version: true,

                write: true,
                write_can_append: true,
                write_can_empty: true,
                write_can_multi: true,
                write_with_if_match: true,
                write_with_if_none_match: true,
                write_with_if_not_exists: true,

                list: true,
                list_with_recursive: true,

                shared: true,

                ..Default::default()
            });

        // allow deprecated api here for compatibility
        #[allow(deprecated)]
        if let Some(client) = self.http_client {
            info.update_http_client(|_| client);
        }

        let accessor_info = Arc::new(info);
        // We must use a proxy client to successfully obtain authorization information
        // from OpenDrive's API.
        let auth_http_client = Client::builder()
            .proxy(
                Proxy::http(constants::OPENDRIVE_BASE_URL)
                    .map_err(new_proxy_request_build_error)?,
            )
            .build()
            .map_err(new_proxy_request_build_error)?;
        let signer = OpendriveSigner::new(auth_http_client, &username, &password);

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
    type Reader = Buffer;
    // type Lister = oio::PageLister<OpendriveLister>;
    type Writer = OpendriveWriters;
    type Lister = oio::PageLister<OpendriveLister>;
    type Deleter = oio::OneShotDeleter<OpendriveDeleter>;

    fn info(&self) -> Arc<AccessorInfo> {
        self.core.info.clone()
    }

    async fn create_dir(&self, path: &str, _args: OpCreateDir) -> Result<RpCreateDir> {
        self.core.opendrive_create_dir(path).await?;
        Ok(RpCreateDir::default())
    }

    async fn stat(&self, path: &str, args: OpStat) -> Result<RpStat> {
        let metadata = self.core.stat(path, Some(args)).await?;

        Ok(RpStat::new(metadata))
    }

    async fn read(&self, path: &str, args: OpRead) -> Result<(RpRead, Self::Reader)> {
        let bs = self.core.read(path, args).await?;
        Ok((RpRead::new(), bs))
    }

    async fn rename(&self, from: &str, to: &str, _args: OpRename) -> Result<RpRename> {
        self.core.rename(from, to).await?;

        Ok(RpRename::default())
    }

    async fn copy(&self, from: &str, to: &str, _args: OpCopy) -> Result<RpCopy> {
        self.core.copy(from, to).await?;

        Ok(RpCopy::default())
    }

    async fn delete(&self) -> Result<(RpDelete, Self::Deleter)> {
        Ok((
            RpDelete::default(),
            oio::OneShotDeleter::new(OpendriveDeleter::new(self.core.clone())),
        ))
    }

    async fn write(&self, path: &str, args: OpWrite) -> Result<(RpWrite, Self::Writer)> {
        let writer = OpendriveWriter::new(self.core.clone(), path, args.clone());

        let w = if args.append() {
            OpendriveWriters::Two(oio::AppendWriter::new(writer))
        } else {
            OpendriveWriters::One(oio::OneShotWriter::new(writer))
        };

        Ok((RpWrite::default(), w))
    }

    async fn list(&self, path: &str, args: OpList) -> Result<(RpList, Self::Lister)> {
        let lister = OpendriveLister::new(
            self.core.clone(),
            path.to_string(),
            args.recursive(),
        );
        Ok((RpList::default(), oio::PageLister::new(lister)))
    }
}
