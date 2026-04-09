use crate::dispatcher::{AppController, UiMode};
use cocoanut::prelude::*;
use surrogate_control::ability_lens::{Ability, AbilityLens, AbilitySnapshot, AbilityStatus};

struct CardSpec {
    title: &'static str,
    nav_hint: &'static str,
}

fn card_spec(ability: Ability) -> CardSpec {
    match ability {
        Ability::PerAppCoverage => CardSpec {
            title: "Per-App Proxy & Coverage",
            nav_hint: "Apps",
        },
        Ability::ProxyUnit => CardSpec {
            title: "Proxy Unit (Holistic Capability)",
            nav_hint: "Profiles",
        },
        Ability::EgressQuality => CardSpec {
            title: "Egress Purity & Site Connectivity",
            nav_hint: "Egress Lab",
        },
        Ability::GlobalAppControl => CardSpec {
            title: "Global/Per-App Start/Stop",
            nav_hint: "Apps + Settings",
        },
        Ability::StreamingStack => CardSpec {
            title: "HTTP/1.1 / HTTP/2 Streaming",
            nav_hint: "Components",
        },
    }
}

fn status_label(snap: &AbilitySnapshot) -> &'static str {
    if !snap.dependencies_unmet.is_empty() {
        return "Unavailable";
    }
    match snap.status {
        AbilityStatus::NotStarted => "NotInstalled",
        AbilityStatus::InProgress => "Degraded",
        AbilityStatus::Functional => "Available",
        AbilityStatus::Mature => "Available",
    }
}

fn traffic_glyph(snap: &AbilitySnapshot) -> &'static str {
    match status_label(snap) {
        "Available" => "🟢",
        "Degraded" => "🟡",
        "Unavailable" => "🔴",
        "NotInstalled" => "⚫",
        _ => "⚫",
    }
}

fn maturity_label(pct: f64) -> &'static str {
    if pct < 25.0 {
        "Experimental"
    } else if pct < 50.0 {
        "Beta"
    } else if pct < 90.0 {
        "Stable"
    } else {
        "Frozen"
    }
}

fn build_card_body(mode: UiMode, snap: &AbilitySnapshot, spec: &CardSpec) -> View {
    let mut col = View::vstack();

    match mode {
        UiMode::Simple => {
            let line = traffic_glyph(snap).to_string();
            col = col.child(View::text(&line).font_size(20.0));
        }
        UiMode::Advanced => {
            col = col.child(
                View::text(&format!("Link: {}", spec.nav_hint)).font_size(11.0),
            );
            col = col.child(View::spacer().height(4.0));
            let status = status_label(snap);
            let mat = maturity_label(snap.maturity_pct);
            col = col.child(
                View::text(&format!("Status: {status} · Maturity: {mat}")).font_size(11.0),
            );
        }
        UiMode::Expert => {
            col = col.child(
                View::text(&format!("Navigation: {}", spec.nav_hint)).font_size(11.0),
            );
            col = col.child(View::spacer().height(4.0));
            let status = status_label(snap);
            let mat = maturity_label(snap.maturity_pct);
            col = col.child(
                View::text(&format!("Status: {status} · Maturity: {mat}")).font_size(11.0),
            );
            let deps_ok = snap.dependencies_met.join(", ");
            let deps_bad = snap.dependencies_unmet.join(", ");
            col = col.child(
                View::text(&format!("Dependencies met: {deps_ok}")).font_size(10.0),
            );
            col = col.child(
                View::text(&format!("Dependencies unmet: {deps_bad}")).font_size(10.0),
            );
            let steps = snap.next_steps.join("; ");
            col = col.child(View::text(&format!("Next: {steps}")).font_size(10.0));
        }
    }

    col
}

pub fn build(controller: &AppController) -> View {
    let abilities = AbilityLens::all_abilities();
    let snapshots = AbilityLens::project(&abilities);
    let mode = controller.ui_mode();

    let mut page = View::vstack()
        .child(View::text("Ability Lens").bold().font_size(20.0))
        .child(View::spacer().height(8.0))
        .child(
            View::text("Capability coverage and maturity (from control plane projection).")
                .font_size(11.0),
        )
        .child(View::spacer().height(12.0));

    for snap in snapshots.iter() {
        let spec = card_spec(snap.ability);
        let body = build_card_body(mode, snap, &spec);
        page = page.child(View::group_box(spec.title).child(body));
        page = page.child(View::spacer().height(8.0));
    }

    page
}
