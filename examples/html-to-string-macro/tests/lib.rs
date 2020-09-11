use html_to_string_macro::html_to_string;

#[test]
fn test() {
    let world = "world!";
    let hi = "hi!";
    assert_eq!(
        html_to_string! { <div hello={world}>{hi}</div> },
        r#"<div hello="world!">hi!</div>"#,
    );
}
