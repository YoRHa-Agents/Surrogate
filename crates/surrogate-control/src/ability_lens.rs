use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Ability {
    PerAppCoverage,
    ProxyUnit,
    EgressQuality,
    GlobalAppControl,
    StreamingStack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilitySnapshot {
    pub ability: Ability,
    pub status: AbilityStatus,
    pub maturity_pct: f64,
    pub dependencies_met: Vec<String>,
    pub dependencies_unmet: Vec<String>,
    pub next_steps: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AbilityStatus {
    NotStarted,
    InProgress,
    Functional,
    Mature,
}

/// Stateless projection service for ability maturity assessment.
pub struct AbilityLens;

impl AbilityLens {
    /// Project current ability status from available data.
    pub fn project(abilities: &[Ability]) -> Vec<AbilitySnapshot> {
        abilities
            .iter()
            .map(|ability| AbilitySnapshot {
                ability: *ability,
                status: AbilityStatus::InProgress,
                maturity_pct: 25.0,
                dependencies_met: vec!["surrogate-contract".to_string()],
                dependencies_unmet: vec![],
                next_steps: vec![format!("Complete {:?} implementation", ability)],
            })
            .collect()
    }

    pub fn all_abilities() -> Vec<Ability> {
        vec![
            Ability::PerAppCoverage,
            Ability::ProxyUnit,
            Ability::EgressQuality,
            Ability::GlobalAppControl,
            Ability::StreamingStack,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_all_five() {
        let abilities = AbilityLens::all_abilities();
        let snapshots = AbilityLens::project(&abilities);

        assert_eq!(snapshots.len(), 5);
        for snap in &snapshots {
            assert_eq!(snap.status, AbilityStatus::InProgress);
            assert!((snap.maturity_pct - 25.0).abs() < f64::EPSILON);
            assert!(
                snap.dependencies_met
                    .contains(&"surrogate-contract".to_string())
            );
            assert!(snap.dependencies_unmet.is_empty());
            assert_eq!(snap.next_steps.len(), 1);
        }

        assert_eq!(snapshots[0].ability, Ability::PerAppCoverage);
        assert_eq!(snapshots[4].ability, Ability::StreamingStack);
    }

    #[test]
    fn all_abilities_returns_five() {
        let abilities = AbilityLens::all_abilities();
        assert_eq!(abilities.len(), 5);

        use std::collections::HashSet;
        let unique: HashSet<_> = abilities.iter().collect();
        assert_eq!(unique.len(), 5);
    }

    #[test]
    fn project_returns_correct_ability_names() {
        let abilities = AbilityLens::all_abilities();
        let snapshots = AbilityLens::project(&abilities);
        let names: Vec<String> = snapshots
            .iter()
            .map(|s| format!("{:?}", s.ability))
            .collect();
        assert_eq!(
            names,
            vec![
                "PerAppCoverage",
                "ProxyUnit",
                "EgressQuality",
                "GlobalAppControl",
                "StreamingStack",
            ]
        );
    }
}
