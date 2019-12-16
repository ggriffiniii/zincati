use crate::fleet_lock::*;
use crate::identity::Identity;
use httptest::{mappers::*, responders::*, Expectation};
use serde_json::json;
use tokio::runtime::current_thread as rt;

#[test]
fn test_pre_reboot_lock() {
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method("POST"),
            request::path(format!("/{}", V1_PRE_REBOOT)),
            request::headers(contains_entry(("fleet-lock-protocol", "true"))),
            request::body(json_decoded(eq(json!({
                "client_params": {
                    "id": "e0f3745b108f471cbd4883c6fbed8cdd",
                    "group": "mock-workers",
                }
            })))),
        ])
        .respond_with(status_code(200)),
    );

    let id = Identity::mock_default();
    let client = ClientBuilder::new(server.url_str("/"), &id)
        .build()
        .unwrap();
    let res = rt::Runtime::new().unwrap().block_on(client.pre_reboot());

    let lock = res.unwrap();
    assert_eq!(lock, true);
}

#[test]
fn test_pre_reboot_error() {
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method("POST"),
            request::path(format!("/{}", V1_PRE_REBOOT)),
            request::headers(contains_entry(("fleet-lock-protocol", "true"))),
        ])
        .respond_with(
            http02::Response::builder()
                .status(404)
                .body(
                    json!({
                          "kind": "f1",
                          "value": "pre-reboot failure"
                    })
                    .to_string(),
                )
                .unwrap(),
        ),
    );

    let id = Identity::mock_default();
    let client = ClientBuilder::new(server.url_str("/"), &id)
        .build()
        .unwrap();
    let res = rt::Runtime::new().unwrap().block_on(client.pre_reboot());

    let _rejection = res.unwrap_err();
}

#[test]
fn test_steady_state_lock() {
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method("POST"),
            request::path(format!("/{}", V1_STEADY_STATE)),
            request::headers(contains_entry(("fleet-lock-protocol", "true"))),
            request::body(json_decoded(eq(json!({
              "client_params": {
                "id": "e0f3745b108f471cbd4883c6fbed8cdd",
                "group": "mock-workers"
              }
            }))))
        ])
        .respond_with(status_code(200)),
    );

    let id = Identity::mock_default();
    let client = ClientBuilder::new(server.url_str("/"), &id)
        .build()
        .unwrap();
    let res = rt::Runtime::new().unwrap().block_on(client.steady_state());

    let unlock = res.unwrap();
    assert_eq!(unlock, true);
}

#[test]
fn test_steady_state_error() {
    let server = httptest::Server::run();
    server.expect(
        Expectation::matching(all_of![
            request::method("POST"),
            request::path(format!("/{}", V1_STEADY_STATE)),
            request::headers(contains_entry(("fleet-lock-protocol", "true"))),
        ])
        .respond_with(
            http02::Response::builder()
                .status(404)
                .body(
                    json!({
                    "kind": "f1",
                    "value": "pre-reboot failure"
                      })
                    .to_string(),
                )
                .unwrap(),
        ),
    );

    let id = Identity::mock_default();
    let client = ClientBuilder::new(server.url_str("/"), &id)
        .build()
        .unwrap();
    let res = rt::Runtime::new().unwrap().block_on(client.steady_state());

    let _rejection = res.unwrap_err();
}
