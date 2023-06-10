use crate::WolframVersion;

#[test]
fn test_wolfram_version_ordering() {
    let v13_2_0 = WolframVersion::new(13, 2, 0);
    let v13_2_1 = WolframVersion::new(13, 2, 1);
    let v13_3_0 = WolframVersion::new(13, 3, 0);

    assert!(v13_2_0 == v13_2_0);
    assert!(v13_2_0 <= v13_2_0);

    assert!(v13_2_0 != v13_2_1);
    assert!(v13_2_0 <= v13_2_1);

    assert!(v13_3_0 > v13_2_0);
    assert!(v13_3_0 > v13_2_1);
}
