/*
 * Test for rapid operator creation/deletion to reproduce SIGSEGV issue
 * This test mimics the behavior of MoonBit tests that fail intermittently
 */

#include "../include/opendal.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#define TEST_ITERATIONS 100
#define TEST_DATA_SIZE 1024

int test_rapid_write_delete(int iteration) {
    // Create operator
    opendal_result_operator_new result = opendal_operator_new("memory", NULL);
    if (result.error != NULL) {
        fprintf(stderr, "[Iteration %d] Failed to create operator: %s\n",
                iteration, result.error->message.data);
        opendal_error_free(result.error);
        return -1;
    }
    opendal_operator* op = result.op;

    // Prepare test data
    char path[256];
    snprintf(path, sizeof(path), "test_file_%d.txt", iteration);

    uint8_t* data = (uint8_t*)malloc(TEST_DATA_SIZE);
    memset(data, 'A', TEST_DATA_SIZE);

    opendal_bytes bytes = {
        .data = data,
        .len = TEST_DATA_SIZE,
        .capacity = TEST_DATA_SIZE
    };

    // Write
    opendal_error* write_error = opendal_operator_write(op, path, &bytes);
    if (write_error != NULL) {
        fprintf(stderr, "[Iteration %d] Write failed: %s\n",
                iteration, write_error->message.data);
        opendal_error_free(write_error);
        free(data);
        opendal_operator_free(op);
        return -1;
    }

    // Read to verify
    opendal_result_read read_result = opendal_operator_read(op, path);
    if (read_result.error != NULL) {
        fprintf(stderr, "[Iteration %d] Read failed: %s\n",
                iteration, read_result.error->message.data);
        opendal_error_free(read_result.error);
        free(data);
        opendal_operator_free(op);
        return -1;
    }
    opendal_bytes_free(&read_result.data);

    // Delete - THIS IS WHERE THE CRASH HAPPENS IN MOONBIT
    opendal_error* delete_error = opendal_operator_delete(op, path);
    if (delete_error != NULL) {
        fprintf(stderr, "[Iteration %d] Delete failed: %s\n",
                iteration, delete_error->message.data);
        opendal_error_free(delete_error);
        free(data);
        opendal_operator_free(op);
        return -1;
    }

    // Cleanup
    free(data);
    opendal_operator_free(op);

    return 0;
}

int main() {
    printf("Testing rapid operator operations (%d iterations)...\n", TEST_ITERATIONS);
    printf("This test mimics the MoonBit test pattern that causes SIGSEGV\n\n");

    int success_count = 0;
    int failure_count = 0;

    for (int i = 0; i < TEST_ITERATIONS; i++) {
        if (test_rapid_write_delete(i) == 0) {
            success_count++;
            if ((i + 1) % 10 == 0) {
                printf("✓ Completed %d iterations successfully\n", i + 1);
            }
        } else {
            failure_count++;
            printf("✗ Failed at iteration %d\n", i);
        }

        // No delay - run as fast as possible to trigger the race condition
    }

    printf("\n========================================\n");
    printf("Results: %d successes, %d failures\n", success_count, failure_count);
    printf("========================================\n");

    return (failure_count > 0) ? 1 : 0;
}
