use criterion::{black_box, criterion_group, criterion_main, Criterion};

const SAMPLE_PYTHON_CODE: &str = r#"
class ExampleClass:
    """A sample class for benchmarking."""
    
    def __init__(self, name: str, value: int = 0):
        self.name = name
        self.value = value
        self.history = []
    
    def process_data(self, data: list[dict]) -> dict:
        """Process a list of dictionaries."""
        result = {}
        for item in data:
            if 'key' in item and 'value' in item:
                result[item['key']] = item['value']
        return result
    
    def calculate_metrics(self) -> dict:
        """Calculate performance metrics."""
        if not self.history:
            return {"mean": 0, "max": 0, "min": 0}
        
        total = sum(self.history)
        return {
            "mean": total / len(self.history),
            "max": max(self.history),
            "min": min(self.history),
            "count": len(self.history)
        }

def complex_function(items: list) -> list:
    """A complex function with nested loops for benchmarking."""
    result = []
    for i, item in enumerate(items):
        if isinstance(item, dict):
            for key, value in item.items():
                if isinstance(value, list):
                    for sub_item in value:
                        result.append((i, key, sub_item))
                else:
                    result.append((i, key, value))
    return result
"#;

const SAMPLE_JAVASCRIPT_CODE: &str = r#"
class DataProcessor {
    constructor(name, options = {}) {
        this.name = name;
        this.options = {
            batchSize: 100,
            timeout: 5000,
            ...options
        };
        this.queue = [];
        this.processing = false;
    }
    
    async processItems(items) {
        const results = [];
        for (const item of items) {
            try {
                const processed = await this.transformItem(item);
                if (processed) {
                    results.push(processed);
                }
            } catch (error) {
                console.error(`Processing failed for item: ${item}`, error);
            }
        }
        return results;
    }
    
    transformItem(item) {
        return new Promise((resolve, reject) => {
            setTimeout(() => {
                if (item && typeof item === 'object') {
                    resolve({
                        ...item,
                        timestamp: Date.now(),
                        processed: true
                    });
                } else {
                    reject(new Error('Invalid item'));
                }
            }, Math.random() * 10);
        });
    }
}
"#;

fn benchmark_lexical_analysis(c: &mut Criterion) {
    c.bench_function("lexical_analysis", |b| {
        b.iter(|| {
            let code = black_box(SAMPLE_PYTHON_CODE);

            // Count different token types
            let keywords = code.matches("def ").count()
                + code.matches("class ").count()
                + code.matches("if ").count()
                + code.matches("for ").count()
                + code.matches("while ").count()
                + code.matches("return ").count();

            let operators = code.matches(" + ").count()
                + code.matches(" - ").count()
                + code.matches(" = ").count()
                + code.matches(" == ").count();

            let identifiers = code
                .split_whitespace()
                .filter(|word| word.chars().all(|c| c.is_alphanumeric() || c == '_'))
                .count();

            (keywords, operators, identifiers)
        });
    });
}

fn benchmark_syntax_analysis(c: &mut Criterion) {
    c.bench_function("syntax_analysis", |b| {
        b.iter(|| {
            let code = black_box(SAMPLE_JAVASCRIPT_CODE);

            // Analyze nesting depth
            let mut max_depth = 0usize;
            let mut current_depth = 0usize;

            for ch in code.chars() {
                match ch {
                    '{' | '(' | '[' => {
                        current_depth += 1;
                        max_depth = max_depth.max(current_depth);
                    }
                    '}' | ')' | ']' => {
                        current_depth = current_depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }

            // Count function declarations
            let function_count = code.matches("function ").count()
                + code.matches("async ").count()
                + code.matches("=> ").count();

            (max_depth, function_count)
        });
    });
}

fn benchmark_content_analysis(c: &mut Criterion) {
    c.bench_function("analyze_code_complexity", |b| {
        b.iter(|| {
            let python_complexity = black_box(
                SAMPLE_PYTHON_CODE.matches("for ").count()
                    + SAMPLE_PYTHON_CODE.matches("if ").count()
                    + SAMPLE_PYTHON_CODE.matches("while ").count(),
            );

            let js_complexity = black_box(
                SAMPLE_JAVASCRIPT_CODE.matches("for ").count()
                    + SAMPLE_JAVASCRIPT_CODE.matches("if ").count()
                    + SAMPLE_JAVASCRIPT_CODE.matches("while ").count(),
            );

            python_complexity + js_complexity
        });
    });
}

fn benchmark_string_operations(c: &mut Criterion) {
    c.bench_function("string_processing", |b| {
        b.iter(|| {
            let combined = format!(
                "{}\n{}",
                black_box(SAMPLE_PYTHON_CODE),
                black_box(SAMPLE_JAVASCRIPT_CODE)
            );

            let word_count = black_box(combined.split_whitespace().count());
            let line_count = black_box(combined.lines().count());
            let char_count = black_box(combined.chars().count());

            (word_count, line_count, char_count)
        });
    });
}

fn benchmark_pattern_matching(c: &mut Criterion) {
    c.bench_function("pattern_matching", |b| {
        b.iter(|| {
            let code = black_box(SAMPLE_PYTHON_CODE);

            // Simulate AST pattern matching
            let class_patterns = code
                .lines()
                .filter(|line| line.trim_start().starts_with("class "))
                .count();

            let function_patterns = code
                .lines()
                .filter(|line| line.trim_start().starts_with("def "))
                .count();

            let comment_patterns = code
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    trimmed.starts_with("#") || trimmed.starts_with("\"\"\"")
                })
                .count();

            (class_patterns, function_patterns, comment_patterns)
        });
    });
}

criterion_group!(
    benches,
    benchmark_lexical_analysis,
    benchmark_syntax_analysis,
    benchmark_content_analysis,
    benchmark_string_operations,
    benchmark_pattern_matching
);
criterion_main!(benches);
