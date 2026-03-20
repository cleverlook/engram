#!/usr/bin/env bash
# Patch Homebrew formula to add shell completion generation.
# Called from .github/workflows/release.yml during publish-homebrew-formula.
#
# Usage: ./scripts/patch-homebrew-completions.sh Formula/engram.rb
#
# Adds generate_completions_from_executable(bin/"engram", "completion")
# after the install method's binary installation lines.

set -euo pipefail

for rb in "$@"; do
  if grep -q 'generate_completions_from_executable' "$rb"; then
    echo "Completions already present in $rb, skipping."
    continue
  fi

  if ! grep -q 'def install' "$rb"; then
    echo "Warning: no install method found in $rb, skipping."
    continue
  fi

  # Insert generate_completions_from_executable before the closing 'end' of install method
  # Strategy: find 'install_binary_aliases!' or last line before first 'end' after 'def install'
  if grep -q 'install_binary_aliases!' "$rb"; then
    sed -i 's/install_binary_aliases!/install_binary_aliases!\n\n    generate_completions_from_executable(bin\/"engram", "completion")/' "$rb"
  else
    # Fallback: add before the first 'end' after 'def install'
    python3 -c "
import sys
with open(sys.argv[1]) as f:
    lines = f.readlines()
in_install = False
inserted = False
result = []
for line in lines:
    if 'def install' in line:
        in_install = True
    if in_install and not inserted and line.strip() == 'end':
        result.append('    generate_completions_from_executable(bin/\"engram\", \"completion\")\n')
        inserted = True
    result.append(line)
with open(sys.argv[1], 'w') as f:
    f.writelines(result)
" "$rb"
  fi

  echo "Patched $rb with shell completions."
done
