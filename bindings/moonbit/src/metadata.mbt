// ============ Metadata Implementation ============

///|
pub fn Metadata::content_length(self : Metadata) -> UInt64 {
  opendal_metadata_content_length(self.inner)
}

///|
pub fn Metadata::is_file(self : Metadata) -> Bool {
  opendal_metadata_is_file(self.inner)
}

///|
pub fn Metadata::is_dir(self : Metadata) -> Bool {
  opendal_metadata_is_dir(self.inner)
}

///|
pub fn Metadata::free(self : Metadata) -> Unit {
  opendal_metadata_free(self.inner)
}
