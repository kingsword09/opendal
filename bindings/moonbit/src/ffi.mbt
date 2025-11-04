// ============ FFI Type Declarations ============
// Opaque types for C pointers

///|
type C_Operator

///|
type C_Metadata

// ============ FFI Function Declarations ============
// Following MoonBit official async library patterns:
// - Use #borrow for read-only parameters
// - Simple return types (Int for error codes)
// - Clear function naming with project prefix

// ============ Operator Lifecycle ============

///|
#borrow(scheme)
extern "C" fn moonbit_opendal_operator_new(scheme : Bytes) -> C_Operator = "moonbit_opendal_operator_new"

///|
#borrow(ptr)
extern "C" fn opendal_operator_free(ptr : C_Operator) -> Unit = "opendal_operator_free"

// ============ Write Operation ============

///|
#borrow(op, path, data)
extern "C" fn moonbit_opendal_operator_write(
  op : C_Operator,
  path : Bytes,
  data : Bytes,
  len : UInt,
) -> Int = "moonbit_opendal_operator_write"

// ============ Read Operation ============

///|
#borrow(op, path, buffer)
extern "C" fn moonbit_opendal_operator_read(
  op : C_Operator,
  path : Bytes,
  buffer : Bytes,
  buffer_size : UInt,
) -> Int = "moonbit_opendal_operator_read"

///|
#borrow(op, path, out_size)
extern "C" fn moonbit_opendal_operator_get_size(
  op : C_Operator,
  path : Bytes,
  out_size : Ref[UInt],
) -> Int = "moonbit_opendal_operator_get_size"

// ============ Delete Operation ============

///|
#borrow(op, path)
extern "C" fn moonbit_opendal_operator_delete(
  op : C_Operator,
  path : Bytes,
) -> Int = "moonbit_opendal_operator_delete"

// ============ Exists Operation ============

///|
#borrow(op, path)
extern "C" fn moonbit_opendal_operator_exists(
  op : C_Operator,
  path : Bytes,
) -> Int = "moonbit_opendal_operator_exists"

// ============ Stat Operation ============

///|
#borrow(op, path)
extern "C" fn moonbit_opendal_operator_stat(
  op : C_Operator,
  path : Bytes,
) -> C_Metadata = "moonbit_opendal_operator_stat"

// ============ Metadata Operations ============

///|
#borrow(meta)
extern "C" fn opendal_metadata_free(meta : C_Metadata) -> Unit = "opendal_metadata_free"

///|
#borrow(meta)
extern "C" fn opendal_metadata_content_length(meta : C_Metadata) -> UInt64 = "opendal_metadata_content_length"

///|
#borrow(meta)
extern "C" fn opendal_metadata_is_file(meta : C_Metadata) -> Bool = "opendal_metadata_is_file"

///|
#borrow(meta)
extern "C" fn opendal_metadata_is_dir(meta : C_Metadata) -> Bool = "opendal_metadata_is_dir"
