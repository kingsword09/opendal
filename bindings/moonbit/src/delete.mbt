// ============ Delete Operation ============

///|
pub fn Operator::delete(self : Operator, path : String) -> Result[Unit, String] {
  let path_bytes = string_to_c_bytes(path)
  let result = moonbit_opendal_operator_delete(self.inner, path_bytes)
  if result == 0 {
    Ok(())
  } else {
    Err("Delete failed with error code: " + result.to_string())
  }
}
