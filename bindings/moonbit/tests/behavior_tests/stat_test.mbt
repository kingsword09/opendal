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

// Stat and exists operation tests

test "stat basic" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("stat_basic")
  let data = generate_test_data(1024)

  // Write data first
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write failed: \{e}")
  }

  // Stat file
  match op.stat(path) {
    Ok(meta) => {
      if meta.content_length().to_int() != 1024 {
        fail("Content length mismatch: expected 1024, got \{meta.content_length().to_int()}")
      }
      if not(meta.is_file()) {
        fail("Expected file, but is_file() returned false")
      }
      meta.free()
    }
    Err(e) => fail("Stat failed: \{e}")
  }

  // Cleanup
  let _ = op.delete(path)
}

test "stat empty file" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("stat_empty")
  let data = Bytes::new(0)

  // Write empty file
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write failed: \{e}")
  }

  // Stat empty file
  match op.stat(path) {
    Ok(meta) => {
      if meta.content_length().to_int() != 0 {
        fail("Expected empty file, got \{meta.content_length().to_int()} bytes")
      }
      meta.free()
    }
    Err(e) => fail("Stat failed: \{e}")
  }

  // Cleanup
  let _ = op.delete(path)
}

test "exists basic" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("exists_basic")
  let data = generate_test_data(512)

  // Check before write
  match op.exists(path) {
    Ok(exists_before) => {
      if exists_before {
        fail("File should not exist before write")
      }
    }
    Err(e) => fail("Exists check before write failed: \{e}")
  }

  // Write file
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write failed: \{e}")
  }

  // Check after write
  match op.exists(path) {
    Ok(exists_after) => {
      if not(exists_after) {
        fail("File should exist after write")
      }
    }
    Err(e) => fail("Exists check after write failed: \{e}")
  }

  // Cleanup
  let _ = op.delete(path)
}
