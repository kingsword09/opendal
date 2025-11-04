// ============ Write Operation ============

///|
pub fn Operator::write(
  self : Operator,
  path : String,
  data : Bytes,
) -> Result[Unit, String] {
  let path_bytes = string_to_c_bytes(path)
  let data_len = data.length().reinterpret_as_uint()
  let result = moonbit_opendal_operator_write(
    self.inner,
    path_bytes,
    data,
    data_len,
  )
  if result == 0 {
    Ok(())
  } else {
    Err("Write failed with error code: " + result.to_string())
  }
}
