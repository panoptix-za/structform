#[macro_export]
macro_rules! impl_numeric_input_with_stringops {
    ($numeric_input: ident, $type_name: literal, $type: ty, $underlying_numeric_type: ty) => {
        impl_numeric_input_with_stringops!(
            $numeric_input,
            $type_name,
            $type,
            $underlying_numeric_type,
            <$type>::MIN,
            <$type>::MAX
        );
    };
    ($numeric_input: ident, $type_name: literal, $type: ty, $underlying_numeric_type: ty, $min: expr, $max: expr) => {
        impl structform::ParseAndFormat<$type> for $numeric_input<$type> {
            fn parse(value: &str) -> Result<$type, ParseError> {
                use std::convert::TryFrom;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Err(ParseError::Required)
                } else {
                    trimmed
                        .parse::<$underlying_numeric_type>()
                        .map_err(|_e| ParseError::NumberOutOfRange {
                            required_type: $type_name.to_string(),
                            min: $min.to_string(),
                            max: $max.to_string(),
                        })
                        .and_then(|via| {
                            <$type>::try_from(via)
                                .map_err(|e| ParseError::FromStrError(e.to_string()))
                        })
                }
            }

            fn format(value: &$type) -> String {
                value.to_string()
            }
        }

        impl structform::ParseAndFormat<Option<$type>> for $numeric_input<Option<$type>> {
            fn parse(value: &str) -> Result<Option<$type>, structform::ParseError> {
                use std::convert::TryFrom;

                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Ok(None)
                } else {
                    trimmed
                        .parse::<$underlying_numeric_type>()
                        .map_err(|_e| structform::ParseError::NumberOutOfRange {
                            required_type: $type_name.to_string(),
                            min: $min.to_string(),
                            max: $max.to_string(),
                        })
                        .and_then(|via| {
                            <$type>::try_from(via)
                                .map_err(|e| structform::ParseError::FromStrError(e.to_string()))
                        })
                        .map(Option::Some)
                }
            }

            fn format(value: &Option<$type>) -> String {
                match value {
                    None => "".to_string(),
                    Some(inner) => inner.to_string(),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_numeric_input_with_default_with_stringops {
    ($numeric_input: ident, $type_name: literal, $type: ty, $underlying_numeric_type: ty) => {
        impl_numeric_input_with_default_with_stringops!(
            $numeric_input,
            $type_name,
            $type,
            $underlying_numeric_type,
            <$type>::MIN,
            <$type>::MAX
        );
    };
    ($numeric_input: ident, $type_name: literal, $type: ty, $underlying_numeric_type: ty, $min: expr, $max: expr) => {
        impl structform::ParseAndFormat<$type> for $numeric_input<$type> {
            fn parse(value: &str) -> Result<$type, ParseError> {
                use std::convert::TryFrom;
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Ok(<$type>::default())
                } else {
                    trimmed
                        .parse::<$underlying_numeric_type>()
                        .map_err(|_e| ParseError::NumberOutOfRange {
                            required_type: $type_name.to_string(),
                            min: $min.to_string(),
                            max: $max.to_string(),
                        })
                        .and_then(|via| {
                            <$type>::try_from(via)
                                .map_err(|e| ParseError::FromStrError(e.to_string()))
                        })
                }
            }

            fn format(value: &$type) -> String {
                value.to_string()
            }
        }
    };
}
