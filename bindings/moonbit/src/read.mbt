// ============ Read Operation ============

///|
pub fn Operator::read(self : Operator, path : String) -> Result[Bytes, String] {
  let path_bytes = string_to_c_bytes(path)

  // First, get the file size
  let size_ref : Ref[UInt] = { val: 0 }
  let size_result = moonbit_opendal_operator_get_size(
    self.inner,
    path_bytes,
    size_ref,
  )
  if size_result != 0 {
    return Err(
      "Failed to get file size, error code: " + size_result.to_string(),
    )
  }
  let file_size = size_ref.val
  if file_size == 0 {
    return Ok(Bytes::new(0))
  }

  // Create a buffer to hold the data
  let buffer_arr = Array::make(file_size.reinterpret_as_int(), b'\x00')
  let buffer = Bytes::from_array(buffer_arr)

  // Read data into buffer
  let bytes_read = moonbit_opendal_operator_read(
    self.inner,
    path_bytes,
    buffer,
    file_size,
  )
  if bytes_read < 0 {
    Err("Read failed")
  } else if bytes_read == 0 {
    Ok(Bytes::new(0))
  } else {
    // Return the buffer (or a slice of it if needed)
    Ok(buffer)
  }
}
