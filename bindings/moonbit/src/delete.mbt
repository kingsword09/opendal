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

// ============ Delete Operation ============

///|
pub fn Operator::delete(self : Operator, path : String) -> Result[Unit, String] {
  let path_bytes = string_to_c_bytes(path)
  let result = moonbit_operator_delete(self.inner, path_bytes)
  if result == 0 {
    Ok(())
  } else {
    Err("Delete failed with error code: " + result.to_string())
  }
}
