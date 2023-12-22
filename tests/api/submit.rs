use maplit::hashmap;
use serde_json::Value;

use crate::helpers::{register_sample_applicant, spawn_app};
use generate_coding_challenge_server::{
    domain::algo_question::parse_barcode,
    routes::{RegisterResponseData, SubmitResponseData},
};

#[tokio::test]
async fn submit_returns_a_200_for_correct_solution() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let register_response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, register_response.status().as_u16());

    let response_json: Value = serde_json::from_str(&register_response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let token = response_json["token"].as_str().unwrap();
    let challenge: Vec<String> = response_json["challenge"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str().map(String::from))
        .collect();

    let solution = challenge
        .iter()
        .map(|case| parse_barcode(case))
        .collect::<Vec<String>>();

    let solution_json: serde_json::Value = serde_json::Value::Array(
        solution
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect(),
    );

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response: SubmitResponseData = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let correct = response.correct;
    let message = response.message;

    assert!(correct);
    assert_eq!("Correct - nice work!", message);

    let saved = sqlx::query!("SELECT nuid, correct FROM submissions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved applicant.");

    assert_eq!(saved.nuid, "001234567");
    assert!(saved.correct);
}

#[tokio::test]
async fn submit_returns_a_200_for_incorrect_solution() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let register_response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, register_response.status().as_u16());

    let response_json: Value = serde_json::from_str(&register_response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let token = response_json["token"].as_str().unwrap();

    let solution_json: serde_json::Value = serde_json::Value::Array(Vec::new());

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response: SubmitResponseData = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let correct = response.correct;
    let message = response.message;

    assert!(!correct);
    assert_eq!("Incorrect Solution", message);

    let saved = sqlx::query!("SELECT nuid, correct FROM submissions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved applicant.");

    assert_eq!(saved.nuid, "001234567");
    assert!(!saved.correct);
}

#[tokio::test]
async fn submit_returns_a_400_for_bad_request() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let register_response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, register_response.status().as_u16());

    let response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let token = response.token;

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&hashmap! {
            "name" => "Garrett",
            "nuid" => "001234567",
        })
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn submit_returns_a_404_for_user_that_does_not_exist() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let solution_json: serde_json::Value = serde_json::Value::Array(Vec::new());

    let bad_token = "67e55044-10b1-426f-9247-bb680e5fe0c8";

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &bad_token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());

    assert_eq!(
        format!(
            "Record associated with given token not found! Token: {}",
            &bad_token
        ),
        response.text().await.unwrap()
    );
}

#[tokio::test]
async fn submit_correct_then_incorrect_results_in_incorrect() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let register_response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, register_response.status().as_u16());

    let response_json: Value = serde_json::from_str(&register_response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let token = response_json["token"].as_str().unwrap();
    let challenge: Vec<String> = response_json["challenge"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str().map(String::from))
        .collect();

    let solution = challenge
        .iter()
        .map(|case| parse_barcode(case))
        .collect::<Vec<String>>();

    let solution_json: serde_json::Value = serde_json::Value::Array(
        solution
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect(),
    );

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response: SubmitResponseData = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let correct = response.correct;
    let message = response.message;

    assert!(correct);
    assert_eq!("Correct - nice work!", message);

    let saved = sqlx::query!("SELECT nuid, correct FROM submissions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved applicant.");

    assert_eq!(saved.nuid, "001234567");
    assert!(saved.correct);

    let solution_json: serde_json::Value = serde_json::Value::Array(Vec::new());

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response: SubmitResponseData = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    let correct = response.correct;
    let message = response.message;

    assert!(!correct);
    assert_eq!("Incorrect Solution", message);

    let most_recent_sub = sqlx::query!(
        "SELECT nuid, correct FROM submissions
        ORDER BY submission_time DESC",
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved applicant.");

    assert_eq!(most_recent_sub.nuid, "001234567");
    assert!(!most_recent_sub.correct);
}
