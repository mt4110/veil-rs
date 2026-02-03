import os
import sys

def check_file_content(path, required_strings, description):
    print(f"Checking {description} in {path}...")
    if not os.path.exists(path):
        print(f"❌ File not found: {path}")
        return False
    
    with open(path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    missing = []
    for s in required_strings:
        if s not in content:
            missing.append(s)
    
    if missing:
        print(f"❌ Missing content in {path}:")
        for m in missing:
            print(f"  - '{m}'")
        return False
    
    print("✅ OK")
    return True

def main():
    root = os.getcwd()
    ci_yml = os.path.join(root, '.github/workflows/ci.yml')
    sqlx_md = os.path.join(root, 'docs/guardrails/sqlx.md')
    sot_md = os.path.join(root, 'docs/pr/PR-TBD-v0.22.0-epic-a-robust-sqlx.md')

    errors = 0

    # 1. CI Checks
    print("\n--- 1. CI Configuration ---")
    if not check_file_content(ci_yml, [
        'future-incompat-report',      # Future Incompat Check
        '.local/ci/future_incompat.txt', # Log file logic
        'sqlx-cli --version 0.8.6',    # Pinned version
        'tee -a .local/ci/sqlx_cli_install.log', # Install log
        'SQLX_OFFLINE=true',           # Offline check
        'actions/cache@v4'             # Cache version
    ], "CI Workflow"):
        errors += 1

    # 2. Docs Checks
    print("\n--- 2. Documentation ---")
    if not check_file_content(sqlx_md, [
        'SQLX_OFFLINE=true',
        'sqlx_cli_install.log',
        '.local/ci/',                  # Artifact directory
    ], "SQLx Guardrail Docs"):
        errors += 1

    # 3. SOT Checks
    print("\n--- 3. SOT (Source of Truth) ---")
    if not check_file_content(sot_md, [
        'actions/cache@v4',
        'sqlx_cli_install.log',
        '0.8.6'
    ], "SOT"):
        errors += 1

    if errors > 0:
        print(f"\n❌ Validation FAILED with {errors} errors.")
        sys.exit(1)
    
    print("\n✅ All Guardrail Validations PASSED.")

if __name__ == "__main__":
    main()
