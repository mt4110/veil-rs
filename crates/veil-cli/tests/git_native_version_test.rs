#[test]
fn linked_libgit2_meets_the_pack_reading_security_floor() {
    let version = git2::Version::get().libgit2_version();
    assert!(
        version >= (1, 9, 5),
        "linked libgit2 is below the patched 1.9.5 floor: {version:?}"
    );
}
