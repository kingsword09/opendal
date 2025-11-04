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

// Read operation tests

test "read basic" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("read_basic")
  let data = generate_test_data(1024)

  // Write data first
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write failed: \{e}")
  }

  // Read data
  match op.read(path) {
    Ok(read_data) => {
      if read_data.length() != data.length() {
        fail("Length mismatch: expected \{data.length()}, got \{read_data.length()}")
      }
    }
    Err(e) => fail("Read failed: \{e}")
  }

  // Cleanup
  let _ = op.delete(path)
}

test "read empty" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("read_empty")
  let data = Bytes::new(0)

  // Write empty data
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write failed: \{e}")
  }

  // Read empty data
  match op.read(path) {
    Ok(read_data) => {
      if read_data.length() != 0 {
        fail("Expected empty file, got \{read_data.length()} bytes")
      }
    }
    Err(e) => fail("Read failed: \{e}")
  }

  // Cleanup
  let _ = op.delete(path)
}

test "read not exist" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("read_not_exist")

  // Try to read non-existent file - should fail
  match op.read(path) {
    Ok(_) => fail("Expected read to fail for non-existent file")
    Err(_) => ()  // Expected
  }
}

test "read large" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("read_large")
  let data = generate_test_data(5 * 1024 * 1024)  // 5MB

  // Write large data
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write large data failed: \{e}")
  }

  // Read large data
  match op.read(path) {
    Ok(read_data) => {
      if read_data.length() != data.length() {
        fail("Large file length mismatch: expected \{data.length()}, got \{read_data.length()}")
      }
    }
    Err(e) => fail("Read large data failed: \{e}")
  }

  // Cleanup
  let _ = op.delete(path)
}
