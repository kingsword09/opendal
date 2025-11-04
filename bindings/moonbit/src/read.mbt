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

// ============ Read Operation ============

///|
pub fn Operator::read(self : Operator, path : String) -> Result[Bytes, String] {
  let path_bytes = string_to_c_bytes(path)

  // First, get the file size
  let size_ref : Ref[UInt] = { val: 0 }
  let size_result = moonbit_operator_get_size(
    self.inner,
    path_bytes,
    size_ref,
  )
  if size_result != 0 {
    return Err(
      "Failed to get file size, error code: " + size_result.to_string(),
    )
  }
  let file_size = size_ref.val
  if file_size == 0 {
    return Ok(Bytes::new(0))
  }

  // Create a buffer to hold the data
  let buffer_arr = Array::make(file_size.reinterpret_as_int(), b'\x00')
  let buffer = Bytes::from_array(buffer_arr)

  // Read data into buffer
  let bytes_read = moonbit_operator_read(
    self.inner,
    path_bytes,
    buffer,
    file_size,
  )
  if bytes_read < 0 {
    Err("Read failed")
  } else if bytes_read == 0 {
    Ok(Bytes::new(0))
  } else {
    // Return the buffer
    Ok(buffer)
  }
}
