// ============ Public Types ============

///|
pub struct Operator {
  inner : C_Operator
}

///|
pub struct Metadata {
  inner : C_Metadata
}

// ============ Helper Functions ============

///|
fn string_to_c_bytes(s : String) -> Bytes {
  let bytes = @encoding/utf8.encode(s)
  let arr = bytes.to_array()
  // Check if already null-terminated
  if arr.length() > 0 && arr[arr.length() - 1] == b'\x00' {
    return bytes
  }
  // Add null terminator
  let new_arr = Array::new()
  for i = 0; i < arr.length(); i = i + 1 {
    new_arr.push(arr[i])
  }
  new_arr.push(b'\x00')
  Bytes::from_array(new_arr)
}
