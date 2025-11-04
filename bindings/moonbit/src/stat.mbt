// ============ Stat and Exists Operations ============

///|
pub fn Operator::exists(self : Operator, path : String) -> Result[Bool, String] {
  let path_bytes = string_to_c_bytes(path)
  let result = moonbit_opendal_operator_exists(self.inner, path_bytes)
  if result == 1 {
    Ok(true)
  } else if result == 0 {
    Ok(false)
  } else {
    Err("Exists check failed with error code: " + result.to_string())
  }
}

///|
pub fn Operator::stat(
  self : Operator,
  path : String,
) -> Result[Metadata, String] {
  let path_bytes = string_to_c_bytes(path)
  let meta = moonbit_opendal_operator_stat(self.inner, path_bytes)
  // TODO: Check if meta is NULL
  Ok({ inner: meta })
}
