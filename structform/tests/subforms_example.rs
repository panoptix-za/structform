use structform::{
    derive_form_input, impl_text_input_with_stringops, ParseAndFormat, ParseError, StructForm,
};

// This example shows creating forms over nested data structures.

// This example builds on the [login example](./login_example.rs).
// This example is written assuming that you're already familiar with
// the login example, so if not please refer to that first.

// Often for larger forms, the strongly typed model isn't just a flat
// series of fields. It often has nested structs, like the addresses
// here.

#[derive(Default, Debug, PartialEq, Eq)]
struct UserDetails {
    username: String,
    primary_address: Address,
    secondary_address: Option<Address>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct Address {
    street_address: String,
    city: String,
    country: String,
}

// When we create our StructForm for capturing these user details, we
// need a form for both UserDetails and Address. The Address form is
// included in the UserDetails form as a subform. The derive macro can
// automatically identify optional subforms, but it needs the
// `#[structform(subform)]` annotation to help it identify required
// subforms.

#[derive(Default, Clone, StructForm)]
#[structform(model = "UserDetails")]
struct UserDetailsForm {
    username: FormTextInput<String>,
    #[structform(subform)]
    primary_address: AddressForm,
    secondary_address: Option<AddressForm>,
}

#[derive(Default, Clone, StructForm)]
#[structform(model = "Address")]
struct AddressForm {
    street_address: FormTextInput<String>,
    city: FormTextInput<String>,
    country: FormTextInput<String>,
}

// These two derivations of StructForms generates the following field definitions:
// ```
// pub enum UserDetailsFormField {
//     Username,
//     PrimaryAddress(AddressFormField),
//     ToggleSecondaryAddress,
//     SecondaryAddress(AddressFormField),
// }
// pub enum AddressFormField {
//     StreetAddress,
//     City,
//     Country,
// }
// ```

// These inputs are the same as the login example. See that example
// for more details.

derive_form_input! {FormTextInput}
impl_text_input_with_stringops!(FormTextInput, String);

#[test]
fn set_input_delegates_to_subform() {
    let mut form = UserDetailsForm::default();

    // The `UserDetailsFormField` has one field for the whole of a
    // required subform, that can contain any of the subform's fields.

    assert_eq!(form.primary_address.city.value, Err(ParseError::Required));
    form.set_input(
        UserDetailsFormField::PrimaryAddress(AddressFormField::City),
        "Johannesburg".to_string(),
    );
    assert_eq!(
        form.primary_address.city.value,
        Ok("Johannesburg".to_string())
    );
}

#[test]
fn optional_subforms_can_be_toggled_on_and_off() {
    let mut form = UserDetailsForm::default();

    // The `UserDetailsFormField` has two fields for an optional
    // subform: one that toggles it between `Some` and `None`, and
    // another that sends data.

    // By default, an optional subform will not be included.
    assert!(form.secondary_address.is_none());

    // You can send `set_update` for the secondary form, and it won't
    // crash, but it also won't do anything. Actually doing this is
    // probably a logic error in your frontend.
    form.set_input(
        UserDetailsFormField::SecondaryAddress(AddressFormField::City),
        "Johannesburg".to_string(),
    );
    assert!(form.secondary_address.is_none());

    // Rather before using the secondary address, you need to toggle
    // it to `Some`. In this case, the string passed to set_input is
    // ignored. It works well if you tie this message to the changed
    // event on an HTML checkbox, and only show the rest of the
    // secondary address in your HTML if `secondary_address` is Some.
    form.set_input(UserDetailsFormField::ToggleSecondaryAddress, "".to_string());
    assert!(form.secondary_address.is_some());
    assert_eq!(
        form.secondary_address.as_ref().unwrap().city.value,
        Err(ParseError::Required)
    );

    // Now that we've toggled secondary_address to Some, we can fill
    // in its fields.
    form.set_input(
        UserDetailsFormField::SecondaryAddress(AddressFormField::City),
        "Johannesburg".to_string(),
    );
    assert_eq!(
        form.secondary_address.as_ref().unwrap().city.value,
        Ok("Johannesburg".to_string())
    );
}

#[test]
fn the_subforms_are_populated_when_initializing_from_an_existing_model() {
    // If you're editing an existing model, you can construct your
    // StructForm from that model. Subforms will also be prepopulated
    // appropriately.

    let model = UserDetails {
        username: "justin".to_string(),
        primary_address: Address {
            street_address: "123 StructForm Drive".to_string(),
            city: "Johannesburg".to_string(),
            country: "South Africa".to_string(),
        },
        secondary_address: Some(Address {
            street_address: "321 StructForm Laan".to_string(),
            city: "Pretoria".to_string(),
            country: "South Africa".to_string(),
        }),
    };

    let form = UserDetailsForm::new(&model);

    assert_eq!(form.username.input, "justin".to_string());
    assert_eq!(
        form.primary_address.street_address.input,
        "123 StructForm Drive".to_string()
    );
    assert!(form.secondary_address.is_some());
    assert_eq!(
        form.secondary_address.unwrap().street_address.input,
        "321 StructForm Laan".to_string()
    );
}

#[test]
fn the_whole_form_can_be_completed() {
    let mut form = UserDetailsForm::default();

    form.set_input(UserDetailsFormField::Username, "justin".to_string());

    // Any required fields in subforms are also required to submit the
    // main form.
    assert_eq!(form.submit(), Err(ParseError::Required));

    form.set_input(
        UserDetailsFormField::PrimaryAddress(AddressFormField::StreetAddress),
        "123 StructForm Drive".to_string(),
    );
    form.set_input(
        UserDetailsFormField::PrimaryAddress(AddressFormField::City),
        "Johannesburg".to_string(),
    );
    form.set_input(
        UserDetailsFormField::PrimaryAddress(AddressFormField::Country),
        "South Africa".to_string(),
    );

    // Optional subforms are not required
    assert_eq!(
        form.submit(),
        Ok(UserDetails {
            username: "justin".to_string(),
            primary_address: Address {
                street_address: "123 StructForm Drive".to_string(),
                city: "Johannesburg".to_string(),
                country: "South Africa".to_string(),
            },
            secondary_address: None,
        })
    );

    // However, if an optional subform is toggled to Some, it is required.
    form.set_input(UserDetailsFormField::ToggleSecondaryAddress, "".to_string());
    assert_eq!(form.submit(), Err(ParseError::Required));

    form.set_input(
        UserDetailsFormField::SecondaryAddress(AddressFormField::StreetAddress),
        "321 StructForm Laan".to_string(),
    );
    form.set_input(
        UserDetailsFormField::SecondaryAddress(AddressFormField::City),
        "Pretoria".to_string(),
    );
    form.set_input(
        UserDetailsFormField::SecondaryAddress(AddressFormField::Country),
        "South Africa".to_string(),
    );

    assert_eq!(
        form.submit(),
        Ok(UserDetails {
            username: "justin".to_string(),
            primary_address: Address {
                street_address: "123 StructForm Drive".to_string(),
                city: "Johannesburg".to_string(),
                country: "South Africa".to_string(),
            },
            secondary_address: Some(Address {
                street_address: "321 StructForm Laan".to_string(),
                city: "Pretoria".to_string(),
                country: "South Africa".to_string(),
            }),
        })
    );
}
