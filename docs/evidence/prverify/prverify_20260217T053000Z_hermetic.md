veil-rs dev env loaded (stable)
Rust version: rustc 1.92.0 (ded5c06cf 2025-12-08)
=== RUN   TestCreate_Determinism
--- PASS: TestCreate_Determinism (0.14s)
=== RUN   TestForge_Smoke
--- PASS: TestForge_Smoke (0.10s)
=== RUN   TestVerify_FailsOnKnownBadBundle
=== RUN   TestVerify_FailsOnKnownBadBundle/Forbidden_PAX_key
=== RUN   TestVerify_FailsOnKnownBadBundle/Provenance_leak
=== RUN   TestVerify_FailsOnKnownBadBundle/Non-zero_UID
=== RUN   TestVerify_FailsOnKnownBadBundle/Non-empty_Gname
=== RUN   TestVerify_FailsOnKnownBadBundle/Non-zero_nanoseconds
--- PASS: TestVerify_FailsOnKnownBadBundle (0.00s)
    --- PASS: TestVerify_FailsOnKnownBadBundle/Forbidden_PAX_key (0.00s)
    --- PASS: TestVerify_FailsOnKnownBadBundle/Provenance_leak (0.00s)
    --- PASS: TestVerify_FailsOnKnownBadBundle/Non-zero_UID (0.00s)
    --- PASS: TestVerify_FailsOnKnownBadBundle/Non-empty_Gname (0.00s)
    --- PASS: TestVerify_FailsOnKnownBadBundle/Non-zero_nanoseconds (0.00s)
=== RUN   TestVerify_PassesOnMinimalValidBundle
--- PASS: TestVerify_PassesOnMinimalValidBundle (0.00s)
PASS
ok  	veil-rs/cmd/reviewbundle	0.595s
