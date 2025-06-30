#!/bin/bash

# Implementation Completeness Check - Prevents placeholder code from being committed
# Based on lessons learned from MCP Test Harness audit where 60%+ of "complete" work was stubs

set -e

# Parse command line arguments
CHECK_STAGED=false
if [ "$1" = "--check-staged" ]; then
    CHECK_STAGED=true
    echo "🔍 Checking staged files for incomplete implementations..."
else
    echo "🔍 Checking for incomplete implementations..."
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track if we found any issues
ISSUES_FOUND=0

# Function to get files to check
get_files_to_check() {
    if [ "$CHECK_STAGED" = true ]; then
        # Only check staged .rs files
        git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' || true
    else
        # Check all .rs files in crates/ directory
        find crates/ -name "*.rs" -type f
    fi
}

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

# Get the list of files to check
FILES_TO_CHECK=$(get_files_to_check)

if [ -z "$FILES_TO_CHECK" ]; then
    echo -e "${GREEN}✅ No files to check${NC}"
    exit 0
fi

# Check for TODO/FIXME/placeholder patterns
echo "Checking for TODO/FIXME/placeholder patterns..."
TODO_RESULTS=$(echo "$FILES_TO_CHECK" | xargs grep -n "TODO\|FIXME\|placeholder\|stub implementation" 2>/dev/null || true)
if [ ! -z "$TODO_RESULTS" ]; then
    report_issue "Found TODO/FIXME/placeholder code" "$TODO_RESULTS"
    echo -e "${YELLOW}Complete all implementations before committing.${NC}"
    echo ""
    echo -e "${YELLOW}💡 For legitimate future work, use these structured markers instead:${NC}"
    echo -e "${YELLOW}   FUTURE: For planned enhancements${NC}"
    echo -e "${YELLOW}   NOTE: For design documentation${NC}"
    echo -e "${YELLOW}   ENHANCEMENT: For performance improvements${NC}"
    echo -e "${YELLOW}   PLANNED(#123): For tracked future work (with issue number)${NC}"
    echo -e "${YELLOW}   CONSIDER: For alternative approaches${NC}"
    echo -e "${YELLOW}   TODO(#123): For TODOs linked to specific GitHub issues${NC}"
    echo ""
    echo -e "${YELLOW}See .cursor/rules/comment-standards.md for detailed guidelines.${NC}"
    echo ""
fi

# Check for unimplemented/todo macros
echo "Checking for unimplemented!/todo!() macros..."
UNIMPL_RESULTS=$(echo "$FILES_TO_CHECK" | xargs grep -n "unimplemented!\|todo!()" 2>/dev/null || true)
if [ ! -z "$UNIMPL_RESULTS" ]; then
    report_issue "Found unimplemented!() or todo!() macros" "$UNIMPL_RESULTS"
    echo -e "${YELLOW}Replace all unimplemented!() and todo!() with actual implementations.${NC}"
    echo ""
fi

# Check for placeholder tests
echo "Checking for placeholder tests..."
if [ "$CHECK_STAGED" = true ]; then
    # For staged files, only check test files
    TEST_FILES=$(echo "$FILES_TO_CHECK" | grep "/tests/" || true)
    if [ ! -z "$TEST_FILES" ]; then
        PLACEHOLDER_TESTS=$(echo "$TEST_FILES" | xargs grep -n "assert!(true)" 2>/dev/null || true)
    else
        PLACEHOLDER_TESTS=""
    fi
else
    # For full check, check both tests/ and crates/*/tests/
    PLACEHOLDER_TESTS=$(grep -r "assert!(true)" tests/ crates/*/tests/ --include="*.rs" -n 2>/dev/null || true)
fi

if [ ! -z "$PLACEHOLDER_TESTS" ]; then
    report_issue "Found placeholder tests with assert!(true)" "$PLACEHOLDER_TESTS"
    echo -e "${YELLOW}Replace placeholder tests with actual functionality verification.${NC}"
    echo ""
fi

# Check for suspicious placeholder patterns
echo "Checking for suspicious placeholder comments..."
SUSPICIOUS_PATTERNS=$(echo "$FILES_TO_CHECK" | xargs grep -n "// For now\|Placeholder.*would\|Mock.*TODO" 2>/dev/null || true)
if [ ! -z "$SUSPICIOUS_PATTERNS" ]; then
    report_issue "Found suspicious placeholder comments" "$SUSPICIOUS_PATTERNS"
    echo -e "${YELLOW}Remove all placeholder comments and implement actual functionality.${NC}"
    echo ""
fi

# Check for common stub function patterns
echo "Checking for stub function patterns..."
STUB_FUNCTIONS=$(echo "$FILES_TO_CHECK" | xargs grep -n "return.*mock\|return.*stub\|return.*placeholder" 2>/dev/null || true)
if [ ! -z "$STUB_FUNCTIONS" ]; then
    report_issue "Found stub function implementations" "$STUB_FUNCTIONS"
    echo -e "${YELLOW}Replace stub functions with actual implementations.${NC}"
    echo ""
fi

# Check for disabled/ignored tests that might hide incomplete work
echo "Checking for ignored tests..."
if [ "$CHECK_STAGED" = true ]; then
    # For staged files, only check the staged files
    IGNORED_TESTS=$(echo "$FILES_TO_CHECK" | xargs grep -n "#\[ignore\]" 2>/dev/null || true)
else
    # For full check, check both tests/ and crates/
    IGNORED_TESTS=$(grep -r "#\[ignore\]" crates/ tests/ --include="*.rs" -n 2>/dev/null || true)
fi

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