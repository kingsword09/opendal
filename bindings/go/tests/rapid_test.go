package tests

import (
	"fmt"
	"testing"

	"github.com/apache/opendal/bindings/go"
)

// TestRapidOperations tests rapid creation/deletion of operators
// This mimics the MoonBit test pattern that causes SIGSEGV
func TestRapidOperations(t *testing.T) {
	const iterations = 100

	for i := 0; i < iterations; i++ {
		// Create operator
		op, err := opendal.NewOperator("memory", opendal.OperatorOptions{})
		if err != nil {
			t.Fatalf("Iteration %d: Failed to create operator: %v", i, err)
		}

		// Write
		path := fmt.Sprintf("test_file_%d.txt", i)
		data := make([]byte, 1024)
		for j := range data {
			data[j] = 'A'
		}

		err = op.Write(path, data)
		if err != nil {
			t.Fatalf("Iteration %d: Write failed: %v", i, err)
		}

		// Read
		_, err = op.Read(path)
		if err != nil {
			t.Fatalf("Iteration %d: Read failed: %v", i, err)
		}

		// Delete - THIS IS WHERE THE CRASH HAPPENS IN MOONBIT
		err = op.Delete(path)
		if err != nil {
			t.Fatalf("Iteration %d: Delete failed: %v", i, err)
		}

		// Free operator
		op.Free()

		if (i+1)%10 == 0 {
			t.Logf("✓ Completed %d iterations successfully", i+1)
		}
	}

	t.Logf("All %d iterations completed successfully", iterations)
}

// BenchmarkRapidOperations runs the test as a benchmark to stress test
func BenchmarkRapidOperations(b *testing.B) {
	for n := 0; n < b.N; n++ {
		op, err := opendal.NewOperator("memory", opendal.OperatorOptions{})
		if err != nil {
			b.Fatalf("Failed to create operator: %v", err)
		}

		path := fmt.Sprintf("bench_file_%d.txt", n)
		data := make([]byte, 1024)

		_ = op.Write(path, data)
		_, _ = op.Read(path)
		_ = op.Delete(path)

		op.Free()
	}
}
