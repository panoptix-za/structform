use structform::{
    derive_form_input, impl_text_input_with_stringops, ParseAndFormat, ParseError, StructForm,
};

// This example shows creating forms over nested lists of data structures.

// This example builds on the [login example](./login_example.rs).
// This example is written assuming that you're already familiar with
// the login example, so if not please refer to that first. It also
// helps to be familiar with the
// [subforms example](./subforms_example.rs).

// Often for larger forms, the strongly typed model isn't just a flat
// series of fields. It often has nested structs. Sometimes, you might
// need to enter an arbitrary number of those structs. In this case, a
// user may have many addresses.

#[derive(Default, Debug, PartialEq, Eq)]
struct UserDetails {
    username: String,
    addresses: Vec<Address>,
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
struct Address {
    street_address: String,
    city: String,
    country: String,
}

// When we create our StructForm for capturing these user details, we
// need a form for both UserDetails and Address. The Address form is
// included in the UserDetails form as a Vec of subforms. The
// derive macro can automatically identify Vecs as being Vecs of
// subforms, so no additional annotations are needed.

#[derive(Default, Clone, StructForm)]
#[structform(model = "UserDetails")]
struct UserDetailsForm {
    username: FormTextInput<String>,
    addresses: Vec<AddressForm>,
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
//     AddAddresses,
//     Addresses(usize, AddressFormField),
//     RemoveAddresses(usize),
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
fn the_list_of_subforms_starts_empty() {
    let form = UserDetailsForm::default();
    assert_eq!(form.addresses.len(), 0);
}

#[test]
fn subforms_can_be_modified_by_their_index() {
    let mut form = UserDetailsForm::default();

    // You can push a new entry onto a subform list with the add
    // field. The add field is always your subform field name with
    // `Add` in front, like `AddAddresses`. In this case, the string
    // passed to set_input is ignored. It works well if you tie this
    // message to an "Add" HTML button.
    form.set_input(UserDetailsFormField::AddAddresses, "".to_string());
    assert_eq!(form.addresses.len(), 1);

    // When you add a new form, it starts empty, and you can fill it
    // in by calling `set_input` with the appropriate index. Indexing
    // starts from 0, like the Vec holding the subforms. A useful
    // pattern to follow when building HTML templates here is to
    // iterate over the subform in your HTML, and make use of
    // [enumerate](https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html#method.enumerate)
    // for the indices.
    assert_eq!(form.addresses[0].city.input, "".to_string());
    form.set_input(
        UserDetailsFormField::Addresses(0, AddressFormField::City),
        "Johannesburg".to_string(),
    );
    assert_eq!(form.addresses[0].city.input, "Johannesburg".to_string());
}

#[test]
fn settings_an_out_of_range_input_does_nothing() {
    let mut form = UserDetailsForm::default();

    // It's probably a logic error to try to set a field that doesn't
    // exist, but if you do at least it won't error out.
    form.set_input(
        UserDetailsFormField::Addresses(1, AddressFormField::City),
        "Hello".to_string(),
    );

    assert_eq!(form.addresses.len(), 0);
}

#[test]
fn any_subform_can_be_removed_from_the_list() {
    // If you're editing an existing model, you can construct your
    // StructForm from that model. Subforms will also be prepopulated
    // appropriately.

    let model = UserDetails {
        username: "justin".to_string(),
        addresses: vec![
            Address {
                street_address: "123 StructForm Drive".to_string(),
                city: "Johannesburg".to_string(),
                country: "South Africa".to_string(),
            },
            Address {
                street_address: "321 StructForm Laan".to_string(),
                city: "Pretoria".to_string(),
                country: "South Africa".to_string(),
            },
            Address {
                street_address: "222 StructForm Crescent".to_string(),
                city: "Midrand".to_string(),
                country: "South Africa".to_string(),
            },
        ],
    };

    let mut form = UserDetailsForm::new(&model);

    assert_eq!(form.addresses.len(), 3);
    assert_eq!(form.addresses[0].city.input, "Johannesburg".to_string());
    assert_eq!(form.addresses[1].city.input, "Pretoria".to_string());
    assert_eq!(form.addresses[2].city.input, "Midrand".to_string());

    // If you want to remove one of the forms, you can send the
    // appropriate remove field to `set_input`. The remove field is
    // always your subform field name with `Remove` in front, like
    // `RemoveAddresses`. It works well to tie this to a remove HTML
    // button next to each subform in your HTML. Like with `Add`, the
    // string passed in here does nothing.
    form.set_input(UserDetailsFormField::RemoveAddresses(1), "".to_string());
    assert_eq!(form.addresses.len(), 2);
    assert_eq!(form.addresses[0].city.input, "Johannesburg".to_string());
    assert_eq!(form.addresses[1].city.input, "Midrand".to_string());
}

#[test]
fn the_whole_form_can_be_completed() {
    let mut form = UserDetailsForm::default();

    form.set_input(UserDetailsFormField::Username, "justin".to_string());

    // It's valid to have an empty list of subforms.
    assert_eq!(
        form.submit(),
        Ok(UserDetails {
            username: "justin".to_string(),
            addresses: vec![]
        })
    );

    // However, if you've added a subform to the list, it is required.
    form.set_input(UserDetailsFormField::AddAddresses, "".to_string());
    assert_eq!(form.submit(), Err(ParseError::Required));

    form.set_input(
        UserDetailsFormField::Addresses(0, AddressFormField::StreetAddress),
        "123 StructForm Drive".to_string(),
    );
    form.set_input(
        UserDetailsFormField::Addresses(0, AddressFormField::City),
        "Johannesburg".to_string(),
    );
    form.set_input(
        UserDetailsFormField::Addresses(0, AddressFormField::Country),
        "South Africa".to_string(),
    );

    assert_eq!(
        form.submit(),
        Ok(UserDetails {
            username: "justin".to_string(),
            addresses: vec![Address {
                street_address: "123 StructForm Drive".to_string(),
                city: "Johannesburg".to_string(),
                country: "South Africa".to_string(),
            }]
        })
    );

    form.set_input(UserDetailsFormField::AddAddresses, "".to_string());
    assert_eq!(form.submit(), Err(ParseError::Required));

    form.set_input(
        UserDetailsFormField::Addresses(1, AddressFormField::StreetAddress),
        "321 StructForm Laan".to_string(),
    );
    form.set_input(
        UserDetailsFormField::Addresses(1, AddressFormField::City),
        "Pretoria".to_string(),
    );
    form.set_input(
        UserDetailsFormField::Addresses(1, AddressFormField::Country),
        "South Africa".to_string(),
    );

    assert_eq!(
        form.submit(),
        Ok(UserDetails {
            username: "justin".to_string(),
            addresses: vec![
                Address {
                    street_address: "123 StructForm Drive".to_string(),
                    city: "Johannesburg".to_string(),
                    country: "South Africa".to_string(),
                },
                Address {
                    street_address: "321 StructForm Laan".to_string(),
                    city: "Pretoria".to_string(),
                    country: "South Africa".to_string(),
                }
            ],
        })
    );
}
