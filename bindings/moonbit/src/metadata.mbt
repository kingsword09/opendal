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

// ============ Metadata Implementation ============

///|
pub fn Metadata::content_length(self : Metadata) -> UInt64 {
  moonbit_metadata_content_length(self.inner)
}

///|
pub fn Metadata::is_file(self : Metadata) -> Bool {
  moonbit_metadata_is_file(self.inner)
}

///|
pub fn Metadata::is_dir(self : Metadata) -> Bool {
  moonbit_metadata_is_dir(self.inner)
}

///|
pub fn Metadata::free(self : Metadata) -> Unit {
  moonbit_metadata_free(self.inner)
}
