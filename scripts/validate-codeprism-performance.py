#!/usr/bin/env python3
"""
CodePrism Performance Validation Script

Validates performance requirements for CodePrism moth specifications
as defined in Issue #235.
"""

import json
import sys
import argparse
from datetime import datetime
from typing import Dict, List, Tuple, Optional


class PerformanceValidator:
    """Validates CodePrism performance requirements"""
    
    def __init__(self):
        """Initialize with performance requirements from Issue #235"""
        self.requirements = {
            'tool_execution_time_ms': 3000,        # Most tools <3s
            'complex_analysis_time_ms': 5500,      # Complex analysis <5.5s
            'memory_usage_mb': 60,                 # Standard tools <60MB
            'complex_memory_mb': 88,               # Complex analysis <88MB
            'total_execution_time_s': 120,         # Total suite <2 minutes
            'success_rate_threshold': 100.0,       # 100% success rate required
        }
        
        # Expected test counts per language (from Issue #231)
        self.expected_counts = {
            'rust': 18,
            'python': 18,
            'java': 18,
            'javascript': 17
        }
        
        # Tool categories for performance classification
        self.complex_tools = {
            'analyze_code_quality',
            'analyze_dependencies',
            'analyze_security_vulnerabilities',
            'analyze_performance_bottlenecks',
            'check_code_complexity',
            'analyze_architecture_patterns',
            'detect_duplicate_code',
            'security_audit',
            'performance_analysis',
            'dependency_analysis',
            'complexity_analysis',
            'architecture_analysis'
        }
    
    def validate_performance_requirements(self, results_file: str, language: str) -> bool:
        """
        Validate CodePrism performance requirements for a given language
        
        Args:
            results_file: Path to JSON results file
            language: Language being tested (rust, python, java, javascript)
            
        Returns:
            bool: True if all requirements are met
        """
        try:
            with open(results_file, 'r') as f:
                results = json.load(f)
            
            print(f"📊 Performance Validation for {language.upper()}")
            print(f"Results file: {results_file}")
            print(f"Timestamp: {datetime.now().isoformat()}")
            print("-" * 60)
            
            # Validate test counts
            if not self._validate_test_counts(results, language):
                return False
            
            # Validate success rate
            if not self._validate_success_rate(results, language):
                return False
            
            # Validate execution time
            if not self._validate_execution_time(results, language):
                return False
            
            # Validate individual test performance
            if not self._validate_individual_test_performance(results, language):
                return False
            
            # Validate memory usage if available
            if not self._validate_memory_usage(results, language):
                return False
            
            print(f"✅ All performance requirements met for {language}")
            print(f"✅ CodePrism {language} comprehensive validation PASSED")
            return True
            
        except Exception as e:
            print(f"❌ Performance validation failed: {e}")
            return False
    
    def _validate_test_counts(self, results: Dict, language: str) -> bool:
        """Validate that the expected number of tests were run"""
        total_tests = results.get('total_tests', 0)
        expected_total = self.expected_counts.get(language, 18)
        
        print(f"📈 Test Count Validation:")
        print(f"  Total Tests: {total_tests}")
        print(f"  Expected: {expected_total}")
        
        if total_tests != expected_total:
            print(f"  ❌ Test count mismatch: expected {expected_total}, got {total_tests}")
            return False
        
        print(f"  ✅ Test count matches expectation")
        return True
    
    def _validate_success_rate(self, results: Dict, language: str) -> bool:
        """Validate that all tests passed"""
        total_tests = results.get('total_tests', 0)
        passed_tests = results.get('passed', 0)
        failed_tests = results.get('failed', 0)
        
        if total_tests == 0:
            print(f"  ❌ No tests were run")
            return False
        
        success_rate = (passed_tests / total_tests) * 100
        
        print(f"📊 Success Rate Validation:")
        print(f"  Passed: {passed_tests}/{total_tests}")
        print(f"  Failed: {failed_tests}")
        print(f"  Success Rate: {success_rate:.1f}%")
        print(f"  Required: {self.requirements['success_rate_threshold']:.1f}%")
        
        if success_rate < self.requirements['success_rate_threshold']:
            print(f"  ❌ Success rate below threshold: {success_rate:.1f}% < {self.requirements['success_rate_threshold']:.1f}%")
            return False
        
        print(f"  ✅ Success rate meets requirement")
        return True
    
    def _validate_execution_time(self, results: Dict, language: str) -> bool:
        """Validate total execution time"""
        total_duration = results.get('total_duration', {})
        duration_seconds = total_duration.get('secs', 0) + total_duration.get('nanos', 0) / 1e9
        
        print(f"⏱️  Execution Time Validation:")
        print(f"  Total Duration: {duration_seconds:.1f}s")
        print(f"  Maximum Allowed: {self.requirements['total_execution_time_s']}s")
        
        if duration_seconds > self.requirements['total_execution_time_s']:
            print(f"  ❌ Execution time too slow: {duration_seconds:.1f}s > {self.requirements['total_execution_time_s']}s")
            return False
        
        print(f"  ✅ Execution time within limits")
        return True
    
    def _validate_individual_test_performance(self, results: Dict, language: str) -> bool:
        """Validate individual test performance requirements"""
        test_results = results.get('test_results', [])
        
        if not test_results:
            print(f"  ❌ No individual test results available")
            return False
        
        print(f"🔍 Individual Test Performance Validation:")
        print(f"  Analyzing {len(test_results)} test results...")
        
        slow_tests = []
        performance_stats = {
            'standard_tests': [],
            'complex_tests': [],
            'violations': []
        }
        
        for test in test_results:
            test_name = test.get('test_name', '')
            test_duration = test.get('duration', {})
            test_duration_ms = test_duration.get('secs', 0) * 1000 + test_duration.get('nanos', 0) / 1e6
            
            # Classify test as standard or complex
            is_complex = self._is_complex_test(test_name)
            max_duration = self.requirements['complex_analysis_time_ms'] if is_complex else self.requirements['tool_execution_time_ms']
            
            if is_complex:
                performance_stats['complex_tests'].append(test_duration_ms)
            else:
                performance_stats['standard_tests'].append(test_duration_ms)
            
            if test_duration_ms > max_duration:
                slow_tests.append((test_name, test_duration_ms, max_duration, is_complex))
                performance_stats['violations'].append({
                    'test_name': test_name,
                    'duration_ms': test_duration_ms,
                    'max_allowed_ms': max_duration,
                    'is_complex': is_complex
                })
        
        # Report performance statistics
        self._report_performance_stats(performance_stats)
        
        if slow_tests:
            print(f"  ❌ {len(slow_tests)} tests exceeded performance requirements:")
            for test_name, duration, max_duration, is_complex in slow_tests[:10]:  # Show first 10
                test_type = "complex" if is_complex else "standard"
                print(f"    - {test_name} ({test_type}): {duration:.0f}ms > {max_duration}ms")
            
            if len(slow_tests) > 10:
                print(f"    ... and {len(slow_tests) - 10} more slow tests")
            
            return False
        
        print(f"  ✅ All {len(test_results)} tests met performance requirements")
        return True
    
    def _validate_memory_usage(self, results: Dict, language: str) -> bool:
        """Validate memory usage requirements if available"""
        test_results = results.get('test_results', [])
        
        memory_violations = []
        
        for test in test_results:
            test_name = test.get('test_name', '')
            memory_usage = test.get('memory_usage_mb')
            
            if memory_usage is not None:
                is_complex = self._is_complex_test(test_name)
                max_memory = self.requirements['complex_memory_mb'] if is_complex else self.requirements['memory_usage_mb']
                
                if memory_usage > max_memory:
                    memory_violations.append((test_name, memory_usage, max_memory, is_complex))
        
        if memory_violations:
            print(f"💾 Memory Usage Validation:")
            print(f"  ❌ {len(memory_violations)} tests exceeded memory requirements:")
            for test_name, usage, max_usage, is_complex in memory_violations[:5]:
                test_type = "complex" if is_complex else "standard"
                print(f"    - {test_name} ({test_type}): {usage:.1f}MB > {max_usage}MB")
            return False
        
        print(f"💾 Memory Usage: ✅ All tests within limits (if measured)")
        return True
    
    def _is_complex_test(self, test_name: str) -> bool:
        """Determine if a test is considered complex analysis"""
        test_name_lower = test_name.lower()
        return any(keyword in test_name_lower for keyword in self.complex_tools)
    
    def _report_performance_stats(self, stats: Dict):
        """Report performance statistics"""
        print(f"  📈 Performance Statistics:")
        
        if stats['standard_tests']:
            avg_standard = sum(stats['standard_tests']) / len(stats['standard_tests'])
            max_standard = max(stats['standard_tests'])
            print(f"    Standard tests: {len(stats['standard_tests'])} tests, avg: {avg_standard:.0f}ms, max: {max_standard:.0f}ms")
        
        if stats['complex_tests']:
            avg_complex = sum(stats['complex_tests']) / len(stats['complex_tests'])
            max_complex = max(stats['complex_tests'])
            print(f"    Complex tests: {len(stats['complex_tests'])} tests, avg: {avg_complex:.0f}ms, max: {max_complex:.0f}ms")
        
        if stats['violations']:
            print(f"    Performance violations: {len(stats['violations'])}")
        else:
            print(f"    Performance violations: 0")
    
    def generate_performance_report(self, results_file: str, language: str, output_file: str = None):
        """Generate a detailed performance report"""
        try:
            with open(results_file, 'r') as f:
                results = json.load(f)
            
            report = {
                'language': language,
                'timestamp': datetime.now().isoformat(),
                'results_file': results_file,
                'validation_results': {},
                'performance_metrics': {},
                'recommendations': []
            }
            
            # Collect validation results
            report['validation_results'] = {
                'test_count_valid': self._validate_test_counts(results, language),
                'success_rate_valid': self._validate_success_rate(results, language),
                'execution_time_valid': self._validate_execution_time(results, language),
                'individual_performance_valid': self._validate_individual_test_performance(results, language),
                'memory_usage_valid': self._validate_memory_usage(results, language)
            }
            
            # Calculate performance metrics
            test_results = results.get('test_results', [])
            if test_results:
                durations = [
                    test.get('duration', {}).get('secs', 0) * 1000 + test.get('duration', {}).get('nanos', 0) / 1e6
                    for test in test_results
                ]
                
                report['performance_metrics'] = {
                    'total_tests': len(test_results),
                    'average_duration_ms': sum(durations) / len(durations) if durations else 0,
                    'max_duration_ms': max(durations) if durations else 0,
                    'min_duration_ms': min(durations) if durations else 0,
                    'median_duration_ms': sorted(durations)[len(durations)//2] if durations else 0,
                    'p95_duration_ms': sorted(durations)[int(len(durations)*0.95)] if durations else 0
                }
            
            # Generate recommendations
            if not all(report['validation_results'].values()):
                report['recommendations'].append("Address performance requirement violations")
            
            if report['performance_metrics'].get('average_duration_ms', 0) > 1000:
                report['recommendations'].append("Consider optimizing average test execution time")
            
            if output_file:
                with open(output_file, 'w') as f:
                    json.dump(report, f, indent=2)
                print(f"Performance report saved to: {output_file}")
            
            return report
            
        except Exception as e:
            print(f"❌ Failed to generate performance report: {e}")
            return None


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description='Validate CodePrism performance requirements')
    parser.add_argument('results_file', help='Path to JSON results file')
    parser.add_argument('language', choices=['rust', 'python', 'java', 'javascript'], 
                       help='Language being tested')
    parser.add_argument('--report', help='Generate detailed performance report to file')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    args = parser.parse_args()
    
    validator = PerformanceValidator()
    
    # Validate performance requirements
    success = validator.validate_performance_requirements(args.results_file, args.language)
    
    # Generate detailed report if requested
    if args.report:
        validator.generate_performance_report(args.results_file, args.language, args.report)
    
    if success:
        print(f"\n✅ Performance validation PASSED for {args.language}")
        sys.exit(0)
    else:
        print(f"\n❌ Performance validation FAILED for {args.language}")
        sys.exit(1)


if __name__ == '__main__':
    main() 