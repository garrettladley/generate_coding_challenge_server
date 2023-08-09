use maplit::hashmap;
use serde_json::Value;

use crate::helpers::spawn_app;
use generate_coding_challenge_server::domain::algo_question::one_edit_away;

#[tokio::test]
async fn submit_returns_a_200_for_correct_solution() {
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
    let challenge: Vec<String> = response_json["challenge"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str().map(String::from))
        .collect();

    let solution = challenge
        .iter()
        .filter_map(|case| one_edit_away(case))
        .map(|color| color.to_string())
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

    let response_json: Value = serde_json::from_str(&response.text().await.unwrap())
        .expect("Failed to parse response JSON");
    let correct = response_json["correct"].as_bool().unwrap();
    let message = response_json["message"].as_str().unwrap();

    assert!(correct);
    assert_eq!("Correct - nice work!", message);
}
