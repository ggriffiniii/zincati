use crate::cincinnati::*;
use crate::identity::Identity;
use httptest::{mappers::*, responders::*, Expectation};
use std::collections::BTreeSet;
use tokio::runtime::current_thread as rt;

#[test]
fn test_empty_graph() {
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::path(matches(r"^/v1/graph?.+$")),
            request::headers(contains_entry(("accept", matches("application/json")))),
        ])
        .respond_with(json_encoded(serde_json::json!({
            "nodes": [],
            "edges": [],
        }))),
    );
    let id = Identity::mock_default();
    let client = Cincinnati {
        base_url: server.url_str("/"),
    };
    let update = rt::block_on_all(client.next_update(&id, BTreeSet::new(), false));

    assert!(update.unwrap().is_none());
}
