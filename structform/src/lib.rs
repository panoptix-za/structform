use std::fmt;

mod numeric_input;
mod text_input;

pub use numeric_input::*;
pub use text_input::*;

// Re-export this, so users don't need to explicitly depend on both crates.
pub use structform_derive::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Required,
    InvalidFormat {
        required_type: String,
    },
    FromStrError(String),
    NumberOutOfRange {
        required_type: String,
        min: String,
        max: String,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Required => write!(f, "This field is required."),
            ParseError::InvalidFormat { required_type } => write!(f, "Expected {}.", required_type),
            ParseError::FromStrError(error) => write!(f, "{}.", error),
            ParseError::NumberOutOfRange {
                required_type,
                min,
                max,
            } => write!(f, "Expected {} between {} and {}.", required_type, min, max),
        }
    }
}

pub trait StructForm<Model> {
    type Field;

    fn new(model: &Model) -> Self;
    fn set_input(&mut self, field: Self::Field, value: String);

    fn submit(&mut self) -> Result<Model, ParseError>;
    fn submit_update(&mut self, model: Model) -> Result<Model, ParseError>;
    fn submit_attempted(&self) -> bool;
    fn is_empty(&self) -> bool;

    fn has_unsaved_changes(&self, pristine: &Model) -> bool
    where
        Self: Clone,
        Model: Clone + PartialEq,
    {
        let mut tmp_form = self.clone();
        let updated_model = tmp_form.submit_update(pristine.clone());
        match updated_model {
            Ok(updated_model) => *pristine != updated_model,
            Err(_) => true,
        }
    }

    fn validation_error(&self) -> Option<ParseError>
    where
        Self: Clone,
    {
        // This is not an efficient implementation because it clones
        // the whole form. It would be better if we had a separate
        // immutable parse vs submit, or have some caching built into
        // the form (model: Option<Result<Model>> updated on each
        // input event?). It could be better to move this over to
        // structform_derive.
        if self.submit_attempted() {
            self.clone().submit().err()
        } else {
            None
        }
    }
}

/// Trait used to tie strongly typed models into form
/// inputs. Libraries must define their own form inputs (although
/// macros are provided to make this easy), and then implement
/// `ParseAndFormat` for their input for all supported types. In other
/// words, `impl ParseAndFormat<MyType> for MyTextInput<MyType>`.
pub trait ParseAndFormat<T> {
    fn parse(value: &str) -> Result<T, ParseError>;
    fn format(value: &T) -> String;
}

#[macro_export]
macro_rules! derive_form_input {
    ($input:ident) => {
        #[derive(Clone)]
        pub struct $input<T> {
            pub initial_input: String,
            pub input: String,
            pub value: Result<T, structform::ParseError>,
            pub is_edited: bool,
        }

        impl<T> Default for $input<T>
        where
            $input<T>: structform::ParseAndFormat<T>,
        {
            fn default() -> $input<T> {
                $input {
                    initial_input: String::new(),
                    input: String::new(),
                    value: $input::parse(""),
                    is_edited: false,
                }
            }
        }

        impl<T> $input<T> {
            pub fn show_validation_msg(&self) -> bool {
                self.is_edited && self.value.is_err()
            }

            pub fn validation_error(&self) -> Option<&structform::ParseError> {
                self.value
                    .as_ref()
                    .err()
                    .filter(|_| self.show_validation_msg())
            }

            pub fn is_empty(&self) -> bool {
                self.input.is_empty()
            }
        }

        #[allow(dead_code)]
        impl<T> $input<T>
        where
            $input<T>: structform::ParseAndFormat<T>,
            T: Clone,
        {
            pub fn new(value: &T) -> $input<T> {
                let initial_input = Self::format(value);
                $input {
                    initial_input: initial_input.clone(),
                    input: initial_input,
                    value: Ok(value.clone()),
                    is_edited: false,
                }
            }

            pub fn submit(&mut self) -> Result<T, structform::ParseError> {
                self.is_edited = true;
                self.value.clone()
            }

            pub fn set_input(&mut self, value: String) {
                self.value = Self::parse(&value);
                self.input = value;
                self.is_edited = true;
            }

            pub fn clear(&mut self) {
                self.initial_input = "".to_string();
                self.set_input("".to_string());
                self.is_edited = false;
            }
        }
    };
}
