use rust_blog_backend::models::tutorial::{Tutorial, TutorialResponse, TutorialSummaryResponse};
use std::convert::TryFrom;

#[test]
fn test_tutorial_response_conversion() {
    let tutorial = Tutorial {
        id: "t1".to_string(),
        title: "Title".to_string(),
        description: "Desc".to_string(),
        icon: "icon".to_string(),
        color: "#fff".to_string(),
        topics: "[\"topic1\", \"topic2\"]".to_string(),
        content: "Content".to_string(),
        version: 1,
        created_at: "created".to_string(),
        updated_at: "updated".to_string(),
    };

    let response = TutorialResponse::try_from(tutorial).unwrap();
    assert_eq!(response.topics.len(), 2);
    assert_eq!(response.topics[0], "topic1");
    assert_eq!(response.id, "t1");
}

#[test]
fn test_tutorial_summary_conversion() {
    let tutorial = Tutorial {
        id: "t2".to_string(),
        title: "Title2".to_string(),
        description: "Desc2".to_string(),
        icon: "icon2".to_string(),
        color: "#000".to_string(),
        topics: "[]".to_string(),
        content: "Long Content".to_string(),
        version: 2,
        created_at: "c".to_string(),
        updated_at: "u".to_string(),
    };

    let summary = TutorialSummaryResponse::try_from(tutorial).unwrap();
    assert_eq!(summary.topics.len(), 0);
    assert_eq!(summary.id, "t2");
}

#[test]
fn test_tutorial_invalid_topics_json_fallback() {
    let tutorial = Tutorial {
        id: "t3".to_string(),
        title: "T".to_string(),
        description: "D".to_string(),
        icon: "i".to_string(),
        color: "c".to_string(),
        topics: "invalid json".to_string(),
        content: "C".to_string(),
        version: 1,
        created_at: "cr".to_string(),
        updated_at: "up".to_string(),
    };

    let response = TutorialResponse::try_from(tutorial).unwrap();
    assert!(response.topics.is_empty());
}
