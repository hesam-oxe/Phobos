// Phobos IDE - Code Explainer AI Module
// Intelligent code explanation and documentation system

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Explanation {
    pub summary: String,
    pub detailed_explanation: String,
    pub complexity: ComplexityLevel,
    pub key_concepts: Vec<String>,
    pub suggestions: Vec<String>,
    pub code_structure: CodeStructure,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComplexityLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone)]
pub struct CodeStructure {
    pub functions: Vec<FunctionInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub traits: Vec<TraitInfo>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub visibility: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub fields: Vec<(String, String)>,
    pub methods: Vec<String>,
    pub visibility: String,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub variants: Vec<String>,
    pub visibility: String,
}

#[derive(Debug, Clone)]
pub struct TraitInfo {
    pub name: String,
    pub methods: Vec<String>,
    pub visibility: String,
}

pub struct CodeExplainer {
    language: String,
    explanation_templates: HashMap<&'static str, &'static str>,
}

impl CodeExplainer {
    pub fn new(language: &str) -> Self {
        let mut explainer = Self {
            language: language.to_string(),
            explanation_templates: HashMap::new(),
        };
        explainer.initialize_templates();
        explainer
    }

    fn initialize_templates(&mut self) {
        self.explanation_templates.insert("rust_fn", 
            "This function {name} takes {params} and returns {return_type}. It appears to {purpose}.");
        self.explanation_templates.insert(
            "rust_struct",
            "This struct {name} represents {purpose}. It has {field_count} fields: {fields}.",
        );
        self.explanation_templates.insert(
            "rust_enum",
            "This enum {name} defines {variant_count} possible states: {variants}.",
        );
        self.explanation_templates.insert(
            "rust_trait",
            "This trait {name} defines a contract with {method_count} methods: {methods}.",
        );
    }

    pub fn explain(&self, code: &str) -> Explanation {
        let structure = self.analyze_structure(code);

        let summary = self.generate_summary(&structure);
        let detailed = self.generate_detailed_explanation(code, &structure);
        let complexity = self.assess_complexity(code, &structure);
        let concepts = self.extract_key_concepts(code, &structure);
        let suggestions = self.generate_suggestions(code, &structure);

        Explanation {
            summary,
            detailed_explanation: detailed,
            complexity,
            key_concepts: concepts,
            suggestions,
            code_structure: structure,
        }
    }

    fn analyze_structure(&self, code: &str) -> CodeStructure {
        let mut structure = CodeStructure {
            functions: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            traits: Vec::new(),
            imports: Vec::new(),
        };

        // Parse functions
        for line in code.lines() {
            if let Some(func_info) = self.parse_function(line, code) {
                structure.functions.push(func_info);
            }
            if let Some(struct_info) = self.parse_struct(line, code) {
                structure.structs.push(struct_info);
            }
            if let Some(enum_info) = self.parse_enum(line, code) {
                structure.enums.push(enum_info);
            }
            if let Some(trait_info) = self.parse_trait(line, code) {
                structure.traits.push(trait_info);
            }
            if line.trim().starts_with("use ") {
                structure.imports.push(line.trim().to_string());
            }
        }

        structure
    }

    fn parse_function(&self, line: &str, full_code: &str) -> Option<FunctionInfo> {
        if !line.contains("fn ") {
            return None;
        }

        // Extract function name
        let name_start = line.find("fn ")? + 3;
        let name_end = line[name_start..].find('(')? + name_start;
        let name = line[name_start..name_end].trim().to_string();

        // Extract parameters
        let param_start = line.find('(')? + 1;
        let param_end = line.find(')')?;
        let params_str = &line[param_start..param_end];
        let parameters: Vec<String> = params_str
            .split(',')
            .map(|p| p.trim().to_string())
            .filter(|p| !p.is_empty())
            .collect();

        // Extract return type
        let return_type = if let Some(ret_idx) = line.find("->") {
            let ret_end = line.find('{').unwrap_or(line.len());
            Some(line[ret_idx + 2..ret_end].trim().to_string())
        } else {
            None
        };

        // Determine visibility
        let visibility = if line.contains("pub fn") {
            "public"
        } else {
            "private"
        }
        .to_string();

        // Generate simple description
        let description = format!("Function that processes {} parameters", parameters.len());

        Some(FunctionInfo {
            name,
            parameters,
            return_type,
            visibility,
            description,
        })
    }

    fn parse_struct(&self, line: &str, full_code: &str) -> Option<StructInfo> {
        if !line.contains("struct ") {
            return None;
        }

        let name_start = line.find("struct ")? + 7;
        let name_end = line[name_start..]
            .find('{')
            .or_else(|| line[name_start..].find(';'))?
            + name_start;
        let name = line[name_start..name_end].trim().to_string();

        let visibility = if line.contains("pub struct") {
            "public"
        } else {
            "private"
        }
        .to_string();

        // Simple field extraction (works for single-line structs)
        let fields = Vec::new(); // Would need more complex parsing for multi-line
        let methods = Vec::new();

        Some(StructInfo {
            name,
            fields,
            methods,
            visibility,
        })
    }

    fn parse_enum(&self, line: &str, full_code: &str) -> Option<EnumInfo> {
        if !line.contains("enum ") {
            return None;
        }

        let name_start = line.find("enum ")? + 5;
        let name_end = line[name_start..].find('{')? + name_start;
        let name = line[name_start..name_end].trim().to_string();

        let visibility = if line.contains("pub enum") {
            "public"
        } else {
            "private"
        }
        .to_string();

        // Extract variants from the same line if possible
        let variants = Vec::new(); // Would need more complex parsing

        Some(EnumInfo {
            name,
            variants,
            visibility,
        })
    }

    fn parse_trait(&self, line: &str, full_code: &str) -> Option<TraitInfo> {
        if !line.contains("trait ") {
            return None;
        }

        let name_start = line.find("trait ")? + 6;
        let name_end = line[name_start..].find('{')? + name_start;
        let name = line[name_start..name_end].trim().to_string();

        let visibility = if line.contains("pub trait") {
            "public"
        } else {
            "private"
        }
        .to_string();

        let methods = Vec::new();

        Some(TraitInfo {
            name,
            methods,
            visibility,
        })
    }

    fn generate_summary(&self, structure: &CodeStructure) -> String {
        let mut parts = Vec::new();

        if !structure.functions.is_empty() {
            parts.push(format!("{} function(s)", structure.functions.len()));
        }
        if !structure.structs.is_empty() {
            parts.push(format!("{} struct(s)", structure.structs.len()));
        }
        if !structure.enums.is_empty() {
            parts.push(format!("{} enum(s)", structure.enums.len()));
        }
        if !structure.traits.is_empty() {
            parts.push(format!("{} trait(s)", structure.traits.len()));
        }
        if !structure.imports.is_empty() {
            parts.push(format!("{} import(s)", structure.imports.len()));
        }

        if parts.is_empty() {
            "This code snippet doesn't contain major structural elements.".to_string()
        } else {
            format!("This code contains: {}. ", parts.join(", "))
        }
    }

    fn generate_detailed_explanation(&self, code: &str, structure: &CodeStructure) -> String {
        let mut explanation = String::new();

        // Explain imports
        if !structure.imports.is_empty() {
            explanation.push_str("\n## Imports\n");
            for imp in &structure.imports {
                explanation.push_str(&format!("- {}\n", imp));
            }
        }

        // Explain functions
        if !structure.functions.is_empty() {
            explanation.push_str("\n## Functions\n");
            for func in &structure.functions {
                explanation.push_str(&format!(
                    "- **{}**({}): {}{}\n",
                    func.name,
                    func.parameters.join(", "),
                    func.description,
                    match &func.return_type {
                        Some(rt) => format!(" → {}", rt),
                        None => String::new(),
                    }
                ));
            }
        }

        // Explain structs
        if !structure.structs.is_empty() {
            explanation.push_str("\n## Data Structures\n");
            for st in &structure.structs {
                explanation.push_str(&format!(
                    "- **{}**: {} struct with fields\n",
                    st.name, st.visibility
                ));
            }
        }

        // Explain enums
        if !structure.enums.is_empty() {
            explanation.push_str("\n## Enumerations\n");
            for en in &structure.enums {
                explanation.push_str(&format!(
                    "- **{}**: {} enum defining multiple variants\n",
                    en.name, en.visibility
                ));
            }
        }

        // Explain traits
        if !structure.traits.is_empty() {
            explanation.push_str("\n## Traits\n");
            for tr in &structure.traits {
                explanation.push_str(&format!(
                    "- **{}**: {} trait defining a contract\n",
                    tr.name, tr.visibility
                ));
            }
        }

        explanation
    }

    fn assess_complexity(&self, code: &str, structure: &CodeStructure) -> ComplexityLevel {
        let mut score = 0;

        // Count lines
        let line_count = code.lines().count();
        if line_count > 100 {
            score += 2;
        } else if line_count > 50 {
            score += 1;
        }

        // Count nested structures
        let nesting = code.matches('{').count();
        if nesting > 10 {
            score += 2;
        } else if nesting > 5 {
            score += 1;
        }

        // Count functions
        if structure.functions.len() > 5 {
            score += 1;
        }

        // Check for advanced features
        if code.contains("unsafe") || code.contains("macro_rules") {
            score += 2;
        }
        if code.contains("async") || code.contains("await") {
            score += 1;
        }
        if code.contains("dyn ") || code.contains("impl ") {
            score += 1;
        }

        match score {
            0..=1 => ComplexityLevel::Beginner,
            2..=3 => ComplexityLevel::Intermediate,
            4..=5 => ComplexityLevel::Advanced,
            _ => ComplexityLevel::Expert,
        }
    }

    fn extract_key_concepts(&self, code: &str, structure: &CodeStructure) -> Vec<String> {
        let mut concepts = Vec::new();

        // Detect common patterns
        if code.contains("match ") {
            concepts.push("Pattern Matching".to_string());
        }
        if code.contains(".iter()") || code.contains(".into_iter()") {
            concepts.push("Iterators".to_string());
        }
        if code.contains("Option<") || code.contains("Some(") || code.contains("None") {
            concepts.push("Option Type".to_string());
        }
        if code.contains("Result<") || code.contains("Ok(") || code.contains("Err(") {
            concepts.push("Error Handling".to_string());
        }
        if code.contains("&mut ") {
            concepts.push("Mutable References".to_string());
        }
        if code.contains("Box<") {
            concepts.push("Heap Allocation".to_string());
        }
        if code.contains("Rc<") || code.contains("Arc<") {
            concepts.push("Reference Counting".to_string());
        }
        if code.contains("async ") || code.contains("await") {
            concepts.push("Asynchronous Programming".to_string());
        }
        if code.contains("unsafe ") {
            concepts.push("Unsafe Code".to_string());
        }

        concepts
    }

    fn generate_suggestions(&self, code: &str, structure: &CodeStructure) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Check for unwrap usage
        if code.contains(".unwrap()") {
            suggestions
                .push("Consider using proper error handling instead of .unwrap()".to_string());
        }

        // Check for TODO comments
        if code.contains("TODO") || code.contains("FIXME") {
            suggestions.push("Address pending TODO/FIXME comments".to_string());
        }

        // Check for long functions
        for func in &structure.functions {
            // Simplified check - would need better analysis in production
            if func.name.len() > 30 {
                suggestions.push(format!(
                    "Consider renaming '{}' to be more concise",
                    func.name
                ));
            }
        }

        // Check for missing documentation
        if !code.contains("///") && !code.contains("//!") && code.len() > 100 {
            suggestions.push("Consider adding documentation comments".to_string());
        }

        if suggestions.is_empty() {
            suggestions.push("Code looks good! No immediate improvements suggested.".to_string());
        }

        suggestions
    }

    pub fn explain_to_beginner(&self, code: &str) -> String {
        let explanation = self.explain(code);

        format!(
            "📚 **Simple Explanation**\n\n\
             {}\n\n\
             **Complexity Level**: {:?}\n\n\
             **Key Concepts to Learn**:\n{}\n\n\
             **Tips**:\n{}",
            explanation.summary,
            explanation.complexity,
            explanation
                .key_concepts
                .iter()
                .map(|c| format!("- {}", c))
                .collect::<Vec<_>>()
                .join("\n"),
            explanation
                .suggestions
                .iter()
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_parsing() {
        let explainer = CodeExplainer::new("rust");
        let code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let explanation = explainer.explain(code);
        assert!(!explanation.summary.is_empty());
        assert_eq!(explanation.code_structure.functions.len(), 1);
    }

    #[test]
    fn test_complexity_assessment() {
        let explainer = CodeExplainer::new("rust");
        let simple_code = "fn main() { println!(\"Hello\"); }";
        let complex_code = r#"
async fn process_data(items: Vec<i32>) -> Result<(), Error> {
    for item in items.iter() {
        match item {
            Some(x) => unsafe { /* ... */ },
            None => continue,
        }
    }
    Ok(())
}
"#;

        let simple_exp = explainer.explain(simple_code);
        let complex_exp = explainer.explain(complex_code);

        assert_eq!(simple_exp.complexity, ComplexityLevel::Beginner);
        assert!(complex_exp.complexity as u8 >= simple_exp.complexity as u8);
    }
}
