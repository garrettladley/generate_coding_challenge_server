use maplit::hashmap;
use serde_json::Value;

use crate::helpers::spawn_app;

#[tokio::test]
async fn forgot_token_returns_a_200_for_nuid_that_exists() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid = "001234567";

    let response = client
        .post(&format!("{}/register", &app.address))
        .json(&hashmap! {
            "name" => "Garrett",
            "nuid" => nuid,
        })
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let token = response
        .json::<Value>()
        .await
        .expect("Failed to deserialize response body.")
        .get("token")
        .expect("Token not found in response body.")
        .as_str()
        .expect("Token is not a string.")
        .to_owned();

    let response = client
        .get(&format!("{}/forgot_token/{}", &app.address, &nuid))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let response_json: Value = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");

    assert_eq!(response_json["token"], token);
}

#[tokio::test]
async fn forgot_token_returns_a_400_for_invalid_nuid() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid = "001234567";

    let response = client
        .post(&format!("{}/register", &app.address))
        .json(&hashmap! {
            "name" => "Garrett",
            "nuid" => nuid,
        })
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let bad_nuid = "a".repeat(9);

    let response = client
        .get(&format!("{}/forgot_token/{}", &app.address, &bad_nuid))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
    let expected: serde_json::Value =
        serde_json::from_str(&format!("\"Invalid NUID! Given: {}\"", &bad_nuid)).unwrap();
    let actual: serde_json::Value =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    assert_eq!(expected, actual);
}

#[tokio::test]
async fn forgot_token_returns_a_404_for_nuid_that_does_not_exist_in_db() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid = "001234567";

    let response = client
        .post(&format!("{}/register", &app.address))
        .json(&hashmap! {
            "name" => "Garrett",
            "nuid" => nuid,
        })
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let bad_nuid = "0".repeat(9);

    let response = client
        .get(&format!("{}/forgot_token/{}", &app.address, &bad_nuid))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());

    let expected: serde_json::Value = serde_json::from_str(&format!(
        "\"Record associated with given NUID not found! NUID: {}\"",
        &bad_nuid
    ))
    .unwrap();
    let actual: serde_json::Value =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
    assert_eq!(expected, actual);
}
