#!/usr/bin/env python3
"""
Complexity Analysis Validation Script
Validates complexity analysis results for accuracy and completeness
"""

def validate_complexity_metrics(complexity_data):
    """Validate complexity metric calculations"""
    required_metrics = ['cyclomatic', 'cognitive', 'halstead']
    validation_results = {}
    
    for metric in required_metrics:
        if metric in complexity_data:
            metric_data = complexity_data[metric]
            validation_results[metric] = {
                'present': True,
                'has_total': 'total' in metric_data,
                'has_average': 'average' in metric_data,
                'has_distribution': 'distribution' in metric_data
            }
        else:
            validation_results[metric] = {'present': False}
    
    return validation_results

def validate_function_analysis(function_details):
    """Validate individual function complexity analysis"""
    if not function_details:
        return {'status': 'no_functions', 'quality_score': 0.8}
    
    valid_functions = 0
    complexity_ranges = {'simple': 0, 'moderate': 0, 'complex': 0, 'very_complex': 0}
    
    for func in function_details:
        required_fields = ['name', 'complexity_score', 'file_path', 'line_number']
        if all(field in func for field in required_fields):
            valid_functions += 1
            
            # Categorize complexity
            complexity = func.get('complexity_score', 0)
            if complexity <= 5:
                complexity_ranges['simple'] += 1
            elif complexity <= 10:
                complexity_ranges['moderate'] += 1
            elif complexity <= 20:
                complexity_ranges['complex'] += 1
            else:
                complexity_ranges['very_complex'] += 1
    
    quality_score = valid_functions / len(function_details)
    
    return {
        'status': 'analyzed',
        'quality_score': quality_score,
        'total_functions': len(function_details),
        'valid_functions': valid_functions,
        'complexity_distribution': complexity_ranges
    }

def validate_threshold_compliance(function_details, warning_threshold=10, error_threshold=20):
    """Validate compliance with complexity thresholds"""
    if not function_details:
        return {'compliant': True, 'violations': []}
    
    violations = {'warning': [], 'error': []}
    
    for func in function_details:
        complexity = func.get('complexity_score', 0)
        func_name = func.get('name', 'unknown')
        
        if complexity > error_threshold:
            violations['error'].append({
                'function': func_name,
                'complexity': complexity,
                'threshold_exceeded': error_threshold
            })
        elif complexity > warning_threshold:
            violations['warning'].append({
                'function': func_name,
                'complexity': complexity,
                'threshold_exceeded': warning_threshold
            })
    
    total_violations = len(violations['warning']) + len(violations['error'])
    compliance_rate = 1.0 - (total_violations / len(function_details))
    
    return {
        'compliant': total_violations == 0,
        'compliance_rate': compliance_rate,
        'violations': violations,
        'total_violations': total_violations
    }

def validate_language_specific_analysis(complexity_data, language):
    """Validate language-specific complexity analysis features"""
    language_validations = {
        'python': ['class_complexity', 'method_complexity'],
        'javascript': ['async_complexity', 'jsx_complexity'],
        'rust': ['trait_complexity', 'closure_complexity'],
        'java': ['class_complexity', 'interface_complexity']
    }
    
    expected_features = language_validations.get(language.lower(), [])
    feature_coverage = {}
    
    for feature in expected_features:
        feature_coverage[feature] = feature in complexity_data
    
    coverage_score = sum(feature_coverage.values()) / len(expected_features) if expected_features else 1.0
    
    return {
        'language': language,
        'expected_features': expected_features,
        'feature_coverage': feature_coverage,
        'coverage_score': coverage_score
    }

def main():
    """Main validation function for complexity analysis results"""
    import json
    import sys
    
    if len(sys.argv) != 2:
        print("Usage: validate_complexity_analysis.py <response_json_file>")
        sys.exit(1)
    
    try:
        with open(sys.argv[1], 'r') as f:
            response = json.load(f)
        
        result = response.get('result', {})
        complexity_metrics = result.get('complexity_metrics', {})
        function_details = result.get('function_details', [])
        language = result.get('language', 'unknown')
        
        # Perform validations
        metrics_validation = validate_complexity_metrics(complexity_metrics)
        function_validation = validate_function_analysis(function_details)
        threshold_validation = validate_threshold_compliance(function_details)
        language_validation = validate_language_specific_analysis(result, language)
        
        # Calculate overall quality score
        metrics_score = sum(1 for m in metrics_validation.values() if m.get('present', False)) / len(metrics_validation)
        function_score = function_validation['quality_score']
        compliance_score = threshold_validation['compliance_rate']
        language_score = language_validation['coverage_score']
        
        overall_score = (metrics_score * 0.3) + (function_score * 0.3) + (compliance_score * 0.2) + (language_score * 0.2)
        
        # Generate validation report
        report = {
            'validation_status': 'PASS' if overall_score >= 0.8 else 'WARN' if overall_score >= 0.6 else 'FAIL',
            'overall_quality_score': overall_score,
            'metrics_validation': metrics_validation,
            'function_analysis': function_validation,
            'threshold_compliance': threshold_validation,
            'language_specific': language_validation,
            'recommendations': []
        }
        
        # Add recommendations
        if metrics_score < 0.8:
            report['recommendations'].append("Improve complexity metrics coverage")
        
        if function_score < 0.8:
            report['recommendations'].append("Enhance function-level complexity analysis detail")
        
        if threshold_validation['total_violations'] > 0:
            report['recommendations'].append(f"Address {threshold_validation['total_violations']} complexity threshold violations")
        
        print(json.dumps(report, indent=2))
        
    except Exception as e:
        print(f"Validation error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
