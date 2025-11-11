#!/usr/bin/env bash
# Automated UI Test Script for cim-keys
# This script performs automated checks that don't require GUI interaction

set -e

echo "==================================="
echo "CIM Keys - Automated UI Test Suite"
echo "==================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
PASSED=0
FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

info() {
    echo -e "${YELLOW}ℹ${NC} $1"
}

# Test 1: Build verification
echo "Test 1: Build Verification"
echo "----------------------------"
if cargo build --features gui 2>&1 | grep -q "Finished"; then
    pass "GUI application compiles successfully"
else
    fail "GUI application failed to compile"
fi
echo ""

# Test 2: Binary exists
echo "Test 2: Binary Verification"
echo "----------------------------"
if [ -f "./target/debug/cim-keys-gui" ]; then
    pass "GUI binary exists at target/debug/cim-keys-gui"

    # Check if binary is executable
    if [ -x "./target/debug/cim-keys-gui" ]; then
        pass "GUI binary is executable"
    else
        fail "GUI binary is not executable"
    fi
else
    fail "GUI binary not found"
fi
echo ""

# Test 3: Dependencies check
echo "Test 3: Dependency Verification"
echo "--------------------------------"
if cargo tree --features gui | grep -q "iced v0.13"; then
    pass "Iced 0.13 framework present"
else
    fail "Iced 0.13 framework missing"
fi

if cargo tree --features gui | grep -q "cim-domain"; then
    pass "cim-domain dependency present"
else
    fail "cim-domain dependency missing"
fi

if cargo tree --features gui | grep -q "tokio"; then
    pass "Tokio async runtime present"
else
    fail "Tokio async runtime missing"
fi
echo ""

# Test 4: Source file verification
echo "Test 4: Source File Verification"
echo "---------------------------------"
REQUIRED_FILES=(
    "src/gui.rs"
    "src/gui/graph.rs"
    "src/gui/event_emitter.rs"
    "src/gui/cowboy_theme.rs"
    "src/gui/firefly_renderer.rs"
    "src/mvi.rs"
    "src/mvi/model.rs"
    "src/mvi/intent.rs"
    "src/mvi/update.rs"
)

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$file" ]; then
        pass "Source file exists: $file"
    else
        fail "Source file missing: $file"
    fi
done
echo ""

# Test 5: Test data verification
echo "Test 5: Test Data Verification"
echo "-------------------------------"
if [ -f "examples/bootstrap-config.json" ]; then
    pass "Bootstrap config example exists"

    # Validate JSON
    if command -v jq &> /dev/null; then
        if jq empty examples/bootstrap-config.json 2>/dev/null; then
            pass "Bootstrap config is valid JSON"

            # Check for required fields
            if jq -e '.organization' examples/bootstrap-config.json > /dev/null; then
                pass "Bootstrap config has organization field"
            else
                fail "Bootstrap config missing organization field"
            fi

            if jq -e '.people' examples/bootstrap-config.json > /dev/null; then
                pass "Bootstrap config has people field"

                PEOPLE_COUNT=$(jq '.people | length' examples/bootstrap-config.json)
                info "Bootstrap config contains $PEOPLE_COUNT people"
            else
                fail "Bootstrap config missing people field"
            fi
        else
            fail "Bootstrap config is not valid JSON"
        fi
    else
        info "jq not available, skipping JSON validation"
    fi
else
    fail "Bootstrap config example not found"
fi
echo ""

# Test 6: Output directory test
echo "Test 6: Output Directory Test"
echo "------------------------------"
TEST_DIR="/tmp/cim-keys-test-$$"
mkdir -p "$TEST_DIR"
if [ -d "$TEST_DIR" ]; then
    pass "Test output directory created: $TEST_DIR"

    # Test write permissions
    if touch "$TEST_DIR/test.txt" 2>/dev/null; then
        pass "Test output directory is writable"
        rm "$TEST_DIR/test.txt"
    else
        fail "Test output directory is not writable"
    fi
else
    fail "Failed to create test output directory"
fi
echo ""

# Test 7: Module structure verification
echo "Test 7: Module Structure Verification"
echo "--------------------------------------"
if grep -q "pub mod gui;" src/lib.rs 2>/dev/null; then
    pass "GUI module exported in lib.rs"
else
    fail "GUI module not exported in lib.rs"
fi

if grep -q "pub mod mvi;" src/lib.rs 2>/dev/null; then
    pass "MVI module exported in lib.rs"
else
    fail "MVI module not exported in lib.rs"
fi

if grep -q "pub mod commands;" src/lib.rs 2>/dev/null; then
    pass "Commands module exported in lib.rs"
else
    fail "Commands module not exported in lib.rs"
fi
echo ""

# Test 8: Feature flags
echo "Test 8: Feature Flag Verification"
echo "----------------------------------"
if grep -q 'gui = \["iced"' Cargo.toml; then
    pass "GUI feature flag defined in Cargo.toml"
else
    fail "GUI feature flag not defined"
fi

if grep -q 'default = \["gui"' Cargo.toml; then
    pass "GUI included in default features"
else
    info "GUI not in default features (optional)"
fi
echo ""

# Test 9: Clippy lints (if available)
echo "Test 9: Code Quality Checks"
echo "----------------------------"
if command -v cargo-clippy &> /dev/null; then
    info "Running clippy..."
    CLIPPY_OUTPUT=$(cargo clippy --features gui 2>&1 || true)
    ERROR_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "error:" || true)
    WARNING_COUNT=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || true)

    if [ "$ERROR_COUNT" -eq 0 ]; then
        pass "No clippy errors found"
    else
        fail "Clippy found $ERROR_COUNT errors"
    fi

    info "Clippy found $WARNING_COUNT warnings (non-critical)"
else
    info "Clippy not available, skipping code quality checks"
fi
echo ""

# Test 10: Documentation verification
echo "Test 10: Documentation Verification"
echo "------------------------------------"
DOC_FILES=(
    "UI_TEST_PLAN.md"
    "UI_TEST_EXECUTION.md"
    "CLAUDE.md"
)

for doc in "${DOC_FILES[@]}"; do
    if [ -f "$doc" ]; then
        pass "Documentation exists: $doc"
    else
        fail "Documentation missing: $doc"
    fi
done
echo ""

# Test 11: MVI Architecture validation
echo "Test 11: MVI Architecture Validation"
echo "-------------------------------------"
if grep -q "pub enum Intent" src/mvi/intent.rs 2>/dev/null; then
    pass "Intent enum defined"
else
    fail "Intent enum not found"
fi

if grep -q "pub struct Model" src/mvi/model.rs 2>/dev/null; then
    pass "Model struct defined"
else
    fail "Model struct not found"
fi

if grep -q "pub fn update" src/mvi/update.rs 2>/dev/null; then
    pass "Update function defined"
else
    fail "Update function not found"
fi

# Check for pure function pattern (no mutable references in update)
if grep -q "fn update.*&mut" src/mvi/update.rs 2>/dev/null; then
    fail "Update function has mutable reference (should be pure)"
else
    pass "Update function is pure (no mutable references)"
fi
echo ""

# Test 12: Event-sourcing verification
echo "Test 12: Event-Sourcing Verification"
echo "-------------------------------------"
if grep -q "correlation_id" src/events.rs 2>/dev/null; then
    pass "Events have correlation_id field"
else
    fail "Events missing correlation_id field"
fi

if grep -q "causation_id" src/events.rs 2>/dev/null; then
    pass "Events have causation_id field"
else
    fail "Events missing causation_id field"
fi

if grep -q "Uuid::now_v7()" src/gui.rs 2>/dev/null; then
    pass "GUI uses UUID v7 for time-ordered IDs"
else
    info "Could not verify UUID v7 usage in GUI"
fi
echo ""

# Summary
echo "=================================="
echo "Test Summary"
echo "=================================="
TOTAL=$((PASSED + FAILED))
echo -e "Total Tests: $TOTAL"
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All automated tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run manual UI tests with: cargo run --features gui -- /tmp/cim-keys-ui-test"
    echo "2. Follow UI_TEST_EXECUTION.md for comprehensive manual testing"
    echo "3. Use UI_TEST_PLAN.md as detailed test reference"
    echo ""
    exit 0
else
    echo -e "${RED}✗ Some tests failed. Please review and fix issues.${NC}"
    echo ""
    exit 1
fi
