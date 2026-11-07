//! Async client tests, ported from `tests/integration/test_client.py` and
//! `tests/unit/test_client.py`.

mod common;

use std::time::Duration;

use common::{PASS, USER, login_ok, pmbus_body, replay};
use smbmc::{Client, Error, MetricKind, Metrics, SensorType};
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::new(&server.uri(), USER, PASS).unwrap()
}

#[tokio::test]
async fn login() {
    let server = replay("login").await;
    client(&server).login().await.unwrap();
}

#[tokio::test]
async fn bad_auth() {
    let server = replay("bad_auth").await;
    let client = Client::new(&server.uri(), "nonexistent_user", "nonexistent_password").unwrap();
    let err = client.login().await.unwrap_err();
    assert!(matches!(err, Error::AuthenticationFailed), "{err:?}");
}

#[tokio::test]
async fn sensor_metrics() {
    let server = replay("get_sensor_metrics").await;
    let client = client(&server);
    client.login().await.unwrap();
    let sensors = client.sensor_metrics().await.unwrap();
    assert_eq!(sensors.len(), 28);
    assert_eq!(sensors[0].name, "System Temp");
    assert_eq!(sensors[27].sensor_type, SensorType::PowerSupply);
}

#[tokio::test]
async fn pmbus_metrics() {
    let server = replay("get_pmbus_metrics").await;
    let client = client(&server);
    client.login().await.unwrap();
    assert_eq!(client.pmbus_metrics().await.unwrap().len(), 4);
}

/// No explicit login: the client logs in once and reuses the session.
#[tokio::test]
async fn metrics_logs_in_automatically() {
    let server = replay("get_metrics").await;
    let metrics = client(&server).metrics(&MetricKind::ALL).await.unwrap();
    assert_eq!(metrics.pmbus.map(|p| p.len()), Some(4));
    assert_eq!(metrics.sensor.map(|s| s.len()), Some(28));
}

/// Duplicates are ignored and unrequested kinds are left as `None`.
#[tokio::test]
async fn metrics_subset_and_duplicates() {
    let server = replay("get_pmbus_metrics").await;
    let metrics = client(&server)
        .metrics(&[MetricKind::Pmbus, MetricKind::Pmbus])
        .await
        .unwrap();
    assert_eq!(metrics.pmbus.as_ref().map(Vec::len), Some(4));
    assert_eq!(metrics.sensor, None);

    let server = MockServer::start().await; // no requests expected
    assert_eq!(
        client(&server).metrics(&[]).await.unwrap(),
        Metrics::default()
    );
}

#[tokio::test]
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

    let client = Client::builder(&server.uri(), USER, PASS)
        .session_timeout(Duration::ZERO)
        .build()
        .unwrap();
    client.pmbus_metrics().await.unwrap();
    client.pmbus_metrics().await.unwrap();
}

/// The BMC dropped the session (e.g. it rebooted) and answers with its login
/// page: the client logs in again and retries once.
#[tokio::test]
async fn dropped_session_relogs_and_retries_once() {
    // Establish a session ("sid").
    let server = MockServer::start().await;
    Mock::given(path("/cgi/login.cgi"))
        .respond_with(login_ok("sid"))
        .mount(&server)
        .await;
    Mock::given(path("/cgi/ipmi.cgi"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pmbus_body()))
        .mount(&server)
        .await;
    let client = client(&server);
    client.pmbus_metrics().await.unwrap();
    server.reset().await;

    // The BMC forgets "sid": one login page, then a new session ("sid2").

    Mock::given(path("/cgi/login.cgi"))
        .respond_with(login_ok("sid2"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/cgi/ipmi.cgi"))
        .and(header("cookie", "SID=sid2"))
        .respond_with(ResponseTemplate::new(200).set_body_string(pmbus_body()))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/cgi/ipmi.cgi"))
        .and(header("cookie", "SID=sid"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>login</html>"))
        .expect(1)
        .mount(&server)
        .await;
    assert_eq!(client.pmbus_metrics().await.unwrap().len(), 4);
}

/// A fresh session that still yields garbage is not retried.
#[tokio::test]
async fn fresh_session_is_not_retried() {
    let server = MockServer::start().await;
    Mock::given(path("/cgi/login.cgi"))
        .respond_with(login_ok("x"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(path("/cgi/ipmi.cgi"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>"))
        .expect(1)
        .mount(&server)
        .await;
    let err = client(&server).sensor_metrics().await.unwrap_err();
    assert!(matches!(err, Error::Xml(_)), "{err:?}");
}

#[tokio::test]
async fn http_error_status() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;
    let err = client(&server).login().await.unwrap_err();
    assert!(
        matches!(&err, Error::Http(e) if e.status().map(|s| s.as_u16()) == Some(500)),
        "{err:?}"
    );
}

#[test]
fn invalid_server() {
    let err = Client::new("192.168.1.1", USER, PASS).unwrap_err();
    assert!(matches!(err, Error::InvalidUrl(_)), "{err:?}");
}

#[test]
fn debug_redacts_password() {
    let client = Client::builder("https://bmc", USER, "hunter2")
        .request_timeout(Duration::from_secs(5))
        .accept_invalid_certs(true)
        .build()
        .unwrap();
    let text = format!("{client:?}");
    assert!(
        text.contains("logged_in: false") && !text.contains("hunter2"),
        "{text}"
    );
}
