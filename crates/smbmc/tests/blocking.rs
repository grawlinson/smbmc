//! Blocking client tests. The mock server runs on a Tokio runtime; the
//! blocking client is driven from a separate thread.

#![cfg(feature = "blocking")]

mod common;

use std::time::Duration;

use common::{PASS, USER, login_ok, pmbus_body, replay};
use smbmc::blocking::Client;
use smbmc::{Error, MetricKind};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn run<T, F>(server: &MockServer, f: F) -> T
where
    F: FnOnce(Client) -> T + Send + 'static,
    T: Send + 'static,
{
    let uri = server.uri();
    tokio::task::spawn_blocking(move || f(Client::new(&uri, USER, PASS).unwrap()))
        .await
        .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
async fn login() {
    let server = replay("login").await;
    run(&server, |c| c.login()).await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_auth() {
    let server = replay("bad_auth").await;
    let uri = server.uri();
    let err = tokio::task::spawn_blocking(move || {
        Client::new(&uri, "nonexistent_user", "nonexistent_password")
            .unwrap()
            .login()
    })
    .await
    .unwrap()
    .unwrap_err();
    assert!(matches!(err, Error::AuthenticationFailed), "{err:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn sensor_metrics() {
    let server = replay("get_sensor_metrics").await;
    let sensors = run(&server, |c| {
        c.login()?;
        c.sensor_metrics()
    })
    .await
    .unwrap();
    assert_eq!(sensors.len(), 28);
}

#[tokio::test(flavor = "multi_thread")]
async fn pmbus_metrics() {
    let server = replay("get_pmbus_metrics").await;
    let psus = run(&server, |c| {
        c.login()?;
        c.pmbus_metrics()
    })
    .await
    .unwrap();
    assert_eq!(psus.len(), 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn metrics_logs_in_automatically() {
    let server = replay("get_metrics").await;
    let metrics = run(&server, |c| c.metrics(&MetricKind::ALL)).await.unwrap();
    assert_eq!(metrics.pmbus.map(|p| p.len()), Some(4));
    assert_eq!(metrics.sensor.map(|s| s.len()), Some(28));
}

#[tokio::test(flavor = "multi_thread")]
async fn expired_session_logs_in_again() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/cgi/login.cgi"))
        .respond_with(login_ok("fresh"))
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/cgi/ipmi.cgi"))
        .and(header("cookie", "SID=fresh"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pmbus_body()))
        .expect(2)
        .mount(&server)
        .await;

    let uri = server.uri();
    tokio::task::spawn_blocking(move || {
        let client = Client::builder(&uri, USER, PASS)
            .session_timeout(Duration::ZERO)
            .build_blocking()
            .unwrap();
        client.pmbus_metrics().unwrap();
        client.pmbus_metrics().unwrap();
        assert!(format!("{client:?}").starts_with("blocking::Client"));
    })
    .await
    .unwrap();
}

#[test]
fn invalid_server() {
    let err = Client::new("not a url", USER, PASS).unwrap_err();
    assert!(matches!(err, Error::InvalidUrl(_)), "{err:?}");
}

#[tokio::test(flavor = "multi_thread")]
async fn dropped_session_relogs_and_retries_once() {
    let server = MockServer::start().await;
    Mock::given(path("/cgi/login.cgi"))
        .respond_with(login_ok("sid"))
        .expect(2)
        .mount(&server)
        .await;
    // Served in priority order: data, then the login page, then data again.
    for (priority, body) in [
        (1, pmbus_body()),
        (2, "<html><body>login</body></html>".to_owned()),
        (3, pmbus_body()),
    ] {
        Mock::given(path("/cgi/ipmi.cgi"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .with_priority(priority)
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;
    }
    let counts = run(&server, |c| {
        let first = c.pmbus_metrics().unwrap().len();
        let second = c.pmbus_metrics().unwrap().len();
        (first, second)
    })
    .await;
    assert_eq!(counts, (4, 4));
}
