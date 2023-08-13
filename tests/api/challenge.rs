use crate::helpers::{register_sample_applicant, spawn_app};
use generate_coding_challenge_server::routes::challenge::ChallengeResponseData;
use generate_coding_challenge_server::routes::register::RegisterResponseData;

#[tokio::test]
async fn challenge_returns_a_200_for_token_that_exists() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let register_response = register_sample_applicant(&client, &app.address).await;

    assert_eq!(200, register_response.status().as_u16());

    let register_response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let challenge_response = client
        .get(&format!(
            "{}/challenge/{}",
            &app.address, &register_response.token
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, challenge_response.status().as_u16());

    let challenge_response: ChallengeResponseData =
        serde_json::from_str(&challenge_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    assert_eq!(challenge_response.challenge, register_response.challenge);
}

#[tokio::test]
async fn challenge_returns_a_400_for_invalid_uuid() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let bad_token = "67e55044-10b1-426f-9247-bb680e5fe0c80123456789";

    let response = client
        .get(&format!("{}/challenge/{}", &app.address, &bad_token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());

    let expected: serde_json::Value =
        serde_json::from_str(&format!("\"Invalid token! Given: {}\"", &bad_token)).unwrap();
    let actual: serde_json::Value =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn challenge_returns_a_404_for_token_that_does_not_exist_in_db() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let bad_token = "67e55044-10b1-426f-9247-bb680e5fe0c8";

    let response = client
        .get(&format!("{}/challenge/{}", &app.address, &bad_token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());

    let expected: String = serde_json::from_str(&format!(
        "\"Record associated with given token not found! Token: {}\"",
        &bad_token
    ))
    .unwrap();
    let actual: String = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    assert_eq!(expected, actual);
}
