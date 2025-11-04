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

#include "../../c/include/opendal.h"
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>

/*
 * MoonBit FFI Helper Functions
 *
 * These functions simplify the interface between MoonBit and OpenDAL C API
 * by handling complex struct returns and pointer checks.
 *
 * IMPORTANT: This wrapper adds thread synchronization to work around
 * Tokio runtime TLS issues when used with MoonBit's test framework.
 */

// Global mutex to serialize access to OpenDAL operations
// This prevents Tokio runtime TLS corruption in MoonBit test environment
static pthread_mutex_t opendal_mutex = PTHREAD_MUTEX_INITIALIZER;

// Flag to track if runtime has been warmed up
static int runtime_warmed_up = 0;

// Warm up the Tokio runtime by creating and destroying a dummy operator
// This helps stabilize TLS initialization
static void warmup_runtime(void) {
    if (runtime_warmed_up) {
        return;
    }

    // Create and destroy multiple dummy operators to fully initialize runtime
    for (int i = 0; i < 3; i++) {
        opendal_result_operator_new result = opendal_operator_new("memory", NULL);
        if (result.error == NULL && result.op != NULL) {
            // Small delay to let runtime stabilize
            usleep(10000);  // 10ms
            opendal_operator_free(result.op);
            usleep(5000);   // 5ms
        } else if (result.error != NULL) {
            opendal_error_free(result.error);
        }
    }

    runtime_warmed_up = 1;
}

// ============ Error Handling Helpers ============

// Check if error pointer is NULL (success)
int moonbit_opendal_error_is_null(const opendal_error* err) {
    return err == NULL ? 1 : 0;
}

// Get error code from error pointer
int moonbit_opendal_error_get_code(const opendal_error* err) {
    if (err == NULL) return 0;
    return (int)err->code;
}

// Get error message length
size_t moonbit_opendal_error_get_message_len(const opendal_error* err) {
    if (err == NULL) return 0;
    return err->message.len;
}

// Copy error message to buffer
void moonbit_opendal_error_copy_message(const opendal_error* err, uint8_t* buf, size_t buf_len) {
    if (err == NULL || buf == NULL) return;
    size_t copy_len = err->message.len < buf_len ? err->message.len : buf_len;
    memcpy(buf, err->message.data, copy_len);
}

// ============ Operator New Helpers ============

// Create operator, returns operator or NULL on error
opendal_operator* moonbit_opendal_operator_new(const char* scheme) {
    pthread_mutex_lock(&opendal_mutex);

    // Warm up runtime on first call
    warmup_runtime();

    opendal_result_operator_new result = opendal_operator_new(scheme, NULL);

    if (result.error != NULL) {
        opendal_error_free(result.error);
        pthread_mutex_unlock(&opendal_mutex);
        return NULL;
    }

    pthread_mutex_unlock(&opendal_mutex);
    return result.op;
}

// ============ Write Operation Helpers ============

// Write data, returns 0 on success, error code on failure
int moonbit_opendal_operator_write(
    const opendal_operator* op,
    const char* path,
    const uint8_t* data,
    size_t len
) {
    pthread_mutex_lock(&opendal_mutex);

    opendal_bytes bytes = {
        .data = (uint8_t*)data,
        .len = len,
        .capacity = len
    };

    opendal_error* error = opendal_operator_write(op, path, &bytes);

    int result;
    if (error == NULL) {
        result = 0;  // Success
    } else {
        result = (int)error->code;
        opendal_error_free(error);
    }

    pthread_mutex_unlock(&opendal_mutex);
    return result;
}

// ============ Read Operation Helpers ============

// Read data into provided buffer
// Returns the number of bytes read, or -1 on error
// buffer must be large enough to hold the data
int moonbit_opendal_operator_read(
    const opendal_operator* op,
    const char* path,
    uint8_t* buffer,
    size_t buffer_size
) {
    pthread_mutex_lock(&opendal_mutex);

    opendal_result_read result = opendal_operator_read(op, path);

    if (result.error != NULL) {
        opendal_error_free(result.error);
        pthread_mutex_unlock(&opendal_mutex);
        return -1;
    }

    size_t copy_len = result.data.len < buffer_size ? result.data.len : buffer_size;
    memcpy(buffer, result.data.data, copy_len);

    // Free the opendal bytes
    opendal_bytes_free(&result.data);

    pthread_mutex_unlock(&opendal_mutex);
    return (int)copy_len;
}

// Get the size of a file without reading it
int moonbit_opendal_operator_get_size(
    const opendal_operator* op,
    const char* path,
    size_t* out_size
) {
    pthread_mutex_lock(&opendal_mutex);

    opendal_result_stat result = opendal_operator_stat(op, path);

    if (result.error != NULL) {
        int code = (int)result.error->code;
        opendal_error_free(result.error);
        pthread_mutex_unlock(&opendal_mutex);
        return code;
    }

    *out_size = opendal_metadata_content_length(result.meta);
    opendal_metadata_free(result.meta);

    pthread_mutex_unlock(&opendal_mutex);
    return 0;
}

// ============ Delete Operation Helpers ============

// Delete file, returns 0 on success, error code on failure
int moonbit_opendal_operator_delete(
    const opendal_operator* op,
    const char* path
) {
    pthread_mutex_lock(&opendal_mutex);

    // Copy the path string to ensure it remains valid during async operations
    // This prevents potential use-after-free if MoonBit's GC collects the original
    char* path_copy = strdup(path);
    if (path_copy == NULL) {
        pthread_mutex_unlock(&opendal_mutex);
        return -1;  // Memory allocation failed
    }

    opendal_error* error = opendal_operator_delete(op, path_copy);

    free(path_copy);

    int result;
    if (error == NULL) {
        result = 0;  // Success
    } else {
        result = (int)error->code;
        opendal_error_free(error);
    }

    pthread_mutex_unlock(&opendal_mutex);
    return result;
}

// ============ Stat Operation Helpers ============

// Get file metadata, returns metadata or NULL on error
opendal_metadata* moonbit_opendal_operator_stat(
    const opendal_operator* op,
    const char* path
) {
    pthread_mutex_lock(&opendal_mutex);

    opendal_result_stat result = opendal_operator_stat(op, path);

    if (result.error != NULL) {
        opendal_error_free(result.error);
        pthread_mutex_unlock(&opendal_mutex);
        return NULL;
    }

    pthread_mutex_unlock(&opendal_mutex);
    return result.meta;
}

// ============ Exists Operation Helpers ============

// Check if path exists, returns 1 if exists, 0 if not, -1 on error
int moonbit_opendal_operator_exists(
    const opendal_operator* op,
    const char* path
) {
    pthread_mutex_lock(&opendal_mutex);

    opendal_result_exists result = opendal_operator_exists(op, path);

    if (result.error != NULL) {
        opendal_error_free(result.error);
        pthread_mutex_unlock(&opendal_mutex);
        return -1;  // Error
    }

    int exists = result.exists ? 1 : 0;
    pthread_mutex_unlock(&opendal_mutex);
    return exists;
}
