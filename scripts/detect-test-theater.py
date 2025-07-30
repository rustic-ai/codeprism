#!/usr/bin/env python3
"""
Test Theater Detection Script

This script automatically detects test theater anti-patterns in Rust test files
and provides specific recommendations for improvement.

Usage:
    python scripts/detect-test-theater.py [--fix] [--report-only]
    
Options:
    --fix: Automatically create TODO issues for detected problems
    --report-only: Only generate report, don't check files
"""

import os
import re
import sys
import argparse
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass
from enum import Enum

class TheaterType(Enum):
    STRING_CONTAINS_ONLY = "string_contains_only"
    MEANINGLESS_ASSERTION = "meaningless_assertion"
    CONFIGURATION_ONLY = "configuration_only"
    COUNT_BASED_VALIDATION = "count_based_validation"
    NO_FUNCTION_CALL = "no_function_call"
    PLACEHOLDER_OUTPUT = "placeholder_output"
    TAUTOLOGICAL_ASSERTION = "tautological_assertion"
    MOCK_TESTING_ONLY = "mock_testing_only"

@dataclass
class TheaterIssue:
    file_path: str
    line_number: int
    line_content: str
    theater_type: TheaterType
    severity: str  # "high", "medium", "low"
    description: str
    recommendation: str

class TestTheaterDetector:
    """Detects test theater patterns in Rust test files."""
    
    def __init__(self):
        self.patterns = {
            # String contains patterns without context validation
            TheaterType.STRING_CONTAINS_ONLY: [
                (r'assert!\(\s*.*\.contains\(["\'][^"\']*tool called["\'][)\]]*\)', 
                 "Testing tool execution message instead of functionality"),
                (r'assert!\(\s*.*\.contains\(["\'][^"\']*Success["\'][)\]]*\)', 
                 "Testing success message instead of actual results"),
                (r'assert!\(\s*.*\.contains\(["\'][^"\']*✅["\'][)\]]*\)', 
                 "Testing status emoji instead of functionality"),
                (r'assert!\(\s*.*\.contains\(["\'][^"\']*Finished["\'][)\]]*\)', 
                 "Testing completion message instead of results"),
            ],
            
            # Meaningless assertions
            TheaterType.MEANINGLESS_ASSERTION: [
                (r'assert!\(true\)', "Always passes - meaningless assertion"),
                (r'assert!\(.*\.is_ok\(\)\)', "Only tests that no error occurred, not functionality"),
                (r'assert!\(.*\.len\(\)\s*>\s*0\)(?!.*&&)', "Only tests non-empty, not content quality"),
                (r'assert!\(.*\.is_some\(\)\)(?!\s*,)', "Only tests that value exists, not its content"),
                (r'assert!\(!.*\.is_empty\(\)\)', "Only tests non-empty, not content validity"),
            ],
            
            # Count-based validation without content check
            TheaterType.COUNT_BASED_VALIDATION: [
                (r'let.*count.*=.*\.matches\(["\'][^"\']*["\'][)\]]*\.count\(\)', 
                 "Counting string occurrences instead of validating functionality"),
                (r'assert_eq!\(.*\.count\(\),\s*\d+\)', 
                 "Asserting count without validating what was counted"),
            ],
            
            # Configuration/structure testing only
            TheaterType.CONFIGURATION_ONLY: [
                (r'assert_eq!\(.*\.len\(\),\s*\d+\)(?!.*validation|.*analysis)', 
                 "Testing structure size without content validation"),
                (r'assert!\(.*\.transport\s*==', 
                 "Testing configuration values without execution"),
                (r'assert!\(.*spec\..*\.is_some\(\)', 
                 "Testing configuration exists without using it"),
            ],
            
            # Tautological assertions
            TheaterType.TAUTOLOGICAL_ASSERTION: [
                (r'assert_eq!\((.*)\.len\(\),\s*\1\.len\(\)\)', 
                 "Tautological assertion - comparing value to itself"),
                (r'assert_eq!\((.*),\s*\1\)', 
                 "Tautological assertion - comparing value to itself"),
            ],
            
            # Missing function calls in tests
            TheaterType.NO_FUNCTION_CALL: [
                # This pattern is harder to detect with regex, will be done in analyze_test_function
            ],
            
            # Mock/placeholder testing without real functionality
            TheaterType.MOCK_TESTING_ONLY: [
                (r'\.mock\(\)|Mock::|create_mock', "Testing mock objects instead of real functionality"),
                (r'placeholder|stub|fake', "Using placeholder implementations in tests"),
                (r'todo!\(\)|unimplemented!\(\)', "Test calls unimplemented functionality"),
            ],
        }
    
    def find_test_files(self, root_dir: str = ".") -> List[str]:
        """Find all Rust test files."""
        test_files = []
        
        # Handle single file case
        if os.path.isfile(root_dir) and root_dir.endswith('.rs'):
            with open(root_dir, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                # Check if it's a test file
                if ('#[test]' in content or '#[tokio::test]' in content or 
                    '/tests/' in root_dir or root_dir.endswith('_test.rs') or 
                    os.path.basename(root_dir) == 'test.rs'):
                    test_files.append(root_dir)
            return test_files
        
        # Handle directory case
        for root, dirs, files in os.walk(root_dir):
            # Skip target directory
            if 'target' in dirs:
                dirs.remove('target')
            
            for file in files:
                if file.endswith('.rs'):
                    file_path = os.path.join(root, file)
                    with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                        content = f.read()
                        # Check if it's a test file
                        if ('#[test]' in content or '#[tokio::test]' in content or 
                            '/tests/' in file_path or file.endswith('_test.rs') or 
                            file == 'test.rs'):
                            test_files.append(file_path)
        
        return test_files
    
    def analyze_file(self, file_path: str) -> List[TheaterIssue]:
        """Analyze a single test file for theater patterns."""
        issues = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                lines = f.readlines()
        except Exception as e:
            print(f"Error reading {file_path}: {e}")
            return issues
        
        # Check each line for patterns
        for line_num, line in enumerate(lines, 1):
            line_stripped = line.strip()
            if not line_stripped or line_stripped.startswith('//'):
                continue
            
            # Check for each theater pattern
            for theater_type, patterns in self.patterns.items():
                for pattern, description in patterns:
                    if re.search(pattern, line):
                        severity = self.determine_severity(theater_type, line)
                        recommendation = self.get_recommendation(theater_type, line)
                        
                        issues.append(TheaterIssue(
                            file_path=file_path,
                            line_number=line_num,
                            line_content=line_stripped,
                            theater_type=theater_type,
                            severity=severity,
                            description=description,
                            recommendation=recommendation
                        ))
        
        # Analyze test functions for missing actual calls
        issues.extend(self.analyze_test_functions(file_path, lines))
        
        return issues
    
    def analyze_test_functions(self, file_path: str, lines: List[str]) -> List[TheaterIssue]:
        """Analyze test functions to detect missing actual function calls."""
        issues = []
        current_test = None
        test_content = []
        brace_level = 0
        
        for line_num, line in enumerate(lines, 1):
            line_stripped = line.strip()
            
            # Detect test function start
            if re.search(r'#\[(tokio::)?test\]', line):
                current_test = line_num
                test_content = []
                brace_level = 0
                continue
            
            if current_test is not None:
                test_content.append((line_num, line))
                
                # Track braces to find end of function
                brace_level += line.count('{')
                brace_level -= line.count('}')
                
                # End of test function
                if brace_level < 0 or (brace_level == 0 and '}' in line and len(test_content) > 1):
                    issues.extend(self.validate_test_function(file_path, current_test, test_content))
                    current_test = None
                    test_content = []
        
        return issues
    
    def validate_test_function(self, file_path: str, start_line: int, content: List[Tuple[int, str]]) -> List[TheaterIssue]:
        """Validate that a test function actually calls functionality."""
        issues = []
        
        # Combine all content
        full_content = '\n'.join([line for _, line in content])
        
        # Look for actual function calls
        has_call_tool = bool(re.search(r'\.call_tool\(', full_content))
        # More flexible await pattern to catch various async call patterns
        has_await_call = bool(re.search(r'\.await\b', full_content))
        # More specific execute patterns to avoid false matches
        has_execute_call = bool(re.search(r'\b(execute|run|process)\s*\(', full_content, re.IGNORECASE))
        # Additional patterns for function calls
        has_function_call = bool(re.search(r'\w+\.\w+\([^)]*\)', full_content))
        has_method_chain = bool(re.search(r'\w+\.\w+\(.*\)\.\w+', full_content))
        
        # Count assertions
        assertion_count = len(re.findall(r'assert[_!]', full_content))
        
        # Look for contains-only assertions
        contains_assertions = len(re.findall(r'assert!\([^)]*\.contains\(', full_content))
        
        # Check for test theater patterns
        if contains_assertions > 0 and not (has_call_tool or has_await_call or has_function_call):
            issues.append(TheaterIssue(
                file_path=file_path,
                line_number=start_line,
                line_content=f"Test function with {contains_assertions} contains assertions",
                theater_type=TheaterType.STRING_CONTAINS_ONLY,
                severity="high",
                description="Test uses contains assertions without calling actual functionality",
                recommendation="Replace contains assertions with actual function calls and result validation"
            ))
        
        # Only flag if no meaningful function calls are detected
        if assertion_count > 0 and not (has_call_tool or has_await_call or has_execute_call or has_function_call or has_method_chain):
            issues.append(TheaterIssue(
                file_path=file_path,
                line_number=start_line,
                line_content=f"Test function with {assertion_count} assertions",
                theater_type=TheaterType.NO_FUNCTION_CALL,
                severity="high",
                description="Test has assertions but doesn't appear to call any functionality",
                recommendation="Add actual function calls to test real behavior"
            ))
        
        return issues
    
    def determine_severity(self, theater_type: TheaterType, line: str) -> str:
        """Determine the severity of a theater issue."""
        if theater_type in [TheaterType.STRING_CONTAINS_ONLY, TheaterType.NO_FUNCTION_CALL]:
            return "high"
        elif theater_type in [TheaterType.MEANINGLESS_ASSERTION, TheaterType.COUNT_BASED_VALIDATION]:
            return "high"
        elif theater_type in [TheaterType.TAUTOLOGICAL_ASSERTION]:
            return "high"
        else:
            return "medium"
    
    def get_recommendation(self, theater_type: TheaterType, line: str) -> str:
        """Get specific recommendation for fixing the theater issue."""
        recommendations = {
            TheaterType.STRING_CONTAINS_ONLY: "Parse tool output as JSON and validate actual analysis results",
            TheaterType.MEANINGLESS_ASSERTION: "Replace with assertions that validate actual behavior",
            TheaterType.COUNT_BASED_VALIDATION: "Validate the content of what was counted, not just the count",
            TheaterType.CONFIGURATION_ONLY: "Actually execute functionality using the configuration",
            TheaterType.TAUTOLOGICAL_ASSERTION: "Compare against expected values, not the same value",
            TheaterType.NO_FUNCTION_CALL: "Add calls to actual functions/tools being tested",
            TheaterType.PLACEHOLDER_OUTPUT: "Use real test data and validate actual output content",
            TheaterType.MOCK_TESTING_ONLY: "Replace mocks with actual implementation testing or integration tests"
        }
        return recommendations.get(theater_type, "Review and improve test validation")

def generate_report(issues: List[TheaterIssue]) -> str:
    """Generate a comprehensive report of test theater issues."""
    if not issues:
        return "🎉 No test theater patterns detected! All tests appear to validate real functionality.\n"
    
    # Group issues by file and severity
    files_with_issues = {}
    severity_counts = {"high": 0, "medium": 0, "low": 0}
    
    for issue in issues:
        if issue.file_path not in files_with_issues:
            files_with_issues[issue.file_path] = []
        files_with_issues[issue.file_path].append(issue)
        severity_counts[issue.severity] += 1
    
    report = ["🚨 Test Theater Detection Report", "=" * 50, ""]
    
    # Summary
    report.extend([
        f"📊 **Summary:**",
        f"- Total issues found: {len(issues)}",
        f"- High severity: {severity_counts['high']}",
        f"- Medium severity: {severity_counts['medium']}",
        f"- Low severity: {severity_counts['low']}",
        f"- Files affected: {len(files_with_issues)}",
        ""
    ])
    
    # Issues by file
    for file_path in sorted(files_with_issues.keys()):
        file_issues = files_with_issues[file_path]
        report.extend([
            f"📁 **{file_path}** ({len(file_issues)} issues)",
            "-" * (len(file_path) + 20)
        ])
        
        for issue in sorted(file_issues, key=lambda x: x.line_number):
            severity_emoji = {"high": "🔴", "medium": "🟡", "low": "🟢"}[issue.severity]
            report.extend([
                f"{severity_emoji} **Line {issue.line_number}**: {issue.theater_type.value}",
                f"   📋 Description: {issue.description}",
                f"   💡 Recommendation: {issue.recommendation}",
                f"   📝 Code: `{issue.line_content}`",
                ""
            ])
    
    # Recommendations summary
    report.extend([
        "🛠️ **Quick Fix Guidelines:**",
        "",
        "1. **Replace string contains assertions:**",
        "   ```rust",
        "   // ❌ BAD",
        "   assert!(stdout.contains(\"tool called\"));",
        "   ",
        "   // ✅ GOOD",
        "   let result = parse_tool_output(&stdout);",
        "   assert!(result.analysis.complexity > 0);",
        "   ```",
        "",
        "2. **Add actual function calls:**",
        "   ```rust",
        "   // ❌ BAD",
        "   let server = create_server();",
        "   assert!(server.is_ok());",
        "   ",
        "   // ✅ GOOD",
        "   let server = create_server()?;",
        "   let result = server.call_tool(\"analyze\", params).await?;",
        "   assert_eq!(result.analysis.metrics.complexity, expected);",
        "   ```",
        "",
        "3. **Validate actual output content:**",
        "   ```rust",
        "   // ❌ BAD",
        "   assert_eq!(results.len(), 5);",
        "   ",
        "   // ✅ GOOD",
        "   assert_eq!(results.len(), 5);",
        "   assert!(results.iter().all(|r| r.score > 0.0));",
        "   assert!(results.iter().any(|r| r.contains_vulnerability));",
        "   ```",
        ""
    ])
    
    return "\n".join(report)

def main():
    parser = argparse.ArgumentParser(description="Detect test theater patterns in Rust test files")
    parser.add_argument("--fix", action="store_true", help="Create TODO issues for detected problems")
    parser.add_argument("--report-only", action="store_true", help="Only generate report")
    parser.add_argument("--output", help="Output file for report (default: stdout)")
    parser.add_argument("path", nargs="?", default=".", help="Path to analyze (default: current directory)")
    
    args = parser.parse_args()
    
    detector = TestTheaterDetector()
    
    print("🔍 Scanning for test theater patterns...")
    test_files = detector.find_test_files(args.path)
    print(f"Found {len(test_files)} test files to analyze")
    
    all_issues = []
    for file_path in test_files:
        print(f"Analyzing: {file_path}")
        issues = detector.analyze_file(file_path)
        all_issues.extend(issues)
    
    # Generate report
    report = generate_report(all_issues)
    
    if args.output:
        with open(args.output, 'w') as f:
            f.write(report)
        print(f"Report saved to {args.output}")
    else:
        print("\n" + report)
    
    # Exit with error code if high severity issues found
    high_severity_count = sum(1 for issue in all_issues if issue.severity == "high")
    if high_severity_count > 0:
        print(f"\n❌ Found {high_severity_count} high severity test theater issues")
        sys.exit(1)
    else:
        print("\n✅ No high severity test theater issues found")
        sys.exit(0)

if __name__ == "__main__":
    main() 