use serde::{Deserialize, Serialize};

/// Rolling coverage snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageSnapshot {
    pub version: u64,
    pub total_targets: usize,
    pub covered_targets: usize,
    pub coverage_pct: f64,
    pub timestamp: u64,
}

pub struct CoverageAnalysis {
    snapshots: Vec<CoverageSnapshot>,
    max_snapshots: usize,
}

impl CoverageAnalysis {
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::new(),
            max_snapshots,
        }
    }

    /// Push a new snapshot, trimming oldest entries to stay within `max_snapshots`.
    pub fn record(&mut self, snapshot: CoverageSnapshot) {
        self.snapshots.push(snapshot);
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    pub fn latest(&self) -> Option<&CoverageSnapshot> {
        self.snapshots.last()
    }

    /// Compute the coverage delta between two version snapshots.
    pub fn diff(&self, from: u64, to: u64) -> Option<CoverageDiff> {
        let from_snap = self.snapshots.iter().find(|s| s.version == from)?;
        let to_snap = self.snapshots.iter().find(|s| s.version == to)?;
        Some(CoverageDiff {
            from_version: from,
            to_version: to,
            coverage_delta: to_snap.coverage_pct - from_snap.coverage_pct,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageDiff {
    pub from_version: u64,
    pub to_version: u64,
    pub coverage_delta: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(version: u64, pct: f64) -> CoverageSnapshot {
        CoverageSnapshot {
            version,
            total_targets: 100,
            covered_targets: (pct as usize),
            coverage_pct: pct,
            timestamp: version * 1000,
        }
    }

    #[test]
    fn record_and_latest() {
        let mut ca = CoverageAnalysis::new(10);
        assert!(ca.latest().is_none());

        ca.record(snap(1, 50.0));
        ca.record(snap(2, 75.0));

        let latest = ca.latest().unwrap();
        assert_eq!(latest.version, 2);
        assert!((latest.coverage_pct - 75.0).abs() < f64::EPSILON);
    }

    #[test]
    fn diff_between_versions() {
        let mut ca = CoverageAnalysis::new(10);
        ca.record(snap(1, 50.0));
        ca.record(snap(2, 75.0));
        ca.record(snap(3, 90.0));

        let diff = ca.diff(1, 3).unwrap();
        assert_eq!(diff.from_version, 1);
        assert_eq!(diff.to_version, 3);
        assert!((diff.coverage_delta - 40.0).abs() < f64::EPSILON);

        assert!(ca.diff(1, 99).is_none());
    }

    #[test]
    fn max_snapshots_trim() {
        let mut ca = CoverageAnalysis::new(2);
        ca.record(snap(1, 10.0));
        ca.record(snap(2, 20.0));
        ca.record(snap(3, 30.0));

        assert_eq!(ca.snapshots.len(), 2);
        assert_eq!(ca.snapshots[0].version, 2);
        assert_eq!(ca.snapshots[1].version, 3);

        assert!(ca.diff(1, 3).is_none());
    }

    #[test]
    fn record_empty_targets_snapshot() {
        let mut ca = CoverageAnalysis::new(10);
        let s = CoverageSnapshot {
            version: 1,
            total_targets: 0,
            covered_targets: 0,
            coverage_pct: 0.0,
            timestamp: 1000,
        };
        ca.record(s);
        let latest = ca.latest().unwrap();
        assert_eq!(latest.total_targets, 0);
        assert_eq!(latest.covered_targets, 0);
        assert!((latest.coverage_pct - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn diff_nonexistent_version_returns_empty() {
        let mut ca = CoverageAnalysis::new(10);
        ca.record(snap(1, 50.0));
        assert!(ca.diff(1, 999).is_none());
        assert!(ca.diff(999, 1).is_none());
        assert!(ca.diff(888, 999).is_none());
    }
}
