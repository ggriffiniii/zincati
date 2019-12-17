use crate::cincinnati::Cincinnati;
use crate::identity::Identity;
use httptest::{mappers::*, responders::*, Expectation};
use serde_json::json;
use std::collections::BTreeSet;
use tokio::runtime::current_thread as rt;

#[test]
fn test_simple_graph() {
    let simple_graph = json!(
    {
      "nodes": [
        {
          "version": "0.0.0-mock",
          "metadata": {
            "org.fedoraproject.coreos.scheme": "checksum",
            "org.fedoraproject.coreos.releases.age_index": "0"
          },
          "payload": "sha-mock"
        },
        {
          "version": "30.20190725.0",
          "metadata": {
            "org.fedoraproject.coreos.scheme": "checksum",
            "org.fedoraproject.coreos.releases.age_index": "1"
          },
          "payload": "8b79877efa7ac06becd8637d95f8ca83aa385f89f383288bf3c2c31ca53216c7"
        }
      ],
      "edges": [
        [
          0,
          1
        ]
      ]
    });
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method("GET"),
            request::path(matches(r"^/v1/graph?.+$")),
            request::headers(contains_entry(("accept", matches("application/json")),)),
        ])
        .respond_with(json_encoded(simple_graph)),
    );

    let id = Identity::mock_default();
    let client = Cincinnati {
        base_url: server.url_str("/"),
    };
    let update =
        rt::block_on_all(client.fetch_update_hint(&id, BTreeSet::new(), true, false)).unwrap();

    let next = update.unwrap();
    assert_eq!(next.version, "30.20190725.0")
}

#[test]
fn test_downgrade() {
    let simple_graph = json!(
    {
      "nodes": [
        {
          "version": "30.20190725.0",
          "metadata": {
            "org.fedoraproject.coreos.scheme": "checksum",
            "org.fedoraproject.coreos.releases.age_index": "0"
          },
          "payload": "8b79877efa7ac06becd8637d95f8ca83aa385f89f383288bf3c2c31ca53216c7"
        },
        {
          "version": "0.0.0-mock",
          "metadata": {
            "org.fedoraproject.coreos.scheme": "checksum",
            "org.fedoraproject.coreos.releases.age_index": "1"
          },
          "payload": "sha-mock"
        }
      ],
      "edges": [
        [
          1,
          0
        ]
      ]
    });
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method("GET"),
            request::path(matches(r"^/v1/graph?.+$")),
            request::headers(contains_entry(("accept", matches("application/json")))),
        ])
        .times(2..=2)
        .respond_with(json_encoded(simple_graph)),
    );

    let id = Identity::mock_default();
    let client = Cincinnati {
        base_url: server.url_str("/"),
    };

    // Downgrades denied.
    let upgrade =
        rt::block_on_all(client.fetch_update_hint(&id, BTreeSet::new(), true, false)).unwrap();
    assert_eq!(upgrade, None);

    // Downgrades allowed.
    let downgrade =
        rt::block_on_all(client.fetch_update_hint(&id, BTreeSet::new(), true, true)).unwrap();

    let next = downgrade.unwrap();
    assert_eq!(next.version, "30.20190725.0")
}
