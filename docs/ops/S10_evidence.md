# S10 Kickoff Evidence

> [!NOTE]
> Initial run (below) was WIP/dirty. See "Clean Rail Re-run" at the bottom for the sterile baseline.

## Verify (WIP)
# PR verify report

このレポートは `nix run .#prverify` の実行結果です。

## Environment

- timestamp (UTC): 20260213T024109Z
- git sha: 3cff447
- rustc: rustc 1.92.0 (ded5c06cf 2025-12-08)
- cargo: cargo 1.92.0 (344c4567c 2025-10-21)
- go: go version go1.24.11 darwin/arm64

## Commands
```bash
cargo test -p veil-cli --test cli_tests
cargo test --workspace
go run ./cmd/prverify
```

## Drift Check (Go)
```
==> cargo test -p veil-cli --test cli_tests
   Compiling ring v0.17.14
   Compiling libz-sys v1.1.23
   Compiling openssl-sys v0.9.111
   Compiling objc2 v0.6.3
   Compiling blake3 v1.8.3
   Compiling libssh2-sys v0.3.1
   Compiling rustls v0.23.36
   Compiling block2 v0.6.2
   Compiling dispatch2 v0.3.0
   Compiling ctrlc v3.5.2
   Compiling libgit2-sys v0.18.3+1.9.2
   Compiling rustls-webpki v0.103.9
   Compiling tokio-rustls v0.26.4
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.12.28
   Compiling veil-guardian v0.17.0 (<repo_root>/crates/veil-guardian)
   Compiling veil-core v0.17.0 (<repo_root>/crates/veil-core)
   Compiling git2 v0.20.4
   Compiling veil-cli v0.17.0 (<repo_root>/crates/veil-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.44s
     Running tests/cli_tests.rs (target/debug/deps/cli_tests-878ade359a526a3a)

running 1 test
Testing tests/cmd/smoke_version.toml ... ok 1s 384ms 293us 167ns
Testing tests/cmd/help.toml ... ok 1s 385ms 123us 84ns
Testing tests/cmd/smoke_config_check.toml ... ok 1s 487ms 681us 41ns
Testing tests/cmd/smoke_filter.toml ... ok 1s 491ms 98us 958ns
Testing tests/cmd/smoke_mask_dryrun.toml ... ok 1s 494ms 255us 875ns
Testing tests/cmd/smoke_scan_json.toml ... ok 1s 499ms 885us 42ns
test cli_tests ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.50s

==> cargo test --workspace
   Compiling ring v0.17.14
   Compiling git2 v0.20.4
   Compiling rustls-webpki v0.103.9
   Compiling rustls v0.23.36
   Compiling tokio-rustls v0.26.4
   Compiling sqlx-core v0.8.6
   Compiling hyper-rustls v0.27.7
   Compiling reqwest v0.12.28
   Compiling sqlx-postgres v0.8.6
   Compiling veil-guardian v0.17.0 (./crates/veil-guardian)
   Compiling veil-core v0.17.0 (./crates/veil-core)
   Compiling veil-cli v0.17.0 (./crates/veil-cli)
   Compiling veil-server v0.17.0 (./crates/veil-server)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 24.37s
     Running unittests src/main.rs (target/debug/deps/veil-7b4d0451de6f73b2)

running 13 tests
test commands::init::tests::test_build_config_standard ... ok
test formatters::json::tests::test_json_output ... ok
test formatters::markdown::tests::test_markdown_output ... ok
test commands::triage::tests::test_add_ignore_path_creates_structure ... ok
test commands::triage::tests::test_add_ignore_path_preserves_comments ... ok
 Severity | Score | Rule ID   | File     | Line | Match 
----------+-------+-----------+----------+------+-------
 High     | 80    | test_rule | test.txt | 1    | 123 
test formatters::table::tests::test_table_output ... ok
test commands::exceptions::tests::test_resolve_priority_none ... ok
test commands::exceptions::tests::test_system_registry_overrides_explicit ... ok
test commands::exceptions::tests::test_resolve_priority_explicit_path ... ok
test commands::exceptions::tests::test_resolve_priority_system_registry ... ok
test commands::exceptions::tests::test_resolve_priority_repo_default_exists ... ok
test config_loader::tests::test_repo_only_config ... ok
test formatters::html::tests::test_html_generation ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/baseline_apply_test.rs (target/debug/deps/baseline_apply_test-7e3d30f590fa7262)

running 2 tests
test test_baseline_error_conditions ... ok
test test_baseline_application_flow ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.62s

     Running tests/baseline_error_test.rs (target/debug/deps/baseline_error_test-30c63ac0c6ac8caf)

running 3 tests
test test_missing_baseline_file ... ok
test test_empty_baseline_file ... ok
test test_corrupt_baseline_file ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/baseline_test.rs (target/debug/deps/baseline_test-9166288074dd4ceb)

running 2 tests
test baseline_argument_conflicts_with_write_baseline ... ok
test write_baseline_creates_file_with_schema ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/baseline_ux_test.rs (target/debug/deps/baseline_ux_test-fdf810b9feb8bae6)

running 4 tests
test test_ux_case_a_clean ... ok
test test_ux_case_d_json_schema ... ok
test test_ux_case_b_suppressed_clean ... ok
test test_ux_case_c_dirty ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s

     Running tests/binary_large_test.rs (target/debug/deps/binary_large_test-863f45d83fd229fa)

running 1 test
test test_binary_and_large_files ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/ci_flags_test.rs (target/debug/deps/ci_flags_test-a82a5b244150a824)

running 2 tests
test test_fail_on_score ... ok
test test_fail_on_severity ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

     Running tests/cli_tests.rs (target/debug/deps/cli_tests-40d919cae03efb2e)

running 1 test
Testing tests/cmd/smoke_version.toml ... ok 14ms 335us 916ns
Testing tests/cmd/help.toml ... ok 15ms 758us 458ns
Testing tests/cmd/smoke_config_check.toml ... ok 114ms 209us 167ns
Testing tests/cmd/smoke_filter.toml ... ok 116ms 968us
Testing tests/cmd/smoke_mask_dryrun.toml ... ok 119ms 403us 250ns
Testing tests/cmd/smoke_scan_json.toml ... ok 124ms 206us 958ns
test cli_tests ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/config_dump_test.rs (target/debug/deps/config_dump_test-2e5f5f5ed57e5a75)

running 6 tests
test config_dump_org_is_empty_by_default ... ok
test config_dump_toml_format ... ok
test config_dump_org_env_explicit ... ok
test config_dump_user_xdg_implicit ... ok
test config_dump_repo_and_effective_json ... ok
test config_layer_precedence ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/exceptions_flags_test.rs (target/debug/deps/exceptions_flags_test-f855c65e49c15b0e)

running 3 tests
test test_system_registry_flag_is_boolean ... ok
test test_exceptions_help_shows_all_flags ... ok
test test_exceptions_flag_exclusivity ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/exit_code_test.rs (target/debug/deps/exit_code_test-1a59820895ab7604)

running 1 test
test test_exit_code_behavior ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/filter_config_test.rs (target/debug/deps/filter_config_test-6b767cacf0b97e01)

running 4 tests
test test_filter_config_rule_override ... ok
test test_filter_config_default_placeholder ... ok
test test_filter_exit_code_zero ... ok
test test_filter_config_custom_placeholder ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/filter_rules_pack_test.rs (target/debug/deps/filter_rules_pack_test-cbd84b5d88cdac42)

running 2 tests
test test_filter_load_rules_pack ... ok
test test_filter_load_rules_pack_jp ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/html_baseline_test.rs (target/debug/deps/html_baseline_test-27ce058bc33f90a7)

running 1 test
test test_html_report_baseline_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

     Running tests/html_report_test.rs (target/debug/deps/html_report_test-3d08f86e4fdf4e65)

running 1 test
test html_report_contains_metadata_and_interactive_elements ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running tests/init_log_pack_test.rs (target/debug/deps/init_log_pack_test-552cde271c3e250d)

running 2 tests
test test_init_app_profile_defaults ... ok
test test_init_logs_profile_generates_pack ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.39s

     Running tests/init_test.rs (target/debug/deps/init_test-0d3dc491838f233c)

running 3 tests
test test_init_ci_unsupported ... ok
test test_init_ci_github ... ok
test test_init_ci_github_pinned_none ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/json_contract_tests.rs (target/debug/deps/json_contract_tests-f68ec4c70705e81a)

running 1 test
test json_contract_simple_project_matches_golden_file ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/json_schema_version_test.rs (target/debug/deps/json_schema_version_test-e23e30bfbee962e8)

running 1 test
test test_json_output_has_schema_version ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/policy_test.rs (target/debug/deps/policy_test-162d971d085cfb40)

running 3 tests
test test_policy_layering_override ... ok
test test_policy_layering_ignore_extend ... ok
test test_policy_layering_fail_score ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s

     Running tests/precommit_test.rs (target/debug/deps/precommit_test-b22212b6a526a7fa)

running 1 test
test test_pre_commit_init ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/rules_test.rs (target/debug/deps/rules_test-80439b7983b3914f)

running 4 tests
test rules_explain_invalid_fails ... ok
test rules_list_severity_filter ... ok
test rules_list_shows_known_rule ... ok
test rules_explain_shows_details ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/scan_hang_test.rs (target/debug/deps/scan_hang_test-d86218490b28ab50)

running 1 test
test scan_directory_terminates_ok ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/sot_new_test.rs (target/debug/deps/sot_new_test-77ffd5f09d5f7b78)

running 3 tests
test test_sot_new_slug_dry_run ... ok
test test_sot_new_success ... ok
test test_sot_new_force ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/sot_rename_test.rs (target/debug/deps/sot_rename_test-0ede2083dd427125)

running 3 tests
test test_sot_rename_multiple_candidates_requires_path ... ok
test test_sot_rename_dry_run ... ok
test test_sot_rename_success_autodetect ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/staged_test.rs (target/debug/deps/staged_test-bb26cdb05caf877b)

running 2 tests
test test_staged_scan ... ok
test test_fail_score_env ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

     Running unittests src/lib.rs (target/debug/deps/veil_config-fcde446eb1169323)

running 2 tests
test validate::tests::test_valid_modes ... ok
test validate::tests::test_fail_fast_plain_mode ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/veil_core-dfe74f3abe60a88a)

running 36 tests
test finding_id::tests::test_finding_id_sensitivity ... ok
test baseline::tests::test_apply_baseline_none ... ok
test baseline::tests::test_apply_baseline_empty_snapshot ... ok
test baseline::tests::fingerprint_is_stable_for_same_input ... ok
test baseline::tests::test_apply_baseline_empty_findings ... ok
test baseline::tests::test_apply_baseline_duplicates ... ok
test baseline::tests::test_apply_baseline_partitioning ... ok
test masking::tests::test_mask_custom_placeholder ... ok
test finding_id::tests::test_finding_id_stability ... ok
test masking::tests::test_mask_partial ... ok
test masking::tests::test_mask_partial_short ... ok
test masking::tests::test_mask_ranges_adjacent ... ok
test baseline::tests::snapshot_roundtrip_json ... ok
test finding_id::tests::test_serde_roundtrip ... ok
test masking::tests::test_mask_ranges_nested ... ok
test masking::tests::test_mask_ranges_overlapping ... ok
test masking::tests::test_mask_ranges_simple ... ok
test masking::tests::test_mask_spans_multi_component ... ok
test masking::tests::test_mask_spans_obs_pii_overlap ... ok
test masking::tests::test_mask_spans_priority_overlap ... ok
test masking::tests::test_mask_spans_same_priority ... ok
test masking::tests::test_plain_mode_returns_original ... ok
test registry::tests::test_registry_check ... ok
test registry::tests::test_serde_roundtrip ... ok
test scanner::tests::test_context_capture_start_of_file ... ok
test scanner::tests::test_context_capture ... ok
test scanner::tests::test_inline_ignore ... ok
test summary::v1::tests::test_summary_serialization ... ok
test scoring::tests::test_base_score_default ... ok
test scoring::tests::test_context_modifiers ... ok
test registry::tests::test_save_canonical_sort ... ok
test registry::tests::test_version_mismatch ... ok
test rules::pack::tests::test_load_auto_order ... ok
test registry::tests::test_save_atomic_and_load ... ok
test rules::pack::tests::test_duplicate_id_error ... ok
test rules::pack::tests::test_load_manifest_order ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/dogfood_schema.rs (target/debug/deps/dogfood_schema-015998c35a751aa3)

running 2 tests
test test_metrics_schema_validity ... ok
test test_worklist_schema_validity ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/dos_safety.rs (target/debug/deps/dos_safety-b5251fcd461a7038)

running 2 tests
test test_many_matches_dos ... ok
test test_large_input_performance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/golden_metrics.rs (target/debug/deps/golden_metrics-cd4f754d9447982d)

running 1 test
test test_golden_metrics_deterministic_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/lock_busy_proof.rs (target/debug/deps/lock_busy_proof-e03deb4381575e4e)

running 3 tests
test test_lock_busy_error_message_contract ... ok
test test_save_lock_busy_is_non_blocking ... ok
test test_lock_busy_is_non_blocking ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/rule_tests.rs (target/debug/deps/rule_tests-3b98576cc52369b8)

running 3 tests
test fake_slack_tokens_have_expected_format ... ok
test rule_creds_slack_token_legacy_detects_dynamic_tokens ... ok
test test_rules_from_fixtures ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

     Running unittests src/lib.rs (target/debug/deps/veil_guardian-0ae04781f3b9337e)

running 11 tests
test providers::npm::tests::test_extract_name ... ok
test providers::yarn::tests::test_parse_package_name ... ok
test providers::yarn::tests::test_berry_parsing_skips_protocols ... ok
test providers::pnpm::tests::test_parse_v6_slash_style ... ok
test providers::pnpm::tests::test_parse_v9_at_style ... ok
test providers::yarn::tests::test_berry_parsing_valid ... ok
test providers::yarn::tests::test_classic_skips_protocols ... ok
test providers::yarn::tests::test_classic_header_quote_aware_split ... ok
test providers::yarn::tests::test_classic_parsing ... ok
test providers::osv::details_store::tests::cache_roundtrip_with_etag ... ok
test providers::osv::details_store::tests::test_load_legacy_format_path_check ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/atomic_write.rs (target/debug/deps/atomic_write-0a73b554ae891094)

running 4 tests
test test_atomic_write_creates_file ... ok
test test_atomic_write_creates_parent_dir ... ok
test test_no_tmp_leftovers ... ok
test test_atomic_write_overwrites_existing ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/basic.rs (target/debug/deps/basic-cf91acc0278c47d3)

running 3 tests
test test_load_builtin ... ok
test test_version_match ... ok
test test_check_vulnerabilities ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/coalescing.rs (target/debug/deps/coalescing-9a11052a53ada18a)

running 2 tests
test test_querybatch_coalesces_in_flight ... ok
test test_details_coalesces_in_flight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.56s

     Running tests/contract_osv_fixtures.rs (target/debug/deps/contract_osv_fixtures-c0ff329a7407217b)

running 1 test
test test_contract_osv_fixtures ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/file_lock.rs (target/debug/deps/file_lock-5d5c703fe9e920fe)

running 2 tests
test test_lock_file_creation ... ok
test test_concurrent_writes_are_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/guardian_next_retry.rs (target/debug/deps/guardian_next_retry-3138a11aab95313f)

running 1 test
test retry_on_500_then_success ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.23s

     Running tests/guardian_next_singleflight.rs (target/debug/deps/guardian_next_singleflight-c3eb2356e48f4e45)

running 1 test
test singleflight_runs_only_once ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/key_versioning.rs (target/debug/deps/key_versioning-e84114869e567740)

running 6 tests
test test_normalize_key_safe_chars ... ok
test test_normalize_key_truncation ... ok
test test_normalize_key_collision_avoidance ... ok
test test_complex_id_roundtrip_with_etag ... ok
test test_legacy_fallback ... ok
test test_directory_conflict ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.10s

     Running tests/live_osv_api.rs (target/debug/deps/live_osv_api-2da9c4b707c556ff)

running 1 test
test test_live_osv_lodash_check ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/metrics.rs (target/debug/deps/metrics-033a576db7f6772c)

running 1 test
test test_metrics_collection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.52s

     Running tests/npm_integration.rs (target/debug/deps/npm_integration-d1020315d3c2cc23)

running 1 test
test test_npm_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.46s

     Running tests/osv_concurrency.rs (target/debug/deps/osv_concurrency-d77856f0f999a4fe)

running 1 test
test respects_max_in_flight_concurrency_gate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.62s

     Running tests/osv_details_flow.rs (target/debug/deps/osv_details_flow-f6c39a4a5bb8ba90)

running 3 tests
test test_osv_details_offline_uses_stale ... ok
test test_osv_details_flow_fresh_skips_fetch ... ok
test test_osv_details_flow_expired_triggers_fetch ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.36s

     Running tests/osv_operator_ux.rs (target/debug/deps/osv_operator_ux-b4d103fb071268bb)

running 2 tests
test test_operator_message_offline_remediation ... ok
test test_operator_message_includes_quarantine_note ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.36s

     Running tests/osv_retry.rs (target/debug/deps/osv_retry-af47ffdc65200cb9)

running 3 tests
test retry_after_exceeds_budget_fails_fast ... ok
test retry_on_500_records_backoff_sleeps ... ok
test retry_after_is_respected ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.45s

     Running tests/osv_retry_metrics.rs (target/debug/deps/osv_retry_metrics-623f55ea4e6cce93)

running 1 test
test verify_metrics_on_retry_and_success ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.54s

     Running tests/pnpm_integration.rs (target/debug/deps/pnpm_integration-b210d6f0e0400dac)

running 1 test
test test_pnpm_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.33s

     Running tests/report_output.rs (target/debug/deps/report_output-c87867a243fb3ce3)

running 3 tests
test test_report_grouping_logic ... ok
test test_report_output_duplicates ... ok
test test_report_output_deterministic ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.36s

     Running tests/yarn_integration.rs (target/debug/deps/yarn_integration-fa1d53186c6eb927)

running 1 test
test test_yarn_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s

     Running unittests src/main.rs (target/debug/deps/veil_server-dce78b447cb20b81)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests veil_config

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests veil_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests veil_guardian

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

==> Dependency Guard
==> Drift Check

## Notes / Evidence

Local env:
- rustc: rustc 1.92.0 (ded5c06cf 2025-12-08)
- cargo: cargo 1.92.0 (344c4567c 2025-10-21)
- git: 3cff44758ba5 (dirty)

Tests:
- `cargo test -p veil-cli --test cli_tests` => OK (18.51s)
- `cargo test --workspace` => OK (186.40s)
- `dep-guard` => OK (788ms)
- `drift-check` => OK (1ms)

## Rollback

Revert the merge/squash commit for this PR.
```bash
# 1コミットだけ戻す
git revert <commit>

# 範囲でまとめて戻す
git revert <oldest_commit>^..<newest_commit>
```

- Local run: PASS
```

- exit_code: 0

PASS: All checks passed.
---
report: .local/prverify/prverify_20260213T024109Z_3cff447.md
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.61s
     Running unittests src/main.rs (target/debug/deps/veil-7b4d0451de6f73b2)

running 13 tests
test commands::init::tests::test_build_config_standard ... ok
test formatters::json::tests::test_json_output ... ok
test commands::triage::tests::test_add_ignore_path_creates_structure ... ok
test commands::triage::tests::test_add_ignore_path_preserves_comments ... ok
test commands::exceptions::tests::test_resolve_priority_none ... ok
test formatters::markdown::tests::test_markdown_output ... ok
test commands::exceptions::tests::test_system_registry_overrides_explicit ... ok
test commands::exceptions::tests::test_resolve_priority_explicit_path ... ok
test commands::exceptions::tests::test_resolve_priority_system_registry ... ok
 Severity | Score | Rule ID   | File     | Line | Match 
----------+-------+-----------+----------+------+-------
 High     | 80    | test_rule | test.txt | 1    | 123 
test formatters::table::tests::test_table_output ... ok
test config_loader::tests::test_repo_only_config ... ok
test commands::exceptions::tests::test_resolve_priority_repo_default_exists ... ok
test formatters::html::tests::test_html_generation ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/baseline_apply_test.rs (target/debug/deps/baseline_apply_test-7e3d30f590fa7262)

running 2 tests
test test_baseline_error_conditions ... ok
test test_baseline_application_flow ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.43s

     Running tests/baseline_error_test.rs (target/debug/deps/baseline_error_test-30c63ac0c6ac8caf)

running 3 tests
test test_missing_baseline_file ... ok
test test_empty_baseline_file ... ok
test test_corrupt_baseline_file ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/baseline_test.rs (target/debug/deps/baseline_test-9166288074dd4ceb)

running 2 tests
test baseline_argument_conflicts_with_write_baseline ... ok
test write_baseline_creates_file_with_schema ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/baseline_ux_test.rs (target/debug/deps/baseline_ux_test-fdf810b9feb8bae6)

running 4 tests
test test_ux_case_a_clean ... ok
test test_ux_case_d_json_schema ... ok
test test_ux_case_b_suppressed_clean ... ok
test test_ux_case_c_dirty ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running tests/binary_large_test.rs (target/debug/deps/binary_large_test-863f45d83fd229fa)

running 1 test
test test_binary_and_large_files ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/ci_flags_test.rs (target/debug/deps/ci_flags_test-a82a5b244150a824)

running 2 tests
test test_fail_on_score ... ok
test test_fail_on_severity ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/cli_tests.rs (target/debug/deps/cli_tests-40d919cae03efb2e)

running 1 test
Testing tests/cmd/smoke_version.toml ... ok 9ms 383us 666ns
Testing tests/cmd/help.toml ... ok 10ms 160us 42ns
Testing tests/cmd/smoke_config_check.toml ... ok 96ms 379us
Testing tests/cmd/smoke_filter.toml ... ok 100ms 856us 333ns
Testing tests/cmd/smoke_mask_dryrun.toml ... ok 102ms 622us 625ns
Testing tests/cmd/smoke_scan_json.toml ... ok 107ms 307us 834ns
test cli_tests ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/config_dump_test.rs (target/debug/deps/config_dump_test-2e5f5f5ed57e5a75)

running 6 tests
test config_dump_org_is_empty_by_default ... ok
test config_dump_org_env_explicit ... ok
test config_dump_toml_format ... ok
test config_dump_user_xdg_implicit ... ok
test config_dump_repo_and_effective_json ... ok
test config_layer_precedence ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/exceptions_flags_test.rs (target/debug/deps/exceptions_flags_test-f855c65e49c15b0e)

running 3 tests
test test_system_registry_flag_is_boolean ... ok
test test_exceptions_flag_exclusivity ... ok
test test_exceptions_help_shows_all_flags ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/exit_code_test.rs (target/debug/deps/exit_code_test-1a59820895ab7604)

running 1 test
test test_exit_code_behavior ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

     Running tests/filter_config_test.rs (target/debug/deps/filter_config_test-6b767cacf0b97e01)

running 4 tests
test test_filter_config_rule_override ... ok
test test_filter_exit_code_zero ... ok
test test_filter_config_default_placeholder ... ok
test test_filter_config_custom_placeholder ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/filter_rules_pack_test.rs (target/debug/deps/filter_rules_pack_test-cbd84b5d88cdac42)

running 2 tests
test test_filter_load_rules_pack_jp ... ok
test test_filter_load_rules_pack ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

     Running tests/html_baseline_test.rs (target/debug/deps/html_baseline_test-27ce058bc33f90a7)

running 1 test
test test_html_report_baseline_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/html_report_test.rs (target/debug/deps/html_report_test-3d08f86e4fdf4e65)

running 1 test
test html_report_contains_metadata_and_interactive_elements ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running tests/init_log_pack_test.rs (target/debug/deps/init_log_pack_test-552cde271c3e250d)

running 2 tests
test test_init_app_profile_defaults ... ok
test test_init_logs_profile_generates_pack ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.37s

     Running tests/init_test.rs (target/debug/deps/init_test-0d3dc491838f233c)

running 3 tests
test test_init_ci_unsupported ... ok
test test_init_ci_github_pinned_none ... ok
test test_init_ci_github ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/json_contract_tests.rs (target/debug/deps/json_contract_tests-f68ec4c70705e81a)

running 1 test
test json_contract_simple_project_matches_golden_file ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/json_schema_version_test.rs (target/debug/deps/json_schema_version_test-e23e30bfbee962e8)

running 1 test
test test_json_output_has_schema_version ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/policy_test.rs (target/debug/deps/policy_test-162d971d085cfb40)

running 3 tests
test test_policy_layering_override ... ok
test test_policy_layering_ignore_extend ... ok
test test_policy_layering_fail_score ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running tests/precommit_test.rs (target/debug/deps/precommit_test-b22212b6a526a7fa)

running 1 test
test test_pre_commit_init ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/rules_test.rs (target/debug/deps/rules_test-80439b7983b3914f)

running 4 tests
test rules_explain_invalid_fails ... ok
test rules_explain_shows_details ... ok
test rules_list_severity_filter ... ok
test rules_list_shows_known_rule ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/scan_hang_test.rs (target/debug/deps/scan_hang_test-d86218490b28ab50)

running 1 test
test scan_directory_terminates_ok ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/sot_new_test.rs (target/debug/deps/sot_new_test-77ffd5f09d5f7b78)

running 3 tests
test test_sot_new_success ... ok
test test_sot_new_slug_dry_run ... ok
test test_sot_new_force ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/sot_rename_test.rs (target/debug/deps/sot_rename_test-0ede2083dd427125)

running 3 tests
test test_sot_rename_multiple_candidates_requires_path ... ok
test test_sot_rename_dry_run ... ok
test test_sot_rename_success_autodetect ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/staged_test.rs (target/debug/deps/staged_test-bb26cdb05caf877b)

running 2 tests
test test_staged_scan ... ok
test test_fail_score_env ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

     Running unittests src/lib.rs (target/debug/deps/veil_config-fcde446eb1169323)

running 2 tests
test validate::tests::test_fail_fast_plain_mode ... ok
test validate::tests::test_valid_modes ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/veil_core-dfe74f3abe60a88a)

running 36 tests
test baseline::tests::snapshot_roundtrip_json ... ok
test baseline::tests::test_apply_baseline_duplicates ... ok
test baseline::tests::test_apply_baseline_empty_findings ... ok
test baseline::tests::test_apply_baseline_none ... ok
test baseline::tests::fingerprint_is_stable_for_same_input ... ok
test baseline::tests::test_apply_baseline_empty_snapshot ... ok
test finding_id::tests::test_finding_id_sensitivity ... ok
test baseline::tests::test_apply_baseline_partitioning ... ok
test finding_id::tests::test_serde_roundtrip ... ok
test finding_id::tests::test_finding_id_stability ... ok
test masking::tests::test_mask_custom_placeholder ... ok
test masking::tests::test_mask_partial ... ok
test masking::tests::test_mask_ranges_adjacent ... ok
test masking::tests::test_mask_partial_short ... ok
test masking::tests::test_mask_ranges_nested ... ok
test masking::tests::test_mask_ranges_overlapping ... ok
test masking::tests::test_mask_spans_obs_pii_overlap ... ok
test registry::tests::test_registry_check ... ok
test masking::tests::test_mask_ranges_simple ... ok
test masking::tests::test_mask_spans_multi_component ... ok
test masking::tests::test_mask_spans_priority_overlap ... ok
test masking::tests::test_mask_spans_same_priority ... ok
test masking::tests::test_plain_mode_returns_original ... ok
test registry::tests::test_serde_roundtrip ... ok
test scanner::tests::test_context_capture_start_of_file ... ok
test scanner::tests::test_context_capture ... ok
test scanner::tests::test_inline_ignore ... ok
test summary::v1::tests::test_summary_serialization ... ok
test scoring::tests::test_base_score_default ... ok
test scoring::tests::test_context_modifiers ... ok
test registry::tests::test_version_mismatch ... ok
test registry::tests::test_save_atomic_and_load ... ok
test rules::pack::tests::test_load_auto_order ... ok
test rules::pack::tests::test_duplicate_id_error ... ok
test rules::pack::tests::test_load_manifest_order ... ok
test registry::tests::test_save_canonical_sort ... ok

test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/dogfood_schema.rs (target/debug/deps/dogfood_schema-015998c35a751aa3)

running 2 tests
test test_metrics_schema_validity ... ok
test test_worklist_schema_validity ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/dos_safety.rs (target/debug/deps/dos_safety-b5251fcd461a7038)

running 2 tests
test test_many_matches_dos ... ok
test test_large_input_performance ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/golden_metrics.rs (target/debug/deps/golden_metrics-cd4f754d9447982d)

running 1 test
test test_golden_metrics_deterministic_output ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/lock_busy_proof.rs (target/debug/deps/lock_busy_proof-e03deb4381575e4e)

running 3 tests
test test_lock_busy_error_message_contract ... ok
test test_lock_busy_is_non_blocking ... ok
test test_save_lock_busy_is_non_blocking ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/rule_tests.rs (target/debug/deps/rule_tests-3b98576cc52369b8)

running 3 tests
test fake_slack_tokens_have_expected_format ... ok
test rule_creds_slack_token_legacy_detects_dynamic_tokens ... ok
test test_rules_from_fixtures ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running unittests src/lib.rs (target/debug/deps/veil_guardian-0ae04781f3b9337e)

running 11 tests
test providers::npm::tests::test_extract_name ... ok
test providers::yarn::tests::test_parse_package_name ... ok
test providers::yarn::tests::test_berry_parsing_skips_protocols ... ok
test providers::yarn::tests::test_classic_skips_protocols ... ok
test providers::yarn::tests::test_classic_parsing ... ok
test providers::yarn::tests::test_classic_header_quote_aware_split ... ok
test providers::yarn::tests::test_berry_parsing_valid ... ok
test providers::pnpm::tests::test_parse_v9_at_style ... ok
test providers::pnpm::tests::test_parse_v6_slash_style ... ok
test providers::osv::details_store::tests::cache_roundtrip_with_etag ... ok
test providers::osv::details_store::tests::test_load_legacy_format_path_check ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/atomic_write.rs (target/debug/deps/atomic_write-0a73b554ae891094)

running 4 tests
test test_atomic_write_creates_file ... ok
test test_no_tmp_leftovers ... ok
test test_atomic_write_creates_parent_dir ... ok
test test_atomic_write_overwrites_existing ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/basic.rs (target/debug/deps/basic-cf91acc0278c47d3)

running 3 tests
test test_check_vulnerabilities ... ok
test test_version_match ... ok
test test_load_builtin ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/coalescing.rs (target/debug/deps/coalescing-9a11052a53ada18a)

running 2 tests
test test_querybatch_coalesces_in_flight ... ok
test test_details_coalesces_in_flight ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.44s

     Running tests/contract_osv_fixtures.rs (target/debug/deps/contract_osv_fixtures-c0ff329a7407217b)

running 1 test
test test_contract_osv_fixtures ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/file_lock.rs (target/debug/deps/file_lock-5d5c703fe9e920fe)

running 2 tests
test test_lock_file_creation ... ok
test test_concurrent_writes_are_safe ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

     Running tests/guardian_next_retry.rs (target/debug/deps/guardian_next_retry-3138a11aab95313f)

running 1 test
test retry_on_500_then_success ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/guardian_next_singleflight.rs (target/debug/deps/guardian_next_singleflight-c3eb2356e48f4e45)

running 1 test
test singleflight_runs_only_once ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/key_versioning.rs (target/debug/deps/key_versioning-e84114869e567740)

running 6 tests
test test_normalize_key_collision_avoidance ... ok
test test_normalize_key_safe_chars ... ok
test test_normalize_key_truncation ... ok
test test_complex_id_roundtrip_with_etag ... ok
test test_legacy_fallback ... ok
test test_directory_conflict ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/live_osv_api.rs (target/debug/deps/live_osv_api-2da9c4b707c556ff)

running 1 test
test test_live_osv_lodash_check ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/metrics.rs (target/debug/deps/metrics-033a576db7f6772c)

running 1 test
test test_metrics_collection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/npm_integration.rs (target/debug/deps/npm_integration-d1020315d3c2cc23)

running 1 test
test test_npm_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.40s

     Running tests/osv_concurrency.rs (target/debug/deps/osv_concurrency-d77856f0f999a4fe)

running 1 test
test respects_max_in_flight_concurrency_gate ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.48s

     Running tests/osv_details_flow.rs (target/debug/deps/osv_details_flow-f6c39a4a5bb8ba90)

running 3 tests
test test_osv_details_offline_uses_stale ... ok
test test_osv_details_flow_fresh_skips_fetch ... ok
test test_osv_details_flow_expired_triggers_fetch ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/osv_operator_ux.rs (target/debug/deps/osv_operator_ux-b4d103fb071268bb)

running 2 tests
test test_operator_message_offline_remediation ... ok
test test_operator_message_includes_quarantine_note ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s

     Running tests/osv_retry.rs (target/debug/deps/osv_retry-af47ffdc65200cb9)

running 3 tests
test retry_after_exceeds_budget_fails_fast ... ok
test retry_on_500_records_backoff_sleeps ... ok
test retry_after_is_respected ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/osv_retry_metrics.rs (target/debug/deps/osv_retry_metrics-623f55ea4e6cce93)

running 1 test
test verify_metrics_on_retry_and_success ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

     Running tests/pnpm_integration.rs (target/debug/deps/pnpm_integration-b210d6f0e0400dac)

running 1 test
test test_pnpm_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/report_output.rs (target/debug/deps/report_output-c87867a243fb3ce3)

running 3 tests
test test_report_grouping_logic ... ok
test test_report_output_duplicates ... ok
test test_report_output_deterministic ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/yarn_integration.rs (target/debug/deps/yarn_integration-fa1d53186c6eb927)

running 1 test
test test_yarn_integration ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running unittests src/main.rs (target/debug/deps/veil_server-dce78b447cb20b81)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests veil_config

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests veil_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests veil_guardian

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


--- Issues ---

--- Dependabot Alerts ---
[]

## Clean Rail Re-run (post-commit)

- git: d795bf9 (clean)
- prverify: PASS
- [Portable-first Evidence Policy](S10_ACCEPTANCE.md#2-deterministic-evidence-portable-first)

```text
Tests:
- `cargo test -p veil-cli --test cli_tests` => OK
- `cargo test --workspace` => OK
- `dep-guard` => OK
- `drift-check` => OK

PASS: All checks passed.
Exit code: 0
```
-rw-r--r-- 1 masakitakemura staff 59K Feb 13 12:00 .local/review-bundles/veil-rs_review_wip_20260213_120042_d795bf977a10.tar.gz
OK: .local/review-bundles/veil-rs_review_wip_20260213_120042_d795bf977a10.tar.gz

## S10-02 pr-kit dry-run v1 Evidence

Command: `unset GOROOT && go run ./cmd/prkit --dry-run`

Output:
```json
{
  "schema_version": 1,
  "timestamp_utc": "20260213T044759Z",
  "mode": "dry-run",
  "status": "FAIL",
  "exit_code": 2,
  "git_sha": "652dfe63a21251c0c3a51bfa2dffb66d5308c264",
  "tool_versions": [
    {
      "name": "go",
      "version": "go version go1.24.11 darwin/arm64"
    },
    {
      "name": "git",
      "version": "git version 2.51.2"
    },
    {
      "name": "rustc",
      "version": "rustc 1.92.0 (ded5c06cf 2025-12-08)"
    },
    {
      "name": "cargo",
      "version": "cargo 1.92.0 (344c4567c 2025-10-21)"
    },
    {
      "name": "nix",
      "version": "nix (Nix) 2.32.4"
    }
  ],
  "checks": [
    {
      "name": "git_clean_worktree",
      "status": "FAIL",
      "details": "?? cmd/prkit/\n?? docs/ops/S10_02_PLAN.md\n?? docs/ops/S10_02_TASK.md\n?? internal/prkit/"
    }
  ],
  "command_list": [
    {
      "name": "git_status_porcelain",
      "cmd": "git status --porcelain=v1"
    }
  ],
  "artifact_hashes": []
}
```

## S10-02 Clean Rail Re-run

Command: `unset GOROOT && go run ./cmd/prkit --dry-run`

Output:
```json
{
  "schema_version": 1,
  "timestamp_utc": "20260213T051231Z",
  "mode": "dry-run",
  "status": "PASS",
  "exit_code": 0,
  "git_sha": "fd1e54d1f3dab6ae587bf2427e3e291d03eb4d48",
  "tool_versions": [
    {
      "name": "go",
      "version": "go version go1.24.11 darwin/arm64"
    },
    {
      "name": "git",
      "version": "git version 2.51.2"
    },
    {
      "name": "rustc",
      "version": "rustc 1.92.0 (ded5c06cf 2025-12-08)"
    },
    {
      "name": "cargo",
      "version": "cargo 1.92.0 (344c4567c 2025-10-21)"
    },
    {
      "name": "nix",
      "version": "nix (Nix) 2.32.4"
    }
  ],
  "checks": [
    {
      "name": "git_clean_worktree",
      "status": "PASS",
      "details": "worktree is clean"
    }
  ],
  "command_list": [
    {
      "name": "git_status_porcelain",
      "cmd": "git status --porcelain=v1"
    }
  ],
  "artifact_hashes": []
}
```

## S10-06 Post-merge Copilot PR Audit Evidence

### Audited PRs (Suspects)
- PR #68: fix(prkit): improve error diagnostics in git check functions (app/copilot-swe-agent)
- PR #67: Use CombinedOutput to capture stderr in git status check (app/copilot-swe-agent)
- PR #66: fix(prkit): wrap exec.LookPath error for actionable debugging (app/copilot-swe-agent)
- PR #65: fix(prkit): remove redundant tool name from skip message format (app/copilot-swe-agent)
- PR #64: Refactor RunDryRun to return exit code instead of calling os.Exit (app/copilot-swe-agent)

### Remediation Actions
- **Absolute Paths**: Removed absolute paths (`<repo_root>/`) from `docs/ops/S10_evidence.md` and `docs/evidence` logs.
- **Broken Fences**: Audit (including PR 68 files) found no broken markdown fences in touched files.
- **CWD Dependencies**: No shell execution found requiring `cmd.Dir` pinning fixes.
- **Temp Files**: No accidental tracked temp files found.

### Verification
**Command**: `nix run .#prverify`
**Status**: PASS
**Report**: `.local/prverify/prverify_20260213T104023Z_a7695e1.md`
