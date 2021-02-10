use structform::{
    derive_form_input, impl_text_input_with_stringops, ParseAndFormat, ParseError, StructForm,
};

// This example shows how StructForm can track if a form has been submitted yet.

// This example builds on the [login example](./login_example.rs).
// This example is written assuming that you're already familiar with
// the login example, so if not please refer to that first.

// Like in the login example, we start with some strongly typed
// data. This is probably a type that you can share with other parts
// of your system, like an API's input value.

#[derive(Default, Debug, PartialEq, Eq)]
struct LoginData {
    username: String,
    password: String,
}

// This StructForm has a new boolean flag, called `submit_attempted`,
// which is annotated with `#[structform(submit_attempted)]`. This
// lets the StructForm derive macro know that we want to track if this
// form has been submitted or not.

#[derive(Default, Clone, StructForm)]
#[structform(model = "LoginData")]
struct LoginForm {
    username: FormTextInput<String>,
    password: FormPasswordInput<String>,
    #[structform(submit_attempted)]
    submit_attempted: bool,
}

// These inputs are the same as the login example. See that example
// for more details.

derive_form_input! {FormTextInput}
impl_text_input_with_stringops!(FormTextInput, String);
derive_form_input! {FormPasswordInput}
impl ParseAndFormat<String> for FormPasswordInput<String> {
    fn parse(value: &str) -> Result<String, ParseError> {
        if value.is_empty() {
            Err(ParseError::Required)
        } else {
            Ok(value.into())
        }
    }

    fn format(value: &String) -> String {
        value.clone()
    }
}

#[test]
fn a_form_tracks_if_submit_is_attempted() {
    let mut form = LoginForm::default();

    // Initially, you haven't tried to submit a form.
    assert_eq!(form.submit_attempted, false);

    // When you try to submit the form, it flips the submit_attempted
    // flag. This can be particularly useful if you want to show a
    // validation message at the bottom of your form to point people
    // to errors further up. You don't want to show error messages
    // until the 'submit' button on the form has been clicked.
    let _parsed = form.submit();
    assert_eq!(form.submit_attempted, true);
}
