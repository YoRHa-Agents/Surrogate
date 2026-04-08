use serde::{Deserialize, Serialize};

/// The 5-dimension test framework.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TestDimension {
    Identity,
    Transport,
    Streaming,
    Tool,
    Risk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub dimension: TestDimension,
    pub test_name: String,
    pub passed: bool,
    pub score: f64,
    pub details: String,
}

pub struct TestWorkbench {
    results: Vec<TestResult>,
}

impl TestWorkbench {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn record(&mut self, result: TestResult) {
        self.results.push(result);
    }

    pub fn results_by_dimension(&self, dim: TestDimension) -> Vec<&TestResult> {
        self.results.iter().filter(|r| r.dimension == dim).collect()
    }

    /// Average of all scores.
    pub fn overall_score(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let total: f64 = self.results.iter().map(|r| r.score).sum();
        total / self.results.len() as f64
    }

    pub fn dimension_score(&self, dim: TestDimension) -> f64 {
        let dim_results: Vec<_> = self.results.iter().filter(|r| r.dimension == dim).collect();
        if dim_results.is_empty() {
            return 0.0;
        }
        let total: f64 = dim_results.iter().map(|r| r.score).sum();
        total / dim_results.len() as f64
    }

    pub fn pass_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let passed = self.results.iter().filter(|r| r.passed).count();
        passed as f64 / self.results.len() as f64
    }
}

impl Default for TestWorkbench {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result(dim: TestDimension, name: &str, passed: bool, score: f64) -> TestResult {
        TestResult {
            dimension: dim,
            test_name: name.to_string(),
            passed,
            score,
            details: String::new(),
        }
    }

    #[test]
    fn record_and_query() {
        let mut wb = TestWorkbench::new();
        wb.record(result(TestDimension::Identity, "id-1", true, 90.0));
        wb.record(result(TestDimension::Transport, "tp-1", false, 40.0));

        assert_eq!(wb.results_by_dimension(TestDimension::Identity).len(), 1);
        assert_eq!(wb.results_by_dimension(TestDimension::Transport).len(), 1);
        assert_eq!(wb.results_by_dimension(TestDimension::Streaming).len(), 0);
    }

    #[test]
    fn dimension_score() {
        let mut wb = TestWorkbench::new();
        wb.record(result(TestDimension::Risk, "r1", true, 80.0));
        wb.record(result(TestDimension::Risk, "r2", true, 60.0));
        wb.record(result(TestDimension::Tool, "t1", true, 100.0));

        let risk_score = wb.dimension_score(TestDimension::Risk);
        assert!((risk_score - 70.0).abs() < f64::EPSILON);

        let tool_score = wb.dimension_score(TestDimension::Tool);
        assert!((tool_score - 100.0).abs() < f64::EPSILON);

        assert!((wb.dimension_score(TestDimension::Streaming) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn pass_rate() {
        let mut wb = TestWorkbench::new();
        wb.record(result(TestDimension::Identity, "id-1", true, 90.0));
        wb.record(result(TestDimension::Transport, "tp-1", false, 40.0));
        wb.record(result(TestDimension::Tool, "tl-1", true, 80.0));

        let rate = wb.pass_rate();
        let expected = 2.0 / 3.0;
        assert!((rate - expected).abs() < 1e-9);
    }

    #[test]
    fn overall_score_empty_workbench() {
        let wb = TestWorkbench::new();
        assert!((wb.overall_score() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn overall_score_mixed_results() {
        let mut wb = TestWorkbench::new();
        wb.record(result(TestDimension::Identity, "id-1", true, 100.0));
        wb.record(result(TestDimension::Transport, "tp-1", false, 40.0));
        wb.record(result(TestDimension::Risk, "r-1", true, 80.0));
        wb.record(result(TestDimension::Tool, "t-1", false, 20.0));

        let expected_score = (100.0 + 40.0 + 80.0 + 20.0) / 4.0;
        assert!((wb.overall_score() - expected_score).abs() < f64::EPSILON);

        let expected_rate = 2.0 / 4.0;
        assert!((wb.pass_rate() - expected_rate).abs() < f64::EPSILON);
    }
}
