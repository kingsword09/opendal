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

// ============ FFI Type Declarations ============
// Opaque types for Rust pointers

///|
type C_Operator

///|
type C_Metadata

// ============ FFI Function Declarations ============

// ============ Operator Lifecycle ============

///|
extern "C" fn moonbit_operator_new(scheme : Bytes) -> C_Operator = "moonbit_operator_new"

///|
extern "C" fn moonbit_operator_free(ptr : C_Operator) -> Unit = "moonbit_operator_free"

// ============ Write Operation ============

///|
extern "C" fn moonbit_operator_write(
  op : C_Operator,
  path : Bytes,
  data : Bytes,
  len : UInt,
) -> Int = "moonbit_operator_write"

// ============ Read Operation ============

///|
extern "C" fn moonbit_operator_read(
  op : C_Operator,
  path : Bytes,
  buffer : Bytes,
  buffer_size : UInt,
) -> Int = "moonbit_operator_read"

///|
extern "C" fn moonbit_operator_get_size(
  op : C_Operator,
  path : Bytes,
  out_size : Ref[UInt],
) -> Int = "moonbit_operator_get_size"

// ============ Delete Operation ============

///|
extern "C" fn moonbit_operator_delete(
  op : C_Operator,
  path : Bytes,
) -> Int = "moonbit_operator_delete"

// ============ Exists Operation ============

///|
extern "C" fn moonbit_operator_exists(
  op : C_Operator,
  path : Bytes,
) -> Int = "moonbit_operator_exists"

// ============ Stat Operation ============

///|
extern "C" fn moonbit_operator_stat(
  op : C_Operator,
  path : Bytes,
) -> C_Metadata = "moonbit_operator_stat"

// ============ Metadata Operations ============

///|
extern "C" fn moonbit_metadata_free(meta : C_Metadata) -> Unit = "moonbit_metadata_free"

///|
extern "C" fn moonbit_metadata_content_length(meta : C_Metadata) -> UInt64 = "moonbit_metadata_content_length"

///|
extern "C" fn moonbit_metadata_is_file(meta : C_Metadata) -> Bool = "moonbit_metadata_is_file"

///|
extern "C" fn moonbit_metadata_is_dir(meta : C_Metadata) -> Bool = "moonbit_metadata_is_dir"
