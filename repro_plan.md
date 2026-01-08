
# Reproduction Steps

1. Create a `repro_test` directory.
2. Inside, run `veil init` (defaults to Application).
3. Create `test_pii.txt` with JP MyNumber: `マイナンバー 1234-5678-9012`.
4. Run `veil scan`.
5. Check output for `pii.jp.mynumber.keyword` (from `pii_jp.toml`).
6. Check output for `log.pii...` (should NOT be there).

## Verification Code
```bash
mkdir -p repro_test
cd repro_test
../../target/debug/veil init
echo "マイナンバー 1234-5678-9012" > test_pii.txt
../../target/debug/veil scan
```
