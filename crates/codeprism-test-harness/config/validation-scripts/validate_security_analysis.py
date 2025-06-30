#!/usr/bin/env python3
"""
Security Analysis Validation Script
Validates security analysis results against OWASP standards and CWE mappings
"""

def validate_owasp_compliance(vulnerabilities):
    """Validate OWASP Top 10 2021 compliance"""
    owasp_2021_categories = [
        'A01:2021-Broken Access Control',
        'A02:2021-Cryptographic Failures', 
        'A03:2021-Injection',
        'A04:2021-Insecure Design',
        'A05:2021-Security Misconfiguration',
        'A06:2021-Vulnerable and Outdated Components',
        'A07:2021-Identification and Authentication Failures',
        'A08:2021-Software and Data Integrity Failures',
        'A09:2021-Security Logging and Monitoring Failures',
        'A10:2021-Server-Side Request Forgery'
    ]
    
    mapped_categories = []
    valid_mappings = 0
    
    for vuln in vulnerabilities:
        owasp_category = vuln.get('owasp_category', '')
        if any(category in owasp_category for category in owasp_2021_categories):
            mapped_categories.append(owasp_category)
            valid_mappings += 1
    
    return {
        'total_vulnerabilities': len(vulnerabilities),
        'owasp_mapped': valid_mappings,
        'mapping_percentage': (valid_mappings / len(vulnerabilities)) * 100 if vulnerabilities else 0,
        'categories_found': list(set(mapped_categories))
    }

def validate_cwe_mappings(vulnerabilities):
    """Validate CWE (Common Weakness Enumeration) mappings"""
    cwe_mapped = 0
    cwe_ids = []
    
    for vuln in vulnerabilities:
        cwe_id = vuln.get('cwe_id', '')
        if cwe_id and cwe_id.startswith('CWE-'):
            cwe_mapped += 1
            cwe_ids.append(cwe_id)
    
    return {
        'cwe_mapped_count': cwe_mapped,
        'cwe_mapping_percentage': (cwe_mapped / len(vulnerabilities)) * 100 if vulnerabilities else 0,
        'unique_cwe_ids': list(set(cwe_ids))
    }

def validate_severity_distribution(vulnerabilities):
    """Validate severity level distribution"""
    severity_counts = {'critical': 0, 'high': 0, 'medium': 0, 'low': 0, 'info': 0}
    
    for vuln in vulnerabilities:
        severity = vuln.get('severity', 'unknown').lower()
        if severity in severity_counts:
            severity_counts[severity] += 1
    
    return severity_counts

def main():
    """Main validation function for security analysis results"""
    import json
    import sys
    
    if len(sys.argv) != 2:
        print("Usage: validate_security_analysis.py <response_json_file>")
        sys.exit(1)
    
    try:
        with open(sys.argv[1], 'r') as f:
            response = json.load(f)
        
        result = response.get('result', {})
        vulnerabilities = result.get('vulnerabilities', [])
        
        # Perform validations
        owasp_validation = validate_owasp_compliance(vulnerabilities)
        cwe_validation = validate_cwe_mappings(vulnerabilities)
        severity_validation = validate_severity_distribution(vulnerabilities)
        
        # Generate validation report
        report = {
            'validation_status': 'PASS' if owasp_validation['mapping_percentage'] >= 70 else 'WARN',
            'owasp_compliance': owasp_validation,
            'cwe_mapping': cwe_validation,
            'severity_distribution': severity_validation,
            'recommendations': []
        }
        
        # Add recommendations based on validation results
        if owasp_validation['mapping_percentage'] < 50:
            report['recommendations'].append("Improve OWASP category mapping coverage")
        
        if cwe_validation['cwe_mapping_percentage'] < 60:
            report['recommendations'].append("Enhance CWE ID assignment for vulnerabilities")
        
        print(json.dumps(report, indent=2))
        
    except Exception as e:
        print(f"Validation error: {e}", file=sys.stderr)
        sys.exit(1)

if __name__ == "__main__":
    main()
