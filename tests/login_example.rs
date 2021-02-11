use structform::{
    derive_form_input, impl_text_input_with_stringops, ParseAndFormat, ParseError, StructForm,
};

// This example shows the basic use of StructForm with a simple login form.

// Using StructForms starts with some strongly typed data. This is
// probably a type that you can share with other parts of your system,
// like an API's input value.

#[derive(Default, Debug, PartialEq, Eq)]
struct LoginData {
    username: String,
    password: String,
}

// We create a StructForm, which is the bridge between where the user is
// typing and our strongly typed data model. You can implement the
// StructForm trait by hand, but usually it's easier to derive it.

#[derive(Default, Clone, StructForm)]
#[structform(model = "LoginData")]
struct LoginForm {
    username: FormTextInput<String>,
    password: FormPasswordInput<String>,
}

// Apart from deriving the StructForm trait, this will also create an
// enum for us to refer to the various fields. The derived code will look like this:
// ```
// pub enum LoginFormField {
//     Username,
//     Password,
// }
// ```
// We'll be using this form field enum later.

// Out of the box, StructForm doesn't provide any inputs for us to put in
// our form. Luckily, it gives us the tools to derive our own.

// You first need to derive_form_input to create a new form input
// type. This is a simple struct that can hold the string the user is
// typing, and can keep track of some other important form input
// state. These form inputs are duck typed into the StructForm, so it's
// best if you use the macro and don't implement it yourself from
// scratch.

derive_form_input! {FormTextInput}

// On its own, this FormTextInput doesn't know how to handle any of
// the strongly typed model fields. We need to implement
// `ParseAndFormat` for the combination of our model type and our
// input type. There are some macros to do this in terms of ToString
// and FromStr.

impl_text_input_with_stringops!(FormTextInput, String);

// In this case, we have two types of input: the username goes in a
// text input and the password goes in a password input. So we derive
// another type of input for handling passwords.

derive_form_input! {FormPasswordInput}

// Our password input doesn't match the default ParseAndFormat
// implementation that the macros provide, so we implement it by
// hand. Specifically, you'd usually want to trim text inputs to
// remove leading and trailing whitespace, so that's what the macro
// does, but that isn't appropriate for passwords.

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

// With all of our types in place, we can start doing things with our
// StructForm. It's designed to work well using a frontend framework
// inspired by the Elm architecture, like Seed.

#[test]
fn a_form_can_be_initialized_with_default_data() {
    // This is how you'll initialize the form when someone's creating
    // something from scratch.
    let mut form = LoginForm::default();
    // When they're ready to click the 'submit' button, you call
    // 'submit' to try to parse the form into your strongly typed model.
    let model = form.submit();
    // In this case, both username and password are required
    // fields. Since we haven't entered those values, we get a parse
    // error.
    assert_eq!(model, Err(ParseError::Required));
}

#[test]
fn a_forms_inputs_are_changed_by_sending_messages() {
    let mut form = LoginForm::default();

    // Our FormTextInput can be thought of as a pipe with two
    // sides. The one side, called `input` always exposes its value as
    // a string. You can bind this to HTML inputs. The other side,
    // called `value`, is the strongly typed representation. It's a
    // `Result`, because you might not be able to parse the input
    // string to a valid value. In this example, our username is also
    // a string, but our derived ParseAndFormat's parse will trim the
    // input string and then insist that it's non-empty.
    assert_eq!(form.username.input, "".to_string());
    assert_eq!(form.username.value, Err(ParseError::Required));
    assert_eq!(form.username.is_edited, false);

    // Frameworks like Seed have their HTML inputs trigger events when
    // the user types in them. When we derived StructForm for our
    // LoginForm, it also generated a message type that we can use to
    // refer to its fields: LoginFormField.
    form.set_input(LoginFormField::Username, "  hello".to_string());

    // This updated our input's value. The `input` side is exactly the
    // string we passed into `set_input`. The value side has gone
    // through our input's parsing logic which, in this case, trimmed
    // the input.
    assert_eq!(form.username.input, "  hello".to_string());
    assert_eq!(form.username.value, Ok("hello".to_string()));
    // This also updated our input's tracking on if it's been edited or
    // not, so we can show validation errors if needed.
    assert_eq!(form.username.is_edited, true);

    // If we fill in the rest of our form in the same way, then when
    // we call form.submit() we will get a successful response.
    form.set_input(LoginFormField::Password, "adm1n".to_string());
    let parsed = form.submit();
    assert_eq!(
        parsed,
        Ok(LoginData {
            username: "hello".into(),
            password: "adm1n".into()
        })
    );
}

#[test]
fn a_form_can_be_initialized_from_an_existing_model() {
    // Sometimes you already have a model that you're updating. Login
    // is a bad example for this, since you'll almost always want to
    // make people enter a login form from scratch, but most other
    // 'edit' forms fit this description.
    let existing_model = LoginData {
        username: "admin".into(),
        password: "admin".into(),
    };

    // You can initialize the form from the existing model.
    let mut form = LoginForm::new(&existing_model);

    // In this case, the username hasn't been edited, even though it
    // already has a value. Our `input` here comes from our
    // `ParseAndFormat`'s format function.
    assert_eq!(form.username.is_edited, false);
    assert_eq!(form.username.input, "admin".to_string());
    assert_eq!(form.username.value, Ok("admin".to_string()));

    // Usually, the user would then do some things to change the form.
    form.set_input(LoginFormField::Password, "adm1n".to_string());

    // When you're ready to submit the form, we can give the form a
    // strongly typed model for it to update. This takes ownership of
    // the strongly typed model you pass it.
    let updated_model = form.submit_update(existing_model);
    assert_eq!(
        updated_model,
        Ok(LoginData {
            username: "admin".into(),
            password: "adm1n".into()
        })
    );

    // This is a useful pattern to follow if the strongly typed model
    // has more fields than the structform. Fields on the existing model
    // that aren't in the form are passed through unchanged.
}

#[test]
fn form_inputs_track_if_submit_is_attempted() {
    let mut form = LoginForm::default();

    // Initially, all of your 'required' fields on your form are
    // probably in an invalid state. However, you don't want to show
    // error messages until someone has actually interacted with a
    // field.
    assert_eq!(form.username.is_edited, false);
    assert_eq!(form.password.is_edited, false);

    // When they try to submit the form, all fields are marked as
    // edited so that their errors will show.
    let _parsed = form.submit();
    assert_eq!(form.username.is_edited, true);
    assert_eq!(form.password.is_edited, true);
}
