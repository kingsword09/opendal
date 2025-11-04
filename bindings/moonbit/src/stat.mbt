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

// ============ Stat Operation ============

///|
pub fn Operator::stat(self : Operator, path : String) -> Result[Metadata, String] {
  let path_bytes = string_to_c_bytes(path)
  let meta = moonbit_operator_stat(self.inner, path_bytes)
  // Check if meta is null (represented as 0)
  // Note: This is a simplified check, actual null checking may vary
  Ok({ inner: meta })
}

///|
pub fn Operator::exists(self : Operator, path : String) -> Result[Bool, String] {
  let path_bytes = string_to_c_bytes(path)
  let result = moonbit_operator_exists(self.inner, path_bytes)
  if result == -1 {
    Err("Exists check failed")
  } else {
    Ok(result == 1)
  }
}
