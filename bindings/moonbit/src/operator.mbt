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

// ============ Operator Implementation ============

///|
pub fn Operator::new(scheme : String) -> Operator {
  let scheme_bytes = string_to_c_bytes(scheme)
  let op = moonbit_operator_new(scheme_bytes)
  { inner: op }
}

///|
pub fn Operator::free(self : Operator) -> Unit {
  moonbit_operator_free(self.inner)
}
