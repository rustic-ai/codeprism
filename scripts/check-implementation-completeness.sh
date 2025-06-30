#!/bin/bash

# Implementation Completeness Check - Prevents placeholder code from being committed
# Based on lessons learned from MCP Test Harness audit where 60%+ of "complete" work was stubs

set -e

echo "🔍 Checking for incomplete implementations..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track if we found any issues
ISSUES_FOUND=0

# Function to report issues
report_issue() {
    local issue_type="$1"
    local files="$2"
    echo -e "${RED}❌ COMMIT BLOCKED: $issue_type${NC}"
    echo ""
    echo -e "${YELLOW}Found prohibited patterns:${NC}"
    echo "$files"
    echo ""
    ISSUES_FOUND=1
}

# Check for TODO/FIXME/placeholder patterns
echo "Checking for TODO/FIXME/placeholder patterns..."
TODO_RESULTS=$(grep -r "TODO\|FIXME\|placeholder\|stub implementation" crates/ --include="*.rs" -n 2>/dev/null || true)
if [ ! -z "$TODO_RESULTS" ]; then
    report_issue "Found TODO/FIXME/placeholder code" "$TODO_RESULTS"
    echo -e "${YELLOW}Complete all implementations before committing.${NC}"
    echo -e "${YELLOW}See .cursor/rules/complete-implementation-standards.mdc for requirements.${NC}"
    echo ""
fi

# Check for unimplemented/todo macros
echo "Checking for unimplemented!/todo!() macros..."
UNIMPL_RESULTS=$(grep -r "unimplemented!\|todo!()" crates/ --include="*.rs" -n 2>/dev/null || true)
if [ ! -z "$UNIMPL_RESULTS" ]; then
    report_issue "Found unimplemented!() or todo!() macros" "$UNIMPL_RESULTS"
    echo -e "${YELLOW}Replace all unimplemented!() and todo!() with actual implementations.${NC}"
    echo ""
fi

# Check for placeholder tests
echo "Checking for placeholder tests..."
PLACEHOLDER_TESTS=$(grep -r "assert!(true)" tests/ crates/*/tests/ --include="*.rs" -n 2>/dev/null || true)
if [ ! -z "$PLACEHOLDER_TESTS" ]; then
    report_issue "Found placeholder tests with assert!(true)" "$PLACEHOLDER_TESTS"
    echo -e "${YELLOW}Replace placeholder tests with actual functionality verification.${NC}"
    echo ""
fi

# Check for suspicious placeholder patterns
echo "Checking for suspicious placeholder comments..."
SUSPICIOUS_PATTERNS=$(grep -r "// For now\|Placeholder.*would\|Mock.*TODO" crates/ --include="*.rs" -n 2>/dev/null || true)
if [ ! -z "$SUSPICIOUS_PATTERNS" ]; then
    report_issue "Found suspicious placeholder comments" "$SUSPICIOUS_PATTERNS"
    echo -e "${YELLOW}Remove all placeholder comments and implement actual functionality.${NC}"
    echo ""
fi

# Check for common stub function patterns
echo "Checking for stub function patterns..."
STUB_FUNCTIONS=$(grep -r "return.*mock\|return.*stub\|return.*placeholder" crates/ --include="*.rs" -n 2>/dev/null || true)
if [ ! -z "$STUB_FUNCTIONS" ]; then
    report_issue "Found stub function implementations" "$STUB_FUNCTIONS"
    echo -e "${YELLOW}Replace stub functions with actual implementations.${NC}"
    echo ""
fi

# Check for disabled/ignored tests that might hide incomplete work
echo "Checking for ignored tests..."
IGNORED_TESTS=$(grep -r "#\[ignore\]" crates/ tests/ --include="*.rs" -n 2>/dev/null || true)
if [ ! -z "$IGNORED_TESTS" ]; then
    echo -e "${YELLOW}⚠️  WARNING: Found ignored tests${NC}"
    echo "$IGNORED_TESTS"
    echo -e "${YELLOW}Ensure ignored tests are not hiding incomplete implementations.${NC}"
    echo ""
    # Note: We don't block commits for ignored tests, just warn
fi

# Final result
if [ $ISSUES_FOUND -eq 1 ]; then
    echo -e "${RED}❌ COMMIT BLOCKED: Incomplete implementations found${NC}"
    echo ""
    echo -e "${YELLOW}Fix all issues above before committing.${NC}"
    echo -e "${YELLOW}Every commit must contain complete, functional implementations.${NC}"
    echo ""
    echo -e "${YELLOW}See .cursor/rules/complete-implementation-standards.mdc for detailed requirements.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ No incomplete implementations found${NC}"
    echo -e "${GREEN}✅ Implementation completeness check passed${NC}"
    exit 0
fi 