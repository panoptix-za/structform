use std::net::{IpAddr, Ipv4Addr};
use structform::{
    derive_form_input, impl_numeric_input_with_stringops, impl_text_input_with_stringops,
    ParseAndFormat, ParseError, StructForm,
};

// This example shows how to handle models that don't implement
// Default by providing a custom `submit` function.

// This example builds on the [login example](./login_example.rs).
// This example is written assuming that you're already familiar with
// the login example, so if not please refer to that first.

// We start with some strongly typed data. In this case, we're
// capturing network connection details. However, in this example, our
// struct does not implement Default!

#[derive(Debug, PartialEq, Eq)]
struct ConnectionDetails {
    ip: IpAddr,
    port: u16,
}

// The derived implementation of `submit` assumes that the model
// implements `Default`, so this is a problem. The solution is to
// provide a custom `submit_with` function.

#[derive(Default, Clone, StructForm)]
#[structform(model = "ConnectionDetails", submit_with = "submit_connection_details")]
struct ConnectionDetailsForm {
    ip: FormTextInput<IpAddr>,
    port: FormNumberInput<u16>,
}

fn submit_connection_details(
    form: &mut ConnectionDetailsForm,
) -> Result<ConnectionDetails, ParseError> {
    // Note that in a custom submit function, you should call submit
    // on all the required fields first, and then exit by returning
    // the error from any of them afterwords while constructing your
    // result. This is to make sure that `is_edited` is set correctly
    // on all the inputs.
    let ip = form.ip.submit();
    let port = form.port.submit();

    Ok(ConnectionDetails {
        ip: ip?,
        port: port?,
    })
}

// Out of the box, StructForm doesn't provide any inputs for us to put in
// our form. Luckily, it gives us the tools to derive our own.

// On this form, we have a text inputs, which are parsed into IpAddrs,
// and number inputs that are parsed into u16s.

derive_form_input! {FormTextInput}
impl_text_input_with_stringops!(FormTextInput, IpAddr);

derive_form_input! {FormNumberInput}
impl_numeric_input_with_stringops!(FormNumberInput, "a number", u16, u16);

#[test]
fn we_can_submit_our_form_using_the_custom_submit_function() {
    let mut form = ConnectionDetailsForm::default();

    form.set_input(ConnectionDetailsFormField::Ip, "127.0.0.1".to_string());
    form.set_input(ConnectionDetailsFormField::Port, "80".to_string());
    assert_eq!(
        form.submit(),
        Ok(ConnectionDetails {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: 80
        })
    );
}
