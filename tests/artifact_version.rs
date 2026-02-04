//! Directly lifted from Maven's test cases.

use mvn_version::ArtifactVersion;
use std::cmp::Ordering;

fn assert_versions_equal(s: &str, t: &str) {
    let c1 = ArtifactVersion::new(s);
    let c2 = ArtifactVersion::new(t);
    assert_eq!(c1.cmp(&c2), Ordering::Equal, "{:?} === {:?}", c1, c2);
    assert_eq!(c2.cmp(&c1), Ordering::Equal, "{:?} === {:?}", c2, c1);
    assert_eq!(c1, c2, "{:?} == {:?}", c1, c2);
    assert_eq!(c2, c1, "{:?} == {:?}", c2, c1);
}

fn assert_versions_ordered(s: &str, t: &str) {
    let c1 = ArtifactVersion::new(s);
    let c2 = ArtifactVersion::new(t);
    assert_eq!(c1.cmp(&c2), Ordering::Less, "{:?} < {:?}", c1, c2);
    assert_eq!(c2.cmp(&c1), Ordering::Greater, "{:?} > {:?}", c2, c1);
    assert_ne!(c1, c2, "{:?} != {:?}", c1, c2);
    assert_ne!(c2, c1, "{:?} != {:?}", c2, c1);
}

fn check_parsing(
    s: &str,
    major: u32,
    minor: u32,
    incremental: u32,
    build: u32,
    qualifier: Option<&str>,
) {
    let version = ArtifactVersion::new(s);
    assert_eq!(version.major(), major, "{:?}", version);
    assert_eq!(version.minor(), minor, "{:?}", version);
    assert_eq!(version.incremental(), incremental, "{:?}", version);
    assert_eq!(version.build(), build, "{:?}", version);
    assert_eq!(
        version.qualifier(),
        &qualifier.map(|s| s.to_string()),
        "{:?}",
        version
    );
}

#[test]
fn test_parsing() {
    check_parsing("1", 1, 0, 0, 0, None);
    check_parsing("1.2", 1, 2, 0, 0, None);
    check_parsing("1.2.3", 1, 2, 3, 0, None);
    check_parsing("1.2.3-1", 1, 2, 3, 1, None);
    check_parsing("1.2.3-alpha-1", 1, 2, 3, 0, Some("alpha-1"));
    check_parsing("1.2-alpha-1", 1, 2, 0, 0, Some("alpha-1"));
    check_parsing(
        "1.2-alpha-1-20050205.060708-1",
        1,
        2,
        0,
        0,
        Some("alpha-1-20050205.060708-1"),
    );
    check_parsing("RELEASE", 0, 0, 0, 0, Some("RELEASE"));
    check_parsing("2.0-1", 2, 0, 0, 1, None);

    check_parsing("02", 0, 0, 0, 0, Some("02"));
    check_parsing("0.09", 0, 0, 0, 0, Some("0.09"));
    check_parsing("0.2.09", 0, 0, 0, 0, Some("0.2.09"));
    check_parsing("2.0-01", 2, 0, 0, 0, Some("01"));

    check_parsing("1.0.1b", 0, 0, 0, 0, Some("1.0.1b"));
    check_parsing("1.0M2", 0, 0, 0, 0, Some("1.0M2"));
    check_parsing("1.0RC2", 0, 0, 0, 0, Some("1.0RC2"));
    check_parsing("1.1.2.beta1", 1, 1, 2, 0, Some("beta1"));
    check_parsing("1.7.3.beta1", 1, 7, 3, 0, Some("beta1"));
    check_parsing("1.7.3.0", 0, 0, 0, 0, Some("1.7.3.0"));
    check_parsing("1.7.3.0-1", 0, 0, 0, 0, Some("1.7.3.0-1"));
    check_parsing("PATCH-1193602", 0, 0, 0, 0, Some("PATCH-1193602"));
    check_parsing(
        "5.0.0alpha-2006020117",
        0,
        0,
        0,
        0,
        Some("5.0.0alpha-2006020117"),
    );
    check_parsing("1.0.0.-SNAPSHOT", 0, 0, 0, 0, Some("1.0.0.-SNAPSHOT"));
    check_parsing("1..0-SNAPSHOT", 0, 0, 0, 0, Some("1..0-SNAPSHOT"));
    check_parsing("1.0.-SNAPSHOT", 0, 0, 0, 0, Some("1.0.-SNAPSHOT"));
    check_parsing(".1.0-SNAPSHOT", 0, 0, 0, 0, Some(".1.0-SNAPSHOT"));

    check_parsing("1.2.3.200705301630", 0, 0, 0, 0, Some("1.2.3.200705301630"));
    check_parsing("1.2.3-200705301630", 1, 2, 3, 0, Some("200705301630"));
}

#[test]
fn test_version_order() {
    assert_versions_equal("1", "1");
    assert_versions_ordered("1", "2");
    assert_versions_ordered("1.5", "2");
    assert_versions_ordered("1", "2.5");
    assert_versions_equal("1", "1.0");
    assert_versions_equal("1", "1.0.0");
    assert_versions_ordered("1.0", "1.1");
    assert_versions_ordered("1.1", "1.2");
    assert_versions_ordered("1.0.0", "1.1");
    assert_versions_ordered("1.1", "1.2.0");

    assert_versions_ordered("1.1.2.alpha1", "1.1.2");
    assert_versions_ordered("1.1.2.alpha1", "1.1.2.beta1");
    assert_versions_ordered("1.1.2.beta1", "1.2");

    assert_versions_ordered("1.0-alpha-1", "1.0");
    assert_versions_ordered("1.0-alpha-1", "1.0-alpha-2");
    assert_versions_ordered("1.0-alpha-2", "1.0-alpha-15");
    assert_versions_ordered("1.0-alpha-1", "1.0-beta-1");

    assert_versions_ordered("1.0-beta-1", "1.0-SNAPSHOT");
    assert_versions_ordered("1.0-SNAPSHOT", "1.0");
    assert_versions_ordered("1.0-alpha-1-SNAPSHOT", "1.0-alpha-1");

    assert_versions_ordered("1.0", "1.0-1");
    assert_versions_ordered("1.0-1", "1.0-2");
    assert_versions_equal("2.0-0", "2.0");
    assert_versions_ordered("2.0", "2.0-1");
    assert_versions_ordered("2.0.0", "2.0-1");
    assert_versions_ordered("2.0-1", "2.0.1");

    assert_versions_ordered("2.0.1-klm", "2.0.1-lmn");
    assert_versions_ordered("2.0.1", "2.0.1-xyz");
    assert_versions_ordered("2.0.1-xyz-1", "2.0.1-1-xyz");

    assert_versions_ordered("2.0.1", "2.0.1-123");
    assert_versions_ordered("2.0.1-xyz", "2.0.1-123");

    assert_versions_ordered("1.2.3-10000000000", "1.2.3-10000000001");
    assert_versions_ordered("1.2.3-1", "1.2.3-10000000001");
    assert_versions_ordered("2.3.0-v200706262000", "2.3.0-v200706262130");
    assert_versions_ordered(
        "2.0.0.v200706041905-7C78EK9E_EkMNfNOd2d8qq",
        "2.0.0.v200706041906-7C78EK9E_EkMNfNOd2d8qq",
    );
}

#[test]
fn test_snapshot_order() {
    assert_versions_equal("1-SNAPSHOT", "1-SNAPSHOT");
    assert_versions_ordered("1-SNAPSHOT", "2-SNAPSHOT");
    assert_versions_ordered("1.5-SNAPSHOT", "2-SNAPSHOT");
    assert_versions_ordered("1-SNAPSHOT", "2.5-SNAPSHOT");
    assert_versions_equal("1-SNAPSHOT", "1.0-SNAPSHOT");
    assert_versions_equal("1-SNAPSHOT", "1.0.0-SNAPSHOT");
    assert_versions_ordered("1.0-SNAPSHOT", "1.1-SNAPSHOT");
    assert_versions_ordered("1.1-SNAPSHOT", "1.2-SNAPSHOT");
    assert_versions_ordered("1.0.0-SNAPSHOT", "1.1-SNAPSHOT");
    assert_versions_ordered("1.1-SNAPSHOT", "1.2.0-SNAPSHOT");
    assert_versions_ordered("1.0-alpha-1-SNAPSHOT", "1.0-alpha-2-SNAPSHOT");
    assert_versions_ordered("1.0-alpha-1-SNAPSHOT", "1.0-beta-1-SNAPSHOT");

    assert_versions_ordered("1.0-beta-1-SNAPSHOT", "1.0-SNAPSHOT-SNAPSHOT");
    assert_versions_ordered("1.0-SNAPSHOT-SNAPSHOT", "1.0-SNAPSHOT");
    assert_versions_ordered("1.0-alpha-1-SNAPSHOT-SNAPSHOT", "1.0-alpha-1-SNAPSHOT");

    assert_versions_ordered("1.0-SNAPSHOT", "1.0-1-SNAPSHOT");
    assert_versions_ordered("1.0-1-SNAPSHOT", "1.0-2-SNAPSHOT");
    assert_versions_ordered("2.0-SNAPSHOT", "2.0-1-SNAPSHOT");
    assert_versions_ordered("2.0.0-SNAPSHOT", "2.0-1-SNAPSHOT");
    assert_versions_ordered("2.0-1-SNAPSHOT", "2.0.1-SNAPSHOT");

    assert_versions_ordered("2.0.1-klm-SNAPSHOT", "2.0.1-lmn-SNAPSHOT");
    assert_versions_ordered("2.0.1-SNAPSHOT", "2.0.1-123-SNAPSHOT");
    assert_versions_ordered("2.0.1-xyz-SNAPSHOT", "2.0.1-123-SNAPSHOT");
}

#[test]
fn test_snapshot_release() {
    assert_versions_ordered("1.0-RC1", "1.0-SNAPSHOT");
    assert_versions_ordered("1.0-rc1", "1.0-SNAPSHOT");
    assert_versions_ordered("1.0-rc-1", "1.0-SNAPSHOT");
}
