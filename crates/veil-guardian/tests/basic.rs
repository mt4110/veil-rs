use semver::Version;
use veil_guardian::GuardianDb;

#[test]
fn test_load_builtin() {
    let db = GuardianDb::load_builtin().expect("Failed to load builtin db");

    // Test known crate
    let advisories = db.advisories_for("dummy-vulnerable-crate");
    assert!(!advisories.is_empty());
    assert_eq!(advisories[0].crate_name, "dummy-vulnerable-crate");

    // Test unknown crate
    let empty = db.advisories_for("safe-crate-9999");
    assert!(empty.is_empty());
}

#[test]
fn test_version_match() {
    let db = GuardianDb::load_builtin().expect("Failed to load builtin db");

    // Vulnerable version
    let vulnerable = Version::parse("0.9.9").unwrap();
    assert!(db.is_version_vulnerable("dummy-vulnerable-crate", &vulnerable));

    // Safe version
    let safe = Version::parse("1.0.0").unwrap();
    assert!(!db.is_version_vulnerable("dummy-vulnerable-crate", &safe));
}

#[test]
fn test_check_vulnerabilities() {
    let db = GuardianDb::load_builtin().expect("Failed to load builtin db");
    let version = Version::parse("0.9.9").unwrap();

    let vulns = db.check_vulnerabilities("dummy-vulnerable-crate", &version);
    assert_eq!(vulns.len(), 1);
    assert_eq!(vulns[0].id, "GF-001");
}
