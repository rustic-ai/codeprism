#!/bin/bash

# Staged Files Implementation Completeness Check
# Only checks files that are staged for commit, not the entire codebase

set -e

echo "🔍 Checking staged files for incomplete implementations..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Track if we found any issues
ISSUES_FOUND=0

# Get list of staged .rs files
STAGED_FILES=$(git diff --staged --name-only --diff-filter=AM | grep '\.rs$' || true)

if [ -z "$STAGED_FILES" ]; then
    echo -e "${GREEN}✅ No staged Rust files to check${NC}"
    exit 0
fi

echo -e "${YELLOW}Checking staged files:${NC}"
echo "$STAGED_FILES" | sed 's/^/  - /'
echo ""

# Function to report issues
report_issue() {
    local issue_type="$1"
    local files="$2"
    echo -e "${RED}❌ COMMIT BLOCKED: $issue_type${NC}"
    echo ""
    echo -e "${YELLOW}Found prohibited patterns in staged files:${NC}"
    echo "$files"
    echo ""
    ISSUES_FOUND=1
}

# Check staged files for TODO/FIXME/placeholder patterns
echo "Checking staged files for TODO/FIXME/placeholder patterns..."
TODO_RESULTS=""
for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        file_results=$(grep -n "TODO\|FIXME\|placeholder\|stub implementation" "$file" 2>/dev/null || true)
        if [ ! -z "$file_results" ]; then
            TODO_RESULTS="${TODO_RESULTS}${file}:\n${file_results}\n"
        fi
    fi
done

if [ ! -z "$TODO_RESULTS" ]; then
    report_issue "Found TODO/FIXME/placeholder code in staged files" "$TODO_RESULTS"
    echo -e "${YELLOW}Complete all implementations before committing.${NC}"
    echo ""
fi

# Check staged files for unimplemented/todo macros
echo "Checking staged files for unimplemented!/todo!() macros..."
UNIMPL_RESULTS=""
for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        file_results=$(grep -n "unimplemented!\|todo!()" "$file" 2>/dev/null || true)
        if [ ! -z "$file_results" ]; then
            UNIMPL_RESULTS="${UNIMPL_RESULTS}${file}:\n${file_results}\n"
        fi
    fi
done

if [ ! -z "$UNIMPL_RESULTS" ]; then
    report_issue "Found unimplemented!() or todo!() macros in staged files" "$UNIMPL_RESULTS"
    echo -e "${YELLOW}Replace all unimplemented!() and todo!() with actual implementations.${NC}"
    echo ""
fi

# Check staged files for placeholder tests
echo "Checking staged files for placeholder tests..."
PLACEHOLDER_TESTS=""
for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        file_results=$(grep -n "assert!(true)" "$file" 2>/dev/null || true)
        if [ ! -z "$file_results" ]; then
            PLACEHOLDER_TESTS="${PLACEHOLDER_TESTS}${file}:\n${file_results}\n"
        fi
    fi
done

if [ ! -z "$PLACEHOLDER_TESTS" ]; then
    report_issue "Found placeholder tests with assert!(true) in staged files" "$PLACEHOLDER_TESTS"
    echo -e "${YELLOW}Replace placeholder tests with actual functionality verification.${NC}"
    echo ""
fi

# Check staged files for suspicious placeholder patterns
echo "Checking staged files for suspicious placeholder comments..."
SUSPICIOUS_PATTERNS=""
for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        file_results=$(grep -n "// For now\|Placeholder.*would\|Mock.*TODO" "$file" 2>/dev/null || true)
        if [ ! -z "$file_results" ]; then
            SUSPICIOUS_PATTERNS="${SUSPICIOUS_PATTERNS}${file}:\n${file_results}\n"
        fi
    fi
done

if [ ! -z "$SUSPICIOUS_PATTERNS" ]; then
    report_issue "Found suspicious placeholder comments in staged files" "$SUSPICIOUS_PATTERNS"
    echo -e "${YELLOW}Remove all placeholder comments and implement actual functionality.${NC}"
    echo ""
fi

# Check staged files for stub function patterns
echo "Checking staged files for stub function patterns..."
STUB_FUNCTIONS=""
for file in $STAGED_FILES; do
    if [ -f "$file" ]; then
        file_results=$(grep -n "return.*mock\|return.*stub\|return.*placeholder" "$file" 2>/dev/null || true)
        if [ ! -z "$file_results" ]; then
            STUB_FUNCTIONS="${STUB_FUNCTIONS}${file}:\n${file_results}\n"
        fi
    fi
done

if [ ! -z "$STUB_FUNCTIONS" ]; then
    report_issue "Found stub function implementations in staged files" "$STUB_FUNCTIONS"
    echo -e "${YELLOW}Replace stub functions with actual implementations.${NC}"
    echo ""
fi

# Final result
if [ $ISSUES_FOUND -eq 1 ]; then
    echo -e "${RED}❌ COMMIT BLOCKED: Incomplete implementations found in staged files${NC}"
    echo ""
    echo -e "${YELLOW}Fix all issues above before committing.${NC}"
    echo -e "${YELLOW}Every commit must contain complete, functional implementations.${NC}"
    echo ""
    exit 1
else
    echo -e "${GREEN}✅ No incomplete implementations found in staged files${NC}"
    echo -e "${GREEN}✅ Staged files implementation completeness check passed${NC}"
    exit 0
fi 