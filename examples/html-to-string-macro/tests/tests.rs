use html_to_string_macro::html;

#[test]
fn test() {
    let world = "planet";
    assert_eq!(
        html! {
            <!DOCTYPE html>
            <html>
                <head>
                    <title>"Example"</title>
                </head>
                <body>
                    <!-- "comment" -->
                    <div hello={world} />
                    <>
                        <div>"1"</div>
                        <div>"2"</div>
                        <div>"3"</div>
                        <div {"some-attribute-from-rust-block"}/>
                    </>
                </body>
            </html>
        },
        r#"
            <!DOCTYPE html>
            <html>
                <head>
                    <title>Example</title>
                </head>
                <body>
                    <!-- comment -->
                    <div hello="planet"></div>
                    <div>1</div>
                    <div>2</div>
                    <div>3</div>
                    <div some-attribute-from-rust-block></div>
                </body>
            </html>
        "#
        .split('\n')
        .map(|line| line.trim())
        .collect::<Vec<&str>>()
        .join("")
    );
}
