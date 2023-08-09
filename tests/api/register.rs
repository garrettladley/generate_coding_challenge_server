use maplit::hashmap;
use serde_json::Value;

use crate::helpers::spawn_app;

#[tokio::test]
async fn register_returns_a_200_for_valid_request_body() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/register", &app.address))
        .json(&hashmap! {
            "name" => "Garrett",
            "nuid" => "001234567",
        })
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response_json: Value = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    assert!(response_json["token"].is_string());
    assert!(response_json["challenge"].is_array());

    assert!(!response_json["token"].as_str().unwrap().is_empty());

    let num_mandatory = 7;
    let num_random = 100;
    assert_eq!(
        response_json["challenge"].as_array().unwrap().len(),
        num_mandatory + num_random
    );

    let saved = sqlx::query!("SELECT applicant_name, nuid FROM applicants",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved applicant.");

    assert_eq!(saved.applicant_name, "Garrett");
    assert_eq!(saved.nuid, "001234567");
}

#[tokio::test]
async fn register_returns_a_400_when_request_body_properties_are_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            hashmap! {
                "name" => "Garrett",
            },
            "missing the nuid",
        ),
        (
            hashmap! {
                "nuid" => "001234567",
            },
            "missing the name",
        ),
        (hashmap! {}, "missing both name and nuid"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/register", &app.address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn register_returns_a_400_when_fields_are_present_but_invalid() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (
            hashmap! {
                "name" => "",
                "nuid" => "001234567",
            },
            "empty name",
        ),
        (
            hashmap! {
                "name" => "Garrett",
                "nuid" => "",
            },
            "empty nuid",
        ),
        (
            hashmap! {
                "name" => "",
                "nuid" => "",
            },
            "empty name and nuid",
        ),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/register", &app.address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
