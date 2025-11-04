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

fn main {
  println("OpenDAL MoonBit Bindings - Testing FFI")

  // Test operator creation
  let op = Operator::new("memory")
  println("Operator created")

  // Test write
  let data = Bytes::from_array([b'H', b'e', b'l', b'l', b'o'])
  match op.write("test.txt", data) {
    Ok(_) => println("Write successful!")
    Err(e) => println("Write failed: \{e}")
  }

  // Test read
  match op.read("test.txt") {
    Ok(read_data) => println("Read successful! Got \{read_data.length()} bytes")
    Err(e) => println("Read failed: \{e}")
  }

  // Cleanup
  op.free()
  println("Test completed")
}
