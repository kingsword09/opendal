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

use std::collections::HashMap;
use std::os::raw::{c_char, c_void};
use std::ffi::CString;
use std::str::FromStr;
use std::sync::LazyLock;
use interoptopus::ffi_function;

mod inventory;
pub use inventory::ffi_inventory;

static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
});

/// Internal handle wrapping both async and blocking operators.
struct OperatorWrapper {
    async_op: opendal::Operator,
    blocking_op: opendal::blocking::Operator,
}

/// # Safety
///
/// Not yet.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn blocking_operator_construct(
    scheme: *const c_char,
) -> *mut c_void {
    if scheme.is_null() {
        return std::ptr::null_mut();
    }

/// Start a background read and return an opaque handle.
/// Caller should later call `blocking_operator_read_await` to obtain result.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn blocking_operator_read_start(
    op: *mut c_void,
    path: *const c_char,
) -> *mut c_void {
    if op.is_null() || path.is_null() {
        return std::ptr::null_mut();
    }

    let wrapper = &*(op as *mut OperatorWrapper);
    let async_op = wrapper.async_op.clone();
    let path_str = match std::ffi::CStr::from_ptr(path).to_str() {
        Ok(s) => s.to_owned(),
        Err(_) => return std::ptr::null_mut(),
    };

    let handle = RUNTIME.spawn(async move {
        async_op.read(&path_str).await
    });

    Box::into_raw(Box::new(handle)) as *mut c_void
}

/// Await the background read started by `blocking_operator_read_start`.
/// Consumes the handle and returns a newly allocated C string, which must be
/// freed by calling `opendal_string_free`.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn blocking_operator_read_await(handle: *mut c_void) -> *const c_char {
    if handle.is_null() {
        return std::ptr::null();
    }

    let handle: Box<tokio::task::JoinHandle<Result<Vec<u8>, opendal::Error>>> =
        Box::from_raw(handle as *mut tokio::task::JoinHandle<Result<Vec<u8>, opendal::Error>>);

    let join_res = RUNTIME.block_on(async move { handle.await });
    let bytes = match join_res {
        Ok(Ok(v)) => v,
        _ => return std::ptr::null(),
    };

    let mut buf = bytes;
    buf.push(0);
    CString::from_vec_with_nul(buf).unwrap().into_raw()
}

/// # Safety
///
/// Destroy a previously constructed operator wrapper.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn blocking_operator_destroy(op: *mut c_void) {
    if !op.is_null() {
        drop(Box::from_raw(op as *mut OperatorWrapper));
    }
}

    let scheme = match opendal::Scheme::from_str(std::ffi::CStr::from_ptr(scheme).to_str().unwrap())
    {
        Ok(scheme) => scheme,
        Err(_) => return std::ptr::null_mut(),
    };

    let mut map = HashMap::<String, String>::default();
    map.insert("root".to_string(), "/tmp".to_string());
    let op = match opendal::Operator::via_iter(scheme, map) {
        Ok(op) => op,
        Err(err) => {
            println!("err={err:?}");
            return std::ptr::null_mut();
        }
    };

    let handle = RUNTIME.handle();
    let _enter = handle.enter();
    let blocking_op = match opendal::blocking::Operator::new(op.clone()) {
        Ok(op) => op,
        Err(err) => {
            println!("err={err:?}");
            return std::ptr::null_mut();
        }
    };

    let wrapper = OperatorWrapper {
        async_op: op,
        blocking_op,
    };
    Box::into_raw(Box::new(wrapper)) as *mut c_void
}

/// # Safety
///
/// Not yet.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn blocking_operator_write(
    op: *mut c_void,
    path: *const c_char,
    content: *const c_char,
) {
    if op.is_null() { return; }
    let op = &*(op as *mut OperatorWrapper);
    let path = std::ffi::CStr::from_ptr(path).to_str().unwrap();
    let content = std::ffi::CStr::from_ptr(content).to_str().unwrap();
    op.blocking_op.write(path, content.to_owned()).map(|_| ()).unwrap()
}

/// # Safety
///
/// Not yet.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn blocking_operator_read(
    op: *mut c_void,
    path: *const c_char,
) -> *const c_char {
    if op.is_null() { return std::ptr::null(); }
    let op = &*(op as *mut OperatorWrapper);
    let path = std::ffi::CStr::from_ptr(path).to_str().unwrap();
    let mut res = op.blocking_op.read(path).unwrap().to_vec();
    res.push(0);
    CString::from_vec_with_nul(res)
        .unwrap()
        .into_raw()
}

/// # Safety
///
/// Free a C string returned by this library.
#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn opendal_string_free(s: *mut c_char) {
    if !s.is_null() {
        let _ = CString::from_raw(s);
    }
}

/// A minimal Interoptopus-exported function to drive code generation.
#[ffi_function]
pub fn opendal_ping() -> i32 {
    1
}
