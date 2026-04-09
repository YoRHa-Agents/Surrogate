use crate::dispatcher::AppController;
use crate::theme::{yorha_button, yorha_group_box, BODY, CAPTION, TITLE_LG};
use cocoanut::prelude::*;

pub fn build(controller: &AppController) -> View {
    let outbounds = controller.outbounds();
    let exit_names: Vec<String> = outbounds.iter().map(|o| o.id.clone()).collect();
    let running = controller.is_running();

    let mut page = View::vstack()
        .child(View::text("EGRESS LAB").bold().font_size(TITLE_LG))
        .child(View::spacer().height(8.0))
        .child(
            View::text("Outbound quality analysis, region risk, and site connectivity.")
                .font_size(BODY),
        )
        .child(View::spacer().height(12.0));

    let mut selector_row = View::hstack()
        .child(View::text("OUTBOUND:").font_size(BODY))
        .child(View::spacer().width(8.0))
        .child(View::dropdown("Select outbound", exit_names))
        .child(View::spacer().width(16.0));

    if running {
        selector_row = selector_row.child(
            yorha_button("RUN EGRESS TESTS").on_click_fn(move || {
                eprintln!("[surrogate] egress-lab: RUN EGRESS TESTS clicked (placeholder)");
            }),
        );
    } else {
        selector_row = selector_row.child(
            View::text("Start proxy to test")
                .font_size(BODY),
        );
    }
    selector_row = selector_row.child(View::spacer());
    page = page.child(selector_row);

    page = page.child(View::spacer().height(16.0));

    page = page.child(
        yorha_group_box("OUTBOUND QUALITY").child(View::table_view(
            vec!["METRIC".to_string(), "VALUE".to_string()],
            vec![
                vec!["Latency".to_string(), "— ms".to_string()],
                vec!["Packet Loss".to_string(), "— %".to_string()],
                vec!["Jitter".to_string(), "— ms".to_string()],
                vec!["Throughput".to_string(), "— Mbps".to_string()],
                vec!["DNS Resolution".to_string(), "— ms".to_string()],
            ],
        )),
    );

    page = page.child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("REGION RISK").child(
            View::vstack()
                .child(
                    View::text("Run egress tests to generate region risk assessment.")
                        .font_size(BODY),
                )
                .child(View::spacer().height(4.0))
                .child(
                    View::text("Checks: ASN geo, datacenter classification, policy compliance")
                        .font_size(CAPTION),
                ),
        ),
    );

    page = page.child(View::spacer().height(12.0));

    page = page.child(
        yorha_group_box("SITE GROUP CONNECTIVITY").child(View::table_view(
            vec![
                "SITE GROUP".to_string(),
                "STATUS".to_string(),
            ],
            vec![
                vec!["AI Tools".to_string(), "Not tested".to_string()],
                vec!["Dev Resources".to_string(), "Not tested".to_string()],
                vec!["Cloud Infra".to_string(), "Not tested".to_string()],
                vec!["Streaming".to_string(), "Not tested".to_string()],
            ],
        )),
    );

    page
}

#[cfg(test)]
mod tests {
    #[test]
    fn egress_lab_compiles() {
        assert!(true);
    }
}
