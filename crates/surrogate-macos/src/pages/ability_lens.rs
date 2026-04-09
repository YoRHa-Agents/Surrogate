use crate::dispatcher::{AppController, UiMode};
use crate::theme::{
    yorha_group_box, yorha_section_header, BODY, CAPTION, MICRO, TITLE_LG, TITLE_SM,
};
use cocoanut::prelude::*;
use surrogate_control::ability_lens::{Ability, AbilityLens, AbilitySnapshot, AbilityStatus};

fn ability_name(ability: Ability) -> &'static str {
    match ability {
        Ability::PerAppCoverage => "PER-APP COVERAGE",
        Ability::ProxyUnit => "PROXY UNIT",
        Ability::EgressQuality => "EGRESS QUALITY",
        Ability::GlobalAppControl => "GLOBAL APP CONTROL",
        Ability::StreamingStack => "STREAMING STACK",
    }
}

fn status_text(status: AbilityStatus) -> &'static str {
    match status {
        AbilityStatus::NotStarted => "NOT STARTED",
        AbilityStatus::InProgress => "IN PROGRESS",
        AbilityStatus::Functional => "FUNCTIONAL",
        AbilityStatus::Mature => "MATURE",
    }
}

fn build_summary_table(snapshots: &[AbilitySnapshot]) -> View {
    let headers = vec![
        "ABILITY".to_string(),
        "STATUS".to_string(),
        "MATURITY".to_string(),
        "DEPS MET".to_string(),
        "DEPS UNMET".to_string(),
    ];

    let rows: Vec<Vec<String>> = snapshots
        .iter()
        .map(|snap| {
            vec![
                ability_name(snap.ability).to_string(),
                status_text(snap.status).to_string(),
                format!("{}%", snap.maturity_pct as u32),
                snap.dependencies_met.len().to_string(),
                snap.dependencies_unmet.len().to_string(),
            ]
        })
        .collect();

    View::table_view(headers, rows)
}

fn build_ability_card(mode: UiMode, snap: &AbilitySnapshot) -> View {
    let name = ability_name(snap.ability);
    let mut body = View::vstack();

    let status_badge = format!("[{}]", status_text(snap.status));
    let maturity = format!("MATURITY: {}%", snap.maturity_pct as u32);

    body = body.child(
        View::hstack()
            .child(View::text(&status_badge).font_size(BODY).bold())
            .child(View::spacer().width(12.0))
            .child(View::text(&maturity).font_size(BODY))
            .child(View::spacer()),
    );

    match mode {
        UiMode::Simple => {}
        UiMode::Advanced | UiMode::Expert => {
            body = body.child(View::spacer().height(6.0));

            let deps_met = if snap.dependencies_met.is_empty() {
                "—".to_string()
            } else {
                snap.dependencies_met.join(", ")
            };
            let deps_unmet = if snap.dependencies_unmet.is_empty() {
                "—".to_string()
            } else {
                snap.dependencies_unmet.join(", ")
            };

            body = body.child(
                View::text(&format!("DEPS MET: {deps_met}"))
                    .font_size(CAPTION),
            );
            body = body.child(
                View::text(&format!("DEPS UNMET: {deps_unmet}"))
                    .font_size(CAPTION),
            );

            if !snap.next_steps.is_empty() {
                body = body.child(View::spacer().height(4.0));
                let steps = snap.next_steps.join("; ");
                body = body.child(
                    View::text(&format!("NEXT: {steps}"))
                        .font_size(MICRO),
                );
            }
        }
    }

    yorha_group_box(name).child(body)
}

pub fn build(controller: &AppController) -> View {
    let abilities = AbilityLens::all_abilities();
    let snapshots = AbilityLens::project(&abilities);
    let mode = controller.ui_mode();

    let mut page = View::vstack()
        .child(View::text("ABILITY LENS").bold().font_size(TITLE_LG))
        .child(View::spacer().height(8.0))
        .child(
            View::text("Capability coverage and maturity projection.")
                .font_size(BODY),
        )
        .child(View::spacer().height(12.0));

    page = page
        .child(yorha_section_header("SUMMARY"))
        .child(build_summary_table(&snapshots))
        .child(View::spacer().height(12.0));

    page = page.child(yorha_section_header("ABILITIES"));

    for snap in snapshots.iter() {
        page = page.child(build_ability_card(mode, snap));
        page = page.child(View::spacer().height(8.0));
    }

    page
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ability_name_covers_all_variants() {
        let abilities = AbilityLens::all_abilities();
        for a in &abilities {
            let name = ability_name(*a);
            assert!(!name.is_empty());
            assert_eq!(name, name.to_uppercase(), "name must be UPPERCASE");
        }
    }

    #[test]
    fn status_text_covers_all_variants() {
        let variants = [
            AbilityStatus::NotStarted,
            AbilityStatus::InProgress,
            AbilityStatus::Functional,
            AbilityStatus::Mature,
        ];
        for s in &variants {
            let text = status_text(*s);
            assert!(!text.is_empty());
            assert_eq!(text, text.to_uppercase());
        }
    }

    #[test]
    fn summary_table_has_correct_row_count() {
        let abilities = AbilityLens::all_abilities();
        let snapshots = AbilityLens::project(&abilities);
        assert_eq!(snapshots.len(), 5);
    }
}
