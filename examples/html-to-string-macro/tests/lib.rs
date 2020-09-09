use html_to_string_macro::html_to_string;

#[test]
fn test() {
    assert_eq!(
        html_to_string! { <div hello="world">"hi"</div> },
        "<div hello=\"world\">hi</div>",
    );
}
