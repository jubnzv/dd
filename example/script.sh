#!/usr/bin/env bash
# Should return non-zero return code if test fails.
# ! grep -q -E "(assert\(false\)|require\(\"mod2\"\))" $1
# ! grep -q -E "assert\(false\)" $1
! grep -q -E "require\(\"mod2\"\)" $1

# ! grep -q -E "assert\(false\)" $1 || ! grep -q -E "require\(\"mod2\"\)" $1
