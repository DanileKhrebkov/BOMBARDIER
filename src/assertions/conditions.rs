// src/assertions/conditions.rs
use std::fmt;

#[derive(Debug, Clone)]
pub enum AssertionStatus {
    Pass,
    Fail,
}

impl fmt::Display for AssertionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssertionStatus::Pass => write!(f, "✅ PASS"),
            AssertionStatus::Fail => write!(f, "❌ FAIL"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub assertion: String,
    pub status: AssertionStatus,
    pub actual_value: String,
    pub expected_value: String,
    pub message: String,
}

impl AssertionResult {
    pub fn pass(assertion: String, actual: String, expected: String) -> Self {
        Self {
            assertion,
            status: AssertionStatus::Pass,
            actual_value: actual,
            expected_value: expected,
            message: "✅ Условие выполнено".to_string(),
        }
    }

    pub fn fail(assertion: String, actual: String, expected: String, message: String) -> Self {
        Self {
            assertion,
            status: AssertionStatus::Fail,
            actual_value: actual,
            expected_value: expected,
            message,
        }
    }
}