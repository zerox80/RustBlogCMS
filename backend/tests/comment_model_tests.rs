use rust_blog_backend::models::comment::Comment;
use serde_json::json;

#[test]
fn test_comment_serialization() {
    let comment = Comment {
        id: "c1".to_string(),
        tutorial_id: Some("t1".to_string()),
        post_id: None,
        author: "Author".to_string(),
        content: "Content".to_string(),
        created_at: "2023-01-01".to_string(),
        votes: 10,
        is_admin: false,
    };

    let serialized = serde_json::to_string(&comment).unwrap();
    assert!(serialized.contains("\"id\":\"c1\""));
    assert!(serialized.contains("\"tutorial_id\":\"t1\""));
    assert!(serialized.contains("\"post_id\":null"));
}

#[test]
fn test_comment_deserialization() {
    let data = json!({
        "id": "c2",
        "tutorial_id": null,
        "post_id": "p1",
        "author": "User",
        "content": "Nice!",
        "created_at": "2023-02-01",
        "votes": 5,
        "is_admin": true
    });

    let comment: Comment = serde_json::from_value(data).unwrap();
    assert_eq!(comment.author, "User");
    assert!(comment.is_admin);
    assert_eq!(comment.post_id, Some("p1".to_string()));
}
