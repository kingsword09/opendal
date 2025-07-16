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
use crate::*;

pub struct OpendriveLister {
    core: Arc<OpendriveCore>,
    path: String,
    recursive: bool,
}

impl OpendriveLister {
    pub fn new(core: Arc<OpendriveCore>, path: String, recursive: bool) -> Self {
        Self {
            core,
            path,
            recursive,
        }
    }
}

impl oio::PageList for OpendriveLister {
    async fn next_page(&self, ctx: &mut oio::PageContext) -> Result<()> {
        // For OpenDrive, we'll get all entries in one go since the API doesn't support pagination
        if ctx.done {
            return Ok(());
        }

        let args = OpList::new().with_recursive(self.recursive);
        let entries = match self.core.list(&self.path, args).await {
            Ok(entries) => entries,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    ctx.done = true;
                    return Ok(());
                }
                return Err(e);
            }
        };

        for entry in entries {
            ctx.entries.push_back(oio::Entry::with(
                entry.path().to_string(),
                entry.metadata().clone(),
            ));
        }

        ctx.done = true;
        Ok(())
    }
}
