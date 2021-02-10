use std::convert::TryFrom;
use std::fmt;
use std::net::{IpAddr, Ipv4Addr};
use structform::{
    derive_form_input, impl_numeric_input_with_stringops, impl_text_input_with_stringops,
    ParseAndFormat, ParseError, StructForm,
};

// This example shows how to add custom validation logic to your
// model, and have it reflect on your form.

// This example builds on the [login example](./login_example.rs).
// This example is written assuming that you're already familiar with
// the login example, so if not please refer to that first.

// We start with some strongly typed data. In this case, we're
// capturing network connection details. However, in this example, our
// struct does not implement Default!

#[derive(Debug, PartialEq, Eq)]
struct ConnectionDetails {
    ip: IpAddr,
    port: Port,
}

// We have specific validation rules on ports. Specifically, it can't
// be zero, and it must fit within a u16. We create a Port
// [newtype](https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html)
// to represent these rules. The newtype implements Display, which is
// used to put existing values into inputs, and TryFrom<u16> which
// we'll use to generate our parsing logic for the newtype. This same
// validation logic can be enforced on your API endpoints
// automatically if you use Serde, and can also be enforced on CLI
// interfaces using StructOpt.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Port(u16);
impl Port {
    pub const MIN: u16 = 1;
    pub const MAX: u16 = std::u16::MAX;
}

impl fmt::Display for Port {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u16> for Port {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value >= Self::MIN && value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(format!("Expected a port between {} and {}", Self::MIN, Self::MAX).into())
        }
    }
}

#[derive(Default, Clone, StructForm)]
#[structform(model = "ConnectionDetails", submit_with = "submit_connection_details")]
struct ConnectionDetailsForm {
    ip: FormTextInput<IpAddr>,
    port: FormNumberInput<Port>,
}

// See the [custom submit function example](./custom_submit_function_example.rs) for details on the custom submit function.

fn submit_connection_details(
    form: &mut ConnectionDetailsForm,
) -> Result<ConnectionDetails, ParseError> {
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
// and number inputs that are parsed into Ports (via Port's TryFrom<u16> function).

derive_form_input! {FormTextInput}
impl_text_input_with_stringops!(FormTextInput, IpAddr);

derive_form_input! {FormNumberInput}
impl_numeric_input_with_stringops!(FormNumberInput, "a port", Port, u16, Port::MIN, Port::MAX);

#[test]
fn if_our_custom_type_is_not_a_number_a_generic_validation_message() {
    let mut form = ConnectionDetailsForm::default();

    form.set_input(ConnectionDetailsFormField::Port, "Eighty".to_string());

    // If what you enter isn't a number at all, then you'll get a
    // generic NumberOutOfRange error. We gave our derived input for
    // port the numeric range of prts so it can include them in the
    // error message.
    assert_eq!(
        form.port.submit(),
        Err(ParseError::NumberOutOfRange {
            required_type: "a port".to_string(),
            min: "1".to_string(),
            max: "65535".to_string()
        })
    );

    // ParseError implements Display itself, which is convenient for
    // showing validation messages in realtime, using each input's
    // `validation_error` function.
    assert_eq!(
        form.port.validation_error().map(|e| e.to_string()),
        Some("Expected a port between 1 and 65535.".to_string())
    );
}

#[test]
fn if_our_custom_type_is_out_of_range_we_see_our_validation_message() {
    let mut form = ConnectionDetailsForm::default();

    form.set_input(ConnectionDetailsFormField::Port, "0".to_string());

    // If the value is a number, it will call our TryFrom<u16>
    // function, and return an error if it fails the validation rules.
    assert_eq!(
        form.port.submit(),
        Err(ParseError::FromStrError(
            "Expected a port between 1 and 65535".to_string()
        ))
    );

    // ParseError implements Display itself, which is convenient for
    // showing validation messages in realtime, using each input's
    // `validation_error` function.
    assert_eq!(
        form.port.validation_error().map(|e| e.to_string()),
        Some("Expected a port between 1 and 65535.".to_string())
    );
}

#[test]
fn filling_in_the_form_correctly_submits_successfully() {
    let mut form = ConnectionDetailsForm::default();

    // If the form is filled in correctly, we can successfully `submit`.

    form.set_input(ConnectionDetailsFormField::Ip, "127.0.0.1".to_string());
    form.set_input(ConnectionDetailsFormField::Port, "80".to_string());

    assert_eq!(form.port.submit(), Ok(Port(80)));

    assert_eq!(
        form.submit(),
        Ok(ConnectionDetails {
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            port: Port(80)
        })
    );
}
