// ============ Operator Implementation ============

///|
pub fn Operator::new(scheme : String) -> Operator {
  let scheme_bytes = string_to_c_bytes(scheme)
  let op = moonbit_opendal_operator_new(scheme_bytes)
  { inner: op }
}

///|
pub fn Operator::free(self : Operator) -> Unit {
  opendal_operator_free(self.inner)
}
