// Binary to verify ValidationScript GREEN phase implementation
// This demonstrates our enhanced data structure works correctly

use mandrel_mcp_th::spec::{ExecutionPhase, ScriptLanguage, ValidationScript};
use serde_json;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TestSpec {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation_scripts: Option<Vec<ValidationScript>>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing ValidationScript Enhanced Data Structure (GREEN Phase)");

    // Test 1: Direct struct creation with new enums
    println!("\n✅ Test 1: Direct ValidationScript Creation");
    let script = ValidationScript {
        name: "test_script".to_string(),
        language: ScriptLanguage::Lua,
        execution_phase: ExecutionPhase::Before,
        required: true,
        source: "result = { success = true }".to_string(),
        timeout_ms: Some(5000),
    };

    println!("  ✓ Created ValidationScript with enhanced structure");
    println!("  ✓ Name: {}", script.name);
    println!("  ✓ Language: {:?}", script.language);
    println!("  ✓ Execution Phase: {:?}", script.execution_phase);
    println!("  ✓ Required: {}", script.required);
    println!("  ✓ Timeout: {:?}", script.timeout_ms);

    // Test 2: Test all enum variants
    println!("\n✅ Test 2: Testing All Enum Variants");

    let languages = vec![
        ScriptLanguage::JavaScript,
        ScriptLanguage::Python,
        ScriptLanguage::Lua,
    ];

    let phases = vec![
        ExecutionPhase::Before,
        ExecutionPhase::After,
        ExecutionPhase::Both,
    ];

    for language in &languages {
        for phase in &phases {
            let test_script = ValidationScript {
                name: format!("test_{:?}_{:?}", language, phase),
                language: language.clone(),
                execution_phase: phase.clone(),
                required: false,
                source: "test script".to_string(),
                timeout_ms: None,
            };
            println!("  ✓ Created script: {:?} + {:?}", language, phase);
        }
    }

    // Test 3: JSON serialization (instead of YAML to avoid dependency issues)
    println!("\n✅ Test 3: JSON Serialization Test");
    let script = ValidationScript {
        name: "serialize_test".to_string(),
        language: ScriptLanguage::JavaScript,
        execution_phase: ExecutionPhase::Both,
        required: false,
        source: "console.log('test')".to_string(),
        timeout_ms: Some(3000),
    };

    let json = serde_json::to_string_pretty(&script)?;
    println!("  ✓ Successfully serialized ValidationScript to JSON:");
    println!("{}", json);

    // Test 4: JSON deserialization
    println!("\n✅ Test 4: JSON Deserialization Test");
    let json_input = r#"{
        "name": "deserialize_test",
        "language": "python",
        "execution_phase": "after",
        "required": true,
        "source": "print('hello world')",
        "timeout_ms": 2000
    }"#;

    let parsed: ValidationScript = serde_json::from_str(json_input)?;
    println!("  ✓ Successfully deserialized ValidationScript from JSON");
    println!("  ✓ Name: {}", parsed.name);
    println!("  ✓ Language: {:?}", parsed.language);
    println!("  ✓ Execution Phase: {:?}", parsed.execution_phase);
    println!("  ✓ Required: {}", parsed.required);
    println!("  ✓ Timeout: {:?}", parsed.timeout_ms);

    // Verify the parsed values
    assert_eq!(parsed.name, "deserialize_test");
    assert_eq!(parsed.language, ScriptLanguage::Python);
    assert_eq!(parsed.execution_phase, ExecutionPhase::After);
    assert_eq!(parsed.required, true);
    assert_eq!(parsed.timeout_ms, Some(2000));

    // Test 5: Test specification with multiple scripts
    println!("\n✅ Test 5: Complex Multi-Script JSON Specification");
    let complex_json = r#"{
        "name": "Enhanced Test Server",
        "version": "1.0.0",
        "validation_scripts": [
            {
                "name": "lua_validator",
                "language": "lua",
                "execution_phase": "after",
                "required": true,
                "source": "result = { success = true }"
            },
            {
                "name": "js_validator",
                "language": "javascript",
                "execution_phase": "both",
                "required": false,
                "source": "result = { success: true };",
                "timeout_ms": 1500
            },
            {
                "name": "py_validator",
                "language": "python",
                "execution_phase": "before",
                "required": true,
                "source": "result = {'success': True}",
                "timeout_ms": 3000
            }
        ]
    }"#;

    let spec: TestSpec = serde_json::from_str(complex_json)?;
    println!("  ✓ Successfully parsed complex specification");

    let scripts = spec.validation_scripts.unwrap();
    println!("  ✓ Found {} validation scripts", scripts.len());

    // Verify each script
    assert_eq!(scripts[0].language, ScriptLanguage::Lua);
    assert_eq!(scripts[0].execution_phase, ExecutionPhase::After);
    assert_eq!(scripts[0].required, true);

    assert_eq!(scripts[1].language, ScriptLanguage::JavaScript);
    assert_eq!(scripts[1].execution_phase, ExecutionPhase::Both);
    assert_eq!(scripts[1].required, false);
    assert_eq!(scripts[1].timeout_ms, Some(1500));

    assert_eq!(scripts[2].language, ScriptLanguage::Python);
    assert_eq!(scripts[2].execution_phase, ExecutionPhase::Before);
    assert_eq!(scripts[2].required, true);
    assert_eq!(scripts[2].timeout_ms, Some(3000));

    println!("  ✓ All assertions passed for complex specification!");

    println!("\n🎉 SUCCESS! ValidationScript Enhanced Data Structure is working correctly!");
    println!("\n📋 GREEN Phase Implementation Summary:");
    println!("  ✅ ScriptLanguage enum (JavaScript, Python, Lua)");
    println!("  ✅ ExecutionPhase enum (Before, After, Both)");
    println!("  ✅ Enhanced ValidationScript struct with proper types");
    println!("  ✅ Non-optional required: bool field");
    println!("  ✅ Non-optional source: String field");
    println!("  ✅ Optional timeout_ms: Option<u64> field");
    println!("  ✅ JSON serialization/deserialization working");
    println!("  ✅ Complex multi-script specifications supported");
    println!("  ✅ Type safety enforced through enum usage");

    println!("\n🔄 Next Steps (REFACTOR Phase):");
    println!("  • Complete script execution integration");
    println!("  • Add ScriptContext generation");
    println!("  • Wire into ValidationEngine");
    println!("  • Add comprehensive error handling");
    println!("  • Performance optimization");

    Ok(())
}
