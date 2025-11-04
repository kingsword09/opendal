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

// ============ Public Types ============

///|
pub struct Operator {
  inner : C_Operator
}

///|
pub struct Metadata {
  inner : C_Metadata
}

// ============ Helper Functions ============

///|
fn string_to_c_bytes(s : String) -> Bytes {
  let bytes = @encoding/utf8.encode(s)
  let arr = bytes.to_array()
  // Check if already null-terminated
  if arr.length() > 0 && arr[arr.length() - 1] == b'\x00' {
    return bytes
  }
  // Add null terminator
  let new_arr = Array::new()
  for i = 0; i < arr.length(); i = i + 1 {
    new_arr.push(arr[i])
  }
  new_arr.push(b'\x00')
  Bytes::from_array(new_arr)
}
