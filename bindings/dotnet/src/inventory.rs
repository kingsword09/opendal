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

use interoptopus::inventory::Inventory;
use interoptopus::function;

/// Interoptopus inventory for generated bindings.
///
/// Register exported functions and types here using Interoptopus attributes
/// (e.g., `#[ffi_function]`, `#[ffi_type]`) and `function!(...)` macros.
pub fn ffi_inventory() -> Inventory {
    Inventory::builder()
        .register(function!(crate::blocking_operator_construct))
        .register(function!(crate::blocking_operator_write))
        .register(function!(crate::blocking_operator_read))
        .register(function!(crate::opendal_string_free))
        .register(function!(crate::opendal_ping))
        .validate()
        .build()
}
