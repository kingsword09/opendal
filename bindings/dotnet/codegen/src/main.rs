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

use std::path::PathBuf;

use interoptopus::backend::NamespaceMappings;
use interoptopus::Error;
use interoptopus::inventory::Bindings;
use interoptopus_backend_csharp::Interop;

fn main() -> Result<(), Error> {
    // Output to the C# project Generated folder.
    let out_dir: PathBuf = PathBuf::from("../DotOpenDAL/generated");
    std::fs::create_dir_all(&out_dir).expect("create generated dir");

    let inventory = opendal_dotnet::ffi_inventory();

    let interop = Interop::builder()
        // Native library name without platform-specific prefixes/suffixes.
        .dll_name("opendal_dotnet")
        // Namespace for generated C# types and interop surface.
        .namespace_mappings(NamespaceMappings::new("Apache.OpenDAL"))
        .inventory(inventory)
        .build()
        .expect("build interop config");

    interop.write_file(out_dir.join("Interop.cs"))?;

    Ok(())
}
