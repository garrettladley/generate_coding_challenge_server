use maplit::hashmap;
use serde_json::Value;

use crate::helpers::spawn_app;

#[tokio::test]
async fn challenge_returns_a_200_for_token_that_exists() {
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

    let token = response_json["token"].as_str().unwrap();
    let challenge = response_json["challenge"].clone();

    let response = client
        .get(&format!("{}/challenge/{}", &app.address, &token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response_json: Value = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    assert_eq!(response_json["challenge"], challenge);
}

#[tokio::test]
async fn challenge_returns_a_400_for_invalid_uuid() {
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

    let bad_token = "67e55044-10b1-426f-9247-bb680e5fe0c8";

    let response = client
        .get(&format!("{}/challenge/{}", &app.address, &bad_token))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());

    let expected: serde_json::Value = serde_json::from_str(&format!(
        "\"Record associated with given token not found! Token: {}\"",
        &bad_token
    ))
    .unwrap();
    let actual: serde_json::Value =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    assert_eq!(expected, actual);
}
