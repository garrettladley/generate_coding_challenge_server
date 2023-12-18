use generate_coding_challenge_server::routes::RegisterResponseData;
use maplit::hashmap;

use crate::helpers::{register_sample_applicant, spawn_app};

#[tokio::test]
async fn register_returns_a_200_for_valid_request_body() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, response.status().as_u16());

    let response: RegisterResponseData = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let num_mandatory = 4;
    let num_random = 256;
    assert_eq!(response.challenge.len(), num_mandatory + num_random);

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

    for (invalid_body, reason) in test_cases {
        let response = client
            .post(&format!("{}/register", &app.address))
            .json(&invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when {}.",
            reason,
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
            "Invalid name! Given: ",
        ),
        (
            hashmap! {
                "name" => "Garrett",
                "nuid" => "",
            },
            "Invalid NUID! Given: ",
        ),
        (
            hashmap! {
                "name" => "",
                "nuid" => "",
            },
            "Invalid name! Given: ",
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
        let expected: String = serde_json::from_str(&format!("\"{}\"", error_message)).unwrap();
        let actual: String = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
        assert_eq!(expected, actual);
    }
}

#[tokio::test]
async fn register_returns_a_409_for_user_that_already_exists() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, response.status().as_u16());

    let response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(409, response.status().as_u16());

    let response_json: String = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    assert_eq!(
        response_json,
        "NUID 001234567 has already registered! Use the forgot-token endpoint to retrieve your token."
    );
}
