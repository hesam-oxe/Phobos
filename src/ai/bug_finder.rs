// Phobos IDE - Bug Finder AI Module
// Intelligent bug detection and code analysis system

use std::collections::HashMap;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct BugReport {
    pub line_number: usize,
    pub severity: Severity,
    pub category: BugCategory,
    pub message: String,
    pub suggestion: String,
    pub code_snippet: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BugCategory {
    Syntax,
    Logic,
    Performance,
    Security,
    Style,
    PotentialBug,
    BestPractice,
}

pub struct BugFinder {
    language: String,
    rules: HashMap<BugCategory, Vec<Box<dyn Rule>>>,
}

trait Rule {
    fn check(&self, code: &str, line_number: usize) -> Option<BugReport>;
    fn name(&self) -> &str;
}

// Rule: Unused variable detection
struct UnusedVariableRule {
    pattern: Regex,
}

impl UnusedVariableRule {
    fn new() -> Self {
        Self {
            pattern: Regex::new(r"let\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap(),
        }
    }
}

impl Rule for UnusedVariableRule {
    fn check(&self, code: &str, line_number: usize) -> Option<BugReport> {
        // Simplified check - in real implementation would track variable usage
        if let Some(caps) = self.pattern.captures(code) {
            let var_name = &caps[1];
            if !var_name.starts_with('_') && !code.contains(&format!("println!({})", var_name)) {
                return Some(BugReport {
                    line_number,
                    severity: Severity::Low,
                    category: BugCategory::Style,
                    message: format!("Variable '{}' may be unused", var_name),
                    suggestion: format!("Consider prefixing with '_' or using the variable: _{}", var_name),
                    code_snippet: Some(code.to_string()),
                });
            }
        }
        None
    }

    fn name(&self) -> &str {
        "UnusedVariableRule"
    }
}

// Rule: Unwrap on Option/Result
struct UnwrapRule;

impl Rule for UnwrapRule {
    fn check(&self, code: &str, line_number: usize) -> Option<BugReport> {
        if code.contains(".unwrap()") {
            return Some(BugReport {
                line_number,
                severity: Severity::High,
                category: BugCategory::PotentialBug,
                message: "Using .unwrap() can cause panics".to_string(),
                suggestion: "Consider using .unwrap_or(), .unwrap_or_else(), or proper error handling with ?".to_string(),
                code_snippet: Some(code.to_string()),
            });
        }
        None
    }

    fn name(&self) -> &str {
        "UnwrapRule"
    }
}

// Rule: Infinite loop detection
struct InfiniteLoopRule;

impl Rule for InfiniteLoopRule {
    fn check(&self, code: &str, line_number: usize) -> Option<BugReport> {
        if code.trim().starts_with("loop {") && !code.contains("break") {
            return Some(BugReport {
                line_number,
                severity: Severity::Critical,
                category: BugCategory::Logic,
                message: "Potential infinite loop detected".to_string(),
                suggestion: "Add a break condition or use while loop with condition".to_string(),
                code_snippet: Some(code.to_string()),
            });
        }
        None
    }

    fn name(&self) -> &str {
        "InfiniteLoopRule"
    }
}

// Rule: Mutable reference issues
struct MutableRefRule;

impl Rule for MutableRefRule {
    fn check(&self, code: &str, line_number: usize) -> Option<BugReport> {
        if code.contains("&mut") && code.contains("&") && !code.contains("&mut") {
            return Some(BugReport {
                line_number,
                severity: Severity::Medium,
                category: BugCategory::PotentialBug,
                message: "Potential borrowing conflict".to_string(),
                suggestion: "Review mutable and immutable borrow rules".to_string(),
                code_snippet: Some(code.to_string()),
            });
        }
        None
    }

    fn name(&self) -> &str {
        "MutableRefRule"
    }
}

// Rule: TODO/FIXME comments
struct TodoCommentRule {
    pattern: Regex,
}

impl TodoCommentRule {
    fn new() -> Self {
        Self {
            pattern: Regex::new(r"(TODO|FIXME|XXX|HACK):?\s*(.*)").unwrap(),
        }
    }
}

impl Rule for TodoCommentRule {
    fn check(&self, code: &str, line_number: usize) -> Option<BugReport> {
        if let Some(caps) = self.pattern.captures(code) {
            let todo_type = &caps[1];
            let description = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            return Some(BugReport {
                line_number,
                severity: Severity::Info,
                category: BugCategory::Style,
                message: format!("{} found: {}", todo_type, description),
                suggestion: "Address this technical debt item".to_string(),
                code_snippet: Some(code.to_string()),
            });
        }
        None
    }

    fn name(&self) -> &str {
        "TodoCommentRule"
    }
}

impl BugFinder {
    pub fn new(language: &str) -> Self {
        let mut finder = Self {
            language: language.to_string(),
            rules: HashMap::new(),
        };
        finder.initialize_rules();
        finder
    }

    fn initialize_rules(&mut self) {
        // Add rules for all categories
        self.rules.insert(BugCategory::Style, vec![
            Box::new(UnusedVariableRule::new()),
            Box::new(TodoCommentRule::new()),
        ]);
        self.rules.insert(BugCategory::PotentialBug, vec![
            Box::new(UnwrapRule),
            Box::new(MutableRefRule),
        ]);
        self.rules.insert(BugCategory::Logic, vec![
            Box::new(InfiniteLoopRule),
        ]);
    }

    pub fn analyze(&self, code: &str) -> Vec<BugReport> {
        let mut reports = Vec::new();
        
        for (line_number, line) in code.lines().enumerate() {
            for (_, rules) in &self.rules {
                for rule in rules {
                    if let Some(report) = rule.check(line, line_number + 1) {
                        reports.push(report);
                    }
                }
            }
        }

        // Sort by severity
        reports.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Critical => 0,
                Severity::High => 1,
                Severity::Medium => 2,
                Severity::Low => 3,
                Severity::Info => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        reports
    }

    pub fn get_quick_fixes(&self, report: &BugReport) -> Vec<String> {
        match report.category {
            BugCategory::PotentialBug if report.message.contains("unwrap") => {
                vec![
                    "Use .unwrap_or(default_value)".to_string(),
                    "Use .unwrap_or_else(|| default)".to_string(),
                    "Use ? operator for error propagation".to_string(),
                    "Use match statement for explicit handling".to_string(),
                ]
            }
            BugCategory::Style if report.message.contains("unused") => {
                vec![
                    "Prefix variable with underscore".to_string(),
                    "Remove unused variable".to_string(),
                    "Use the variable in code".to_string(),
                ]
            }
            _ => vec!["Review and fix manually".to_string()],
        }
    }

    pub fn analyze_file(&self, file_path: &str, code: &str) -> FileAnalysisReport {
        let bugs = self.analyze(code);
        let total_lines = code.lines().count();
        
        FileAnalysisReport {
            file_path: file_path.to_string(),
            total_lines,
            total_bugs: bugs.len(),
            bugs_by_severity: self.count_by_severity(&bugs),
            bugs_by_category: self.count_by_category(&bugs),
            bugs,
        }
    }

    fn count_by_severity(&self, bugs: &[BugReport]) -> HashMap<Severity, usize> {
        let mut counts = HashMap::new();
        for bug in bugs {
            *counts.entry(bug.severity.clone()).or_insert(0) += 1;
        }
        counts
    }

    fn count_by_category(&self, bugs: &[BugReport]) -> HashMap<BugCategory, usize> {
        let mut counts = HashMap::new();
        for bug in bugs {
            *counts.entry(bug.category.clone()).or_insert(0) += 1;
        }
        counts
    }
}

#[derive(Debug)]
pub struct FileAnalysisReport {
    pub file_path: String,
    pub total_lines: usize,
    pub total_bugs: usize,
    pub bugs_by_severity: HashMap<Severity, usize>,
    pub bugs_by_category: HashMap<BugCategory, usize>,
    pub bugs: Vec<BugReport>,
}

impl FileAnalysisReport {
    pub fn summary(&self) -> String {
        format!(
            "Analysis of {}: {} lines, {} issues found ({} critical, {} high, {} medium)",
            self.file_path,
            self.total_lines,
            self.total_bugs,
            self.bugs_by_severity.get(&Severity::Critical).unwrap_or(&0),
            self.bugs_by_severity.get(&Severity::High).unwrap_or(&0),
            self.bugs_by_severity.get(&Severity::Medium).unwrap_or(&0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_detection() {
        let finder = BugFinder::new("rust");
        let code = r#"
fn main() {
    let result = Some(5);
    let value = result.unwrap();
}
"#;
        let reports = finder.analyze(code);
        assert!(reports.iter().any(|r| r.message.contains("unwrap")));
    }

    #[test]
    fn test_todo_detection() {
        let finder = BugFinder::new("rust");
        let code = r#"
// TODO: Implement this function
fn incomplete() {}
"#;
        let reports = finder.analyze(code);
        assert!(reports.iter().any(|r| r.message.contains("TODO")));
    }
}
