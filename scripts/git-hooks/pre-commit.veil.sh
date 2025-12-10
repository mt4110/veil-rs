#!/bin/sh
#
# Pre-commit hook to scan staged files with veil-rs
#

if ! command -v veil >/dev/null 2>&1; then
    echo "Veil is not installed. Skipping secret scan."
    exit 0
fi

# Get staged files, robustly handling spaces
# We use git diff --cached --name-only
# And we filter for text files usually, but veil handles binary skipping.
# We just pass the file list.
# 
# Note: if list is empty, we skip.

STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM)

if [ -z "$STAGED_FILES" ]; then
    exit 0
fi

# Run veil scan on the staged files
# We accept spaces in filenames by using echo and xargs carefully, or just trusting current shell split.
# For sh, simplest is passing the list if not too long.
# If too long, xargs is safer.

echo "$STAGED_FILES" | tr '\n' '\0' | xargs -0 veil scan --fail-on-findings --quiet

EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo "🛡️  Veil detected potential secrets in staged files."
    echo "   Please review the output above."
    echo "   Use '--no-verify' to bypass if this is a false positive (and consider adding a # veil:ignore comment)."
    exit 1
fi

exit 0
