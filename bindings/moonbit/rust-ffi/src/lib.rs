/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

//! MoonBit-specific OpenDAL FFI bindings
//!
//! This crate provides a MoonBit-specific FFI layer for OpenDAL that uses
//! a single-threaded Tokio runtime to avoid TLS issues with MoonBit's test framework.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::str::FromStr;
use std::sync::LazyLock;

use opendal as core;

/// Global Tokio runtime for blocking operations
/// Using multi_thread runtime with a single worker thread for stability
static RUNTIME: LazyLock<tokio::runtime::Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
});

/// Opaque operator handle for MoonBit
#[repr(C)]
pub struct MoonbitOperator {
    inner: *mut core::blocking::Operator,
}

/// Create a new operator
///
/// # Safety
/// - `scheme` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_new(scheme: *const c_char) -> *mut MoonbitOperator {
    if scheme.is_null() {
        return std::ptr::null_mut();
    }

    let scheme_str = match CStr::from_ptr(scheme).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let scheme_enum = match core::Scheme::from_str(scheme_str) {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    // Create async operator first
    let async_op = match core::Operator::via_iter(scheme_enum, std::iter::empty()) {
        Ok(op) => op,
        Err(_) => return std::ptr::null_mut(),
    };

    // Enter the runtime context and create blocking operator
    let _guard = RUNTIME.enter();
    let blocking_op = match core::blocking::Operator::new(async_op) {
        Ok(op) => op,
        Err(_) => return std::ptr::null_mut(),
    };

    let boxed = Box::new(MoonbitOperator {
        inner: Box::into_raw(Box::new(blocking_op)),
    });
    Box::into_raw(boxed)
}

/// Free an operator
///
/// # Safety
/// - `op` must be a valid pointer returned by `moonbit_operator_new`
/// - `op` must not be used after this call
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_free(op: *mut MoonbitOperator) {
    if !op.is_null() {
        let op_box = Box::from_raw(op);
        if !op_box.inner.is_null() {
            drop(Box::from_raw(op_box.inner));
        }
    }
}

/// Write data to a path
///
/// # Safety
/// - `op` must be a valid operator pointer
/// - `path` must be a valid null-terminated C string
/// - `data` must point to valid memory of at least `data_len` bytes
///
/// # Returns
/// - 0 on success
/// - -1 on error
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_write(
    op: *const MoonbitOperator,
    path: *const c_char,
    data: *const u8,
    data_len: usize,
) -> i32 {
    if op.is_null() || path.is_null() || data.is_null() {
        return -1;
    }

    let operator = &*(*op).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let data_slice = slice::from_raw_parts(data, data_len);

    match operator.write(path_str, data_slice) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Read data from a path
///
/// # Safety
/// - `op` must be a valid operator pointer
/// - `path` must be a valid null-terminated C string
/// - `buffer` must point to valid memory of at least `buffer_size` bytes
///
/// # Returns
/// - Number of bytes read on success (>= 0)
/// - -1 on error
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_read(
    op: *const MoonbitOperator,
    path: *const c_char,
    buffer: *mut u8,
    buffer_size: usize,
) -> isize {
    if op.is_null() || path.is_null() || buffer.is_null() {
        return -1;
    }

    let operator = &*(*op).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match operator.read(path_str) {
        Ok(data) => {
            let bytes = data.to_bytes();
            let copy_len = bytes.len().min(buffer_size);
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, copy_len);
            copy_len as isize
        }
        Err(_) => -1,
    }
}

/// Get the size of a file
///
/// # Safety
/// - `op` must be a valid operator pointer
/// - `path` must be a valid null-terminated C string
/// - `out_size` must point to valid memory
///
/// # Returns
/// - 0 on success
/// - -1 on error
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_get_size(
    op: *const MoonbitOperator,
    path: *const c_char,
    out_size: *mut usize,
) -> i32 {
    if op.is_null() || path.is_null() || out_size.is_null() {
        return -1;
    }

    let operator = &*(*op).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match operator.stat(path_str) {
        Ok(metadata) => {
            *out_size = metadata.content_length() as usize;
            0
        }
        Err(_) => -1,
    }
}

/// Delete a file
///
/// # Safety
/// - `op` must be a valid operator pointer
/// - `path` must be a valid null-terminated C string
///
/// # Returns
/// - 0 on success
/// - -1 on error
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_delete(
    op: *const MoonbitOperator,
    path: *const c_char,
) -> i32 {
    if op.is_null() || path.is_null() {
        return -1;
    }

    let operator = &*(*op).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match operator.delete(path_str) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// Check if a path exists
///
/// # Safety
/// - `op` must be a valid operator pointer
/// - `path` must be a valid null-terminated C string
///
/// # Returns
/// - 1 if exists
/// - 0 if not exists
/// - -1 on error
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_exists(
    op: *const MoonbitOperator,
    path: *const c_char,
) -> i32 {
    if op.is_null() || path.is_null() {
        return -1;
    }

    let operator = &*(*op).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match operator.exists(path_str) {
        Ok(exists) => if exists { 1 } else { 0 },
        Err(_) => -1,
    }
}

/// Get metadata for a path
///
/// Returns an opaque pointer to metadata, or null on error
/// Caller must free the metadata using moonbit_metadata_free
///
/// # Safety
/// - `op` must be a valid operator pointer
/// - `path` must be a valid null-terminated C string
#[no_mangle]
pub unsafe extern "C" fn moonbit_operator_stat(
    op: *const MoonbitOperator,
    path: *const c_char,
) -> *mut core::Metadata {
    if op.is_null() || path.is_null() {
        return std::ptr::null_mut();
    }

    let operator = &*(*op).inner;

    let path_str = match CStr::from_ptr(path).to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    match operator.stat(path_str) {
        Ok(metadata) => Box::into_raw(Box::new(metadata)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free metadata
///
/// # Safety
/// - `meta` must be a valid pointer returned by `moonbit_operator_stat`
#[no_mangle]
pub unsafe extern "C" fn moonbit_metadata_free(meta: *mut core::Metadata) {
    if !meta.is_null() {
        drop(Box::from_raw(meta));
    }
}

/// Get content length from metadata
///
/// # Safety
/// - `meta` must be a valid metadata pointer
#[no_mangle]
pub unsafe extern "C" fn moonbit_metadata_content_length(meta: *const core::Metadata) -> u64 {
    if meta.is_null() {
        return 0;
    }
    (*meta).content_length()
}

/// Check if metadata represents a file
///
/// # Safety
/// - `meta` must be a valid metadata pointer
#[no_mangle]
pub unsafe extern "C" fn moonbit_metadata_is_file(meta: *const core::Metadata) -> i32 {
    if meta.is_null() {
        return 0;
    }
    if (*meta).is_file() { 1 } else { 0 }
}

/// Check if metadata represents a directory
///
/// # Safety
/// - `meta` must be a valid metadata pointer
#[no_mangle]
pub unsafe extern "C" fn moonbit_metadata_is_dir(meta: *const core::Metadata) -> i32 {
    if meta.is_null() {
        return 0;
    }
    if (*meta).is_dir() { 1 } else { 0 }
}
