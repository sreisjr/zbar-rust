#[test]
fn version() {
    assert_ne!(String::new(), zbar_rust::ZBarImage::zbar_version());
}
