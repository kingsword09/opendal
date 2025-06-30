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

use std::sync::Arc;

use super::core::*;
use crate::raw::*;
use crate::services::opendrive::error::parse_numeric_types_error;
use crate::*;

pub type OpendriveWriters =
    TwoWays<oio::OneShotWriter<OpendriveWriter>, oio::AppendWriter<OpendriveWriter>>;

pub struct OpendriveWriter {
    core: Arc<OpendriveCore>,

    op: OpWrite,
    path: String,
}

impl OpendriveWriter {
    pub fn new(core: Arc<OpendriveCore>, path: &str, op: OpWrite) -> Self {
        OpendriveWriter {
            core,
            path: path.to_string(),
            op,
        }
    }
}

impl oio::OneShotWrite for OpendriveWriter {
    async fn write_once(&self, bs: Buffer) -> Result<Metadata> {
        let path = build_abs_path(&self.core.root, &self.path);

        let metadata = self.core.stat(&path, None).await?;
        if metadata.is_dir() {
            return Err(Error::new(
                ErrorKind::IsADirectory,
                "directory does not support write operations",
            ));
        }

        let file_id = self.core.parse_id_by_metadata(&path, metadata).await?;

        let result = self.core.write_once(&file_id, &path, bs).await?;

        let last_modified = parse_datetime_from_from_timestamp(
            result
                .date_modified
                .parse()
                .map_err(parse_numeric_types_error)?,
        )
        .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

        let metadata = Metadata::new(EntryMode::FILE)
            .with_etag(file_id)
            .with_last_modified(last_modified)
            .with_version(result.version)
            .with_content_length(result.size.parse().map_err(parse_numeric_types_error)?);

        Ok(metadata)
    }
}

impl oio::AppendWrite for OpendriveWriter {
    async fn offset(&self) -> Result<u64> {
        let path = build_abs_path(&self.core.root, &self.path);

        let info = self.core.create_file(&path).await?;

        let offset: u64 = info.size.parse().map_err(parse_numeric_types_error)?;

        Ok(offset)
    }

    async fn append(&self, offset: u64, size: u64, body: Buffer) -> Result<Metadata> {
        let path = build_abs_path(&self.core.root, &self.path);
        let result = self.core.write_append(&path, size, offset, body).await?;

        let last_modified = parse_datetime_from_from_timestamp(
            result
                .date_modified
                .parse()
                .map_err(parse_numeric_types_error)?,
        )
        .map_err(|_| Error::new(ErrorKind::Unsupported, "invalid since format"))?;

        let metadata = Metadata::new(EntryMode::FILE)
            .with_etag(result.file_id)
            .with_last_modified(last_modified)
            .with_version(result.version)
            .with_content_length(result.size.parse().map_err(parse_numeric_types_error)?);

        Ok(metadata)
    }
}
