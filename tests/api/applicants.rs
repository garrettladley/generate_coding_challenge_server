use crate::helpers::{register_sample_applicant_with_nuid, spawn_app};
use generate_coding_challenge_server::{
    domain::{algo_question::parse_barcode, Nuid},
    routes::{applicants::ApplicantsResponseData, RegisterResponseData},
};

#[tokio::test]
async fn applicants_returns_a_200_for_valid_nuids_that_exist() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid1 = "001234567";
    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid1).await;

    assert_eq!(200, register_response.status().as_u16());

    let response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let token = response.token;
    let challenge = response.challenge;

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

    let nuid2 = "000000000";

    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid2).await;

    assert_eq!(200, register_response.status().as_u16());

    let response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let token = response.token;

    let solution_json: serde_json::Value = serde_json::Value::Array(Vec::new());

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let nuid3 = "007654321";
    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid3).await;

    assert_eq!(200, register_response.status().as_u16());

    let body: serde_json::Value =
        serde_json::Value::Array(vec![nuid1.into(), nuid2.into(), nuid3.into()]);

    let response = client
        .get(&format!("{}/applicants", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let actual: ApplicantsResponseData =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();

    assert!(actual.applicants_found.len() == 2);
    assert!(actual.applicants_not_submitted.len() == 1);
    assert!(actual.applicants_not_found.is_empty());

    let nuid1 = Nuid::parse(nuid1).unwrap();
    assert!(actual
        .applicants_found
        .iter()
        .any(|a| a.nuid.as_ref() == nuid1.as_ref()));
    assert!(
        actual
            .applicants_found
            .iter()
            .find(|a| a.nuid.as_ref() == nuid1.as_ref())
            .unwrap()
            .correct
    );

    let nuid2 = Nuid::parse(nuid2).unwrap();
    assert!(actual
        .applicants_found
        .iter()
        .any(|a| a.nuid.as_ref() == nuid2.as_ref()));
    assert!(
        !actual
            .applicants_found
            .iter()
            .find(|a| a.nuid.as_ref() == nuid2.as_ref())
            .unwrap()
            .correct
    );

    let nuid3 = Nuid::parse(nuid3).unwrap();
    assert!(actual
        .applicants_not_submitted
        .iter()
        .any(|a| a == nuid3.as_ref()));
}

#[tokio::test]
async fn applicants_returns_a_400_for_invalid_nuids() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body: serde_json::Value = serde_json::Value::Array(vec![
        "".into(),
        "bad-nuid".into(),
        "foo".into(),
        "bar".into(),
        "baz".into(),
        "fizz".into(),
        "buzz".into(),
    ]);

    let response = client
        .get(&format!("{}/applicants", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(400, response.status().as_u16());
}

#[tokio::test]
async fn applicants_returns_a_404_for_a_mix_of_valid_nuids_that_do_and_dont_exist() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid1 = "001234567";
    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid1).await;

    assert_eq!(200, register_response.status().as_u16());

    let response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let token = response.token;
    let challenge = response.challenge;

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

    let nuid2 = "000000000";

    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid2).await;

    assert_eq!(200, register_response.status().as_u16());

    let response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let token = response.token;

    let solution_json: serde_json::Value = serde_json::Value::Array(Vec::new());

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let nuid3 = "007654321";
    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid3).await;

    assert_eq!(200, register_response.status().as_u16());

    let nuid4 = "000000001";

    let body: serde_json::Value =
        serde_json::Value::Array(vec![nuid1.into(), nuid2.into(), nuid3.into(), nuid4.into()]);

    let response = client
        .get(&format!("{}/applicants", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());

    let actual: ApplicantsResponseData =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();

    assert!(actual.applicants_found.len() == 2);
    assert!(actual.applicants_not_submitted.len() == 1);
    assert!(actual.applicants_not_found.len() == 1);

    let nuid1 = Nuid::parse(nuid1).unwrap();
    assert!(actual
        .applicants_found
        .iter()
        .any(|a| a.nuid.as_ref() == nuid1.as_ref()));
    assert!(
        actual
            .applicants_found
            .iter()
            .find(|a| a.nuid.as_ref() == nuid1.as_ref())
            .unwrap()
            .correct
    );

    let nuid2 = Nuid::parse(nuid2).unwrap();
    assert!(actual
        .applicants_found
        .iter()
        .any(|a| a.nuid.as_ref() == nuid2.as_ref()));
    assert!(
        !actual
            .applicants_found
            .iter()
            .find(|a| a.nuid.as_ref() == nuid2.as_ref())
            .unwrap()
            .correct
    );

    let nuid3 = Nuid::parse(nuid3).unwrap();
    assert!(actual
        .applicants_not_submitted
        .iter()
        .any(|a| a == nuid3.as_ref()));

    let nuid4 = Nuid::parse(nuid4).unwrap();
    assert!(actual
        .applicants_not_found
        .iter()
        .any(|a| a == nuid4.as_ref()));
}

#[tokio::test]
async fn applicants_returns_a_404_for_nuids_that_dont_exist() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid1 = "001234567";

    let nuid2 = "000000000";

    let nuid3 = "007654321";

    let body: serde_json::Value =
        serde_json::Value::Array(vec![nuid1.into(), nuid2.into(), nuid3.into()]);

    let response = client
        .get(&format!("{}/applicants", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(404, response.status().as_u16());

    let actual: ApplicantsResponseData =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();

    assert!(actual.applicants_found.is_empty());
    assert!(actual.applicants_not_submitted.is_empty());
    assert!(actual.applicants_not_found.len() == 3);

    let nuid1 = Nuid::parse(nuid1).unwrap();
    assert!(actual
        .applicants_not_found
        .iter()
        .any(|a| a == nuid1.as_ref()));

    let nuid2 = Nuid::parse(nuid2).unwrap();
    assert!(actual
        .applicants_not_found
        .iter()
        .any(|a| a == nuid2.as_ref()));

    let nuid3 = Nuid::parse(nuid3).unwrap();
    assert!(actual
        .applicants_not_found
        .iter()
        .any(|a| a == nuid3.as_ref()));
}

#[tokio::test]
async fn applicants_when_submit_correct_then_incorrect_results_in_incorrect() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let nuid1 = "001234567";
    let register_response = register_sample_applicant_with_nuid(&client, &app.address, nuid1).await;

    assert_eq!(200, register_response.status().as_u16());

    let response: RegisterResponseData =
        serde_json::from_str(&register_response.text().await.unwrap())
            .expect("Failed to parse response JSON");

    let token = response.token;
    let challenge = response.challenge;

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

    let solution_json: serde_json::Value = serde_json::Value::Array(Vec::new());

    let response = client
        .post(&format!("{}/submit/{}", &app.address, &token))
        .json(&solution_json)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let body: serde_json::Value = serde_json::Value::Array(vec![nuid1.into()]);

    let response = client
        .get(&format!("{}/applicants", &app.address))
        .json(&body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let actual: ApplicantsResponseData =
        serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();

    assert!(actual.applicants_found.len() == 1);
    assert!(actual.applicants_not_submitted.is_empty());
    assert!(actual.applicants_not_found.is_empty());

    let nuid1 = Nuid::parse(nuid1).unwrap();
    assert!(actual
        .applicants_found
        .iter()
        .any(|a| a.nuid.as_ref() == nuid1.as_ref()));

    assert!(
        !actual
            .applicants_found
            .iter()
            .find(|a| a.nuid.as_ref() == nuid1.as_ref())
            .unwrap()
            .correct
    );
}
