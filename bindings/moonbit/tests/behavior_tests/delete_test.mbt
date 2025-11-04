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

// Delete operation tests

test "delete basic" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("delete_basic")
  let data = generate_test_data(1024)

  // Write data first
  match op.write(path, data) {
    Ok(_) => ()
    Err(e) => fail("Write failed: \{e}")
  }

  // Verify file exists
  match op.exists(path) {
    Ok(exists) => {
      if not(exists) {
        fail("File should exist after write")
      }
    }
    Err(e) => fail("Exists check failed: \{e}")
  }

  // Delete file
  match op.delete(path) {
    Ok(_) => ()
    Err(e) => fail("Delete failed: \{e}")
  }

  // Verify file is deleted
  match op.exists(path) {
    Ok(exists_after) => {
      if exists_after {
        fail("File should not exist after delete")
      }
    }
    Err(e) => fail("Exists check after delete failed: \{e}")
  }
}

test "delete not exist" {
  let op = @opendal.Operator::new("memory")
  defer op.free()

  let path = generate_test_path("delete_not_exist")

  // Delete non-existent file (should succeed or be idempotent)
  match op.delete(path) {
    Ok(_) => ()  // Most backends treat delete of non-existent file as success
    Err(e) => fail("Delete of non-existent file failed: \{e}")
  }
}
