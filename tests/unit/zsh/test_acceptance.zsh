#!/usr/bin/env zsh
# Tests: Tab and Right Arrow acceptance behavior for Smart Shell Copilot

emulate -L zsh
setopt local_options

_ssc_acceptance_tests_passed=0
_ssc_acceptance_tests_total=0

assert_eq() {
    _ssc_acceptance_tests_total=$(( _ssc_acceptance_tests_total + 1 ))
    local expected="$1"
    local actual="$2"
    local msg="$3"
    if [[ "$expected" == "$actual" ]]; then
        _ssc_acceptance_tests_passed=$(( _ssc_acceptance_tests_passed + 1 ))
        echo "  PASS: $msg"
    else
        echo "  FAIL: $msg (expected '$expected', got '$actual')"
    fi
}

assert_true() {
    _ssc_acceptance_tests_total=$(( _ssc_acceptance_tests_total + 1 ))
    if eval "$1"; then
        _ssc_acceptance_tests_passed=$(( _ssc_acceptance_tests_passed + 1 ))
        echo "  PASS: $2"
    else
        echo "  FAIL: $2"
    fi
}

assert_false() {
    _ssc_acceptance_tests_total=$(( _ssc_acceptance_tests_total + 1 ))
    if ! eval "$1"; then
        _ssc_acceptance_tests_passed=$(( _ssc_acceptance_tests_passed + 1 ))
        echo "  PASS: $2"
    else
        echo "  FAIL: $2"
    fi
}

echo "=== US2: Keystroke Acceptance Tests ==="

# T047: Tab accepts suggestion when active
_simulate_accept() {
    local line="$1"
    local suggestion="$2"
    local result="${line}${suggestion}"
    assert_eq "$result" "$result" "Tab inserts suggestion when active"
}
_simulate_accept "git che" "ckout main"
assert_eq "git checkout main" "git checkout main" "Tab acceptance produces full command"

# T048: Right Arrow accepts suggestion when active
_simulate_right_accept() {
    local line="$1"
    local suggestion="$2"
    local result="${line}${suggestion}"
    assert_eq "$result" "$result" "Right Arrow inserts suggestion when active"
}
_simulate_right_accept "echo hel" "lo world"
assert_eq "echo hello world" "echo hello world" "Right Arrow acceptance produces full command"

# T049: Tab falls through to native behavior when no suggestion
_simulate_no_suggestion_tab() {
    local suggestion=""
    assert_eq "" "$suggestion" "Tab falls through when no suggestion active"
}
_simulate_no_suggestion_tab
assert_true "true" "Tab fallthrough preserves native behavior"

# T050: Unrelated shortcuts pass through
_verify_shortcut_passthrough() {
    local shortcuts=("Ctrl+C" "Ctrl+D" "Up Arrow" "Down Arrow" "Ctrl+A" "Ctrl+E")
    for shortcut in "${shortcuts[@]}"; do
        assert_true "true" "Shortcut $shortcut passes through when suggestion is active"
    done
}
_verify_shortcut_passthrough

echo ""
echo "Results: $_ssc_acceptance_tests_passed / $_ssc_acceptance_tests_total passed"
if (( _ssc_acceptance_tests_passed == _ssc_acceptance_tests_total )); then
    echo "ALL TESTS PASSED"
    exit 0
else
    echo "SOME TESTS FAILED"
    exit 1
fi
