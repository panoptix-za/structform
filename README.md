StructForm
==========
[![build status](https://panoptix.semaphoreci.com/badges/structform/branches/main.svg)](https://panoptix.semaphoreci.com/projects/structform)

Library for managing interactive forms which encodes validation rules
using the type system.

## Why StructForm

Forms are a common way of capturing data in many programs. It's nice
in these programs if the form can give realtime feedback on the data
being captured. Additionally, if you have a separate frontend and
backend communicating over an API, it's good to keep the validation
rules enforced by the frontend and backend the same.

Rust has an excellent type system. StructForm is designed to leverage
the type system to give realtime feedback and enforce validation
rules.

StructForm was designed with web applications that follow the Elm
architecture in mind (specifically we use
[Seed](https://seed-rs.org/)), but it should work well with any
frontend framework.

## How Does it Work?

It all starts with a strongly typed version of the data you want your form to represent.

```rust
#[derive(Default, Debug, PartialEq, Eq)]
struct LoginData {
    username: String,
    password: String,
}
```

This is mirrorred by a Form struct, that derives the StructForm trait.

```rust
#[derive(Default, Clone, StructForm)]
#[structform(model = "LoginData")]
struct LoginForm {
    username: FormTextInput<String>,
    password: FormPasswordInput<String>,
}
```

This will do a few things:

- An enum will be created, named `LoginFormField` in this example,
  which has a case for each of your inputs.
- A `set_input` function is implemented on your form, which takes the
  field enum and a string and updates the appropriate field in your
  form.
- A `submit` function is implemented on your form, which checks all of
  the inputs on your form and, if possible, creates your strongly
  typed model (`LoginData` in this example).

In a typical web form, this is how you would hook it up:

- Have an HTML input for each field in your form.
- Add a listener to the [input
  event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLElement/input_event)
  on the inputs to call your form's `set_input`. Conveniently, all of
  your inputs can use the same event callback as long as they pass
  through the appropriate value from the form field enum.
- Add a listener to the [submit
  event](https://developer.mozilla.org/en-US/docs/Web/API/HTMLFormElement/submit_event)
  on your HTML form, which calls `submit` on your form and, if
  `submit` is successful, send your data on to an API.

The best way to learn to use StructForm is to look at the [examples](#Examples).

## Form Inputs

StructForm is unusual in that it doesn't come with form
inputs. Rather, it gives you macros that you use to derive your own
form inputs. This lets you specify which types of inputs you have in
your application, and how those different inputs might convert from
strings typed by your end users to strongly typed data differently.

The basic way of creating a form input is to call the
`derive_form_input` macro.

```rust
derive_form_input! {MyFormInput}
```

This will create a struct for you called `MyFormInput`, with an
`input` string that you tie to your UI. You can now specify how
`MyFormInput` handles any model type, including details like trimming
the input strings or not.

```rust
impl ParseAndFormat<MyInputModel> for MyFormInput<Model> {
    fn parse(value: &str) -> Result<MyInputModel, ParseError> { todo!() }
    fn format(value: &MyInputModel) -> String { todo!() }
}
```

For some common implementations of `ParseAndFormat`, see the macros
`impl_text_input_with_stringops` and
`impl_numeric_input_with_stringops`.

## Validation

Validation should be added by making the types that your form inputs
wrap enforce your validation rules. The [newtype
pattern](https://www.worthe-it.co.za/blog/2020-10-31-newtype-pattern-in-rust.html)
is an excellent way to add these rules. See the [validation rules
example](./structform/tests/validation_example.rs) for an example of
how this can be done.

## Examples

- [Basic login page](./structform/tests/login_example.rs)
- [Submit attempted tracking](./structform/tests/submit_attempted_example.rs)
- [Custom submit function](./structform/tests/custom_submit_function_example.rs)
- [Validation rules](./structform/tests/validation_example.rs)
- [Subforms and optional subforms](./structform/tests/subforms_example.rs)
- [List of subforms](./structform/tests/list_of_subforms_example.rs)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
