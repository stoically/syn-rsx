use html_to_string_macro::html;

fn main() {
    html! {
        <!DOCTYPE html>
        <html>
            <head>
                <title>"Example"</title>
            </head>
            <body>
                <!-- "comment" -->
                    <div x=x a=2 x=3 >
                        <div>"1" { world.} "2" </div>
                    </div>
                        <div x=a some text as attribute flags and unclosed tag>  "2" </div>
                    <div x =a > some unquoted text with quotes "3". <div/></div>
                    <div {"some-attribute-from-rust-block"}/>
                    <div {"some-attribute-from-rust-blocks22".}>"3"</div>

                <div hello=world x=x >
                </br>
                </x>
            </body>
        </html>
    
    };
}