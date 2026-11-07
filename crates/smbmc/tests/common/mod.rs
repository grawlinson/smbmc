//! Replays the betamax cassettes recorded for the Python library against a
//! local `wiremock` server.
//!
//! Every recorded interaction is mounted as a mock that must be hit exactly
//! once, with the same method, path, form body and `Cookie` header as the
//! original request, so the replay also checks login/session behaviour.

#![allow(dead_code)] // each test binary uses a different subset

use serde_json::Value;
use wiremock::matchers::{body_string, header, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

pub const USER: &str = "ipmi_user";
pub const PASS: &str = "ipmi_pass";

fn load(name: &str) -> Value {
    let path = format!(
        "{}/tests/fixtures/cassettes/Client_{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&text).unwrap()
}

fn fill(s: &str) -> String {
    s.replace("<USER>", USER).replace("<PASS>", PASS)
}

/// Start a mock BMC that replays the named cassette (e.g. `"login"`).
pub async fn replay(cassette: &str) -> MockServer {
    let server = MockServer::start().await;
    let cassette = load(cassette);

    for (i, interaction) in cassette["http_interactions"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let req = &interaction["request"];
        let res = &interaction["response"];

        let uri = req["uri"].as_str().unwrap();
        let req_path = uri.strip_prefix("<SERVER>").expect("uri uses <SERVER>");
        let mut mock = Mock::given(method(req["method"].as_str().unwrap()))
            .and(path(req_path))
            .and(body_string(fill(req["body"]["string"].as_str().unwrap())));
        match req["headers"]["Cookie"].as_array() {
            Some(cookies) => {
                mock = mock.and(header("cookie", cookies[0].as_str().unwrap()));
            }
            None => mock = mock.and(no_cookie),
        }

        let mut template = ResponseTemplate::new(res["status"]["code"].as_u64().unwrap() as u16)
            .set_body_string(res["body"]["string"].as_str().unwrap());
        for (name, values) in res["headers"].as_object().unwrap() {
            for value in values.as_array().unwrap() {
                template = template.append_header(name.as_str(), value.as_str().unwrap());
            }
        }

        mock.respond_with(template)
            .up_to_n_times(1)
            .expect(1)
            .named(format!("interaction {i}: POST {req_path}"))
            .mount(&server)
            .await;
    }
    server
}

fn no_cookie(request: &Request) -> bool {
    !request.headers.contains_key("cookie")
}

/// Login response that issues the given session ID.
pub fn login_ok(sid: &str) -> ResponseTemplate {
    ResponseTemplate::new(200)
        .append_header(
            "Set-Cookie",
            "SID=; expires=Thursday,01-Jan-1970 00:00:00 GMT; HttpOnly",
        )
        .append_header(
            "Set-Cookie",
            format!("SID={sid}; path=/ ; HttpOnly").as_str(),
        )
}

/// A recorded PMBus response body.
pub fn pmbus_body() -> String {
    std::fs::read_to_string(format!(
        "{}/tests/fixtures/ipmi_response_pmbus.xml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap()
}
