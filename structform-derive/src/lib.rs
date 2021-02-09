use proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(StructForm, attributes(structform))]
pub fn derive_structform(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let form_ident = input.ident.clone();
    let field_enum_ident = field_enum_ident_transform(&form_ident);

    let input_struct_data = match input.data {
        Data::Struct(data) => data,
        _ => panic!("StructForm can only be derived for structs"),
    };
    let container_attrs: FormContainerAttribute = input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("structform"))
        .map(|attr| {
            attr.parse_args()
                .expect("Failed to parse the #[structform] attr on the container")
        })
        .expect("Require a #[structform] attribute on the container");
    let model = container_attrs.model;

    let enriched_fields = enrich_fields(&input_struct_data);

    let (input_names, input_fields_type): (Vec<(Ident, Ident)>, Vec<Type>) = enriched_fields
        .iter()
        .filter_map(|field| match &field.ty {
            FieldType::Input { input_type } => Some((field.names(), input_type.clone())),
            _ => None,
        })
        .unzip();
    let (input_fields_snake_case, input_fields_pascal_case): (Vec<Ident>, Vec<Ident>) =
        input_names.into_iter().unzip();

    let (option_form_names, option_form_fields_type): (Vec<(Ident, Ident)>, Vec<Type>) =
        enriched_fields
            .iter()
            .filter_map(|field| match &field.ty {
                FieldType::OptionalSubform { subform_type } => {
                    Some((field.names(), subform_type.clone()))
                }
                _ => None,
            })
            .unzip();
    let (option_form_fields_snake_case, option_form_fields_pascal_case): (Vec<Ident>, Vec<Ident>) =
        option_form_names.into_iter().unzip();
    let option_form_fields_type_field_enum: Vec<Ident> = option_form_fields_type
        .iter()
        .map(type_to_field_enum_ident)
        .collect();

    let option_form_fields_toggles_pascal_case: Vec<Ident> = option_form_fields_pascal_case
        .iter()
        .map(|field_ident| Ident::new(&format!("Toggle{}", field_ident), field_ident.span()))
        .collect();

    let (list_form_names, list_form_fields_type): (Vec<(Ident, Ident)>, Vec<Type>) =
        enriched_fields
            .iter()
            .filter_map(|field| match &field.ty {
                FieldType::ListSubform { subform_type } => {
                    Some((field.names(), subform_type.clone()))
                }
                _ => None,
            })
            .unzip();
    let (list_form_fields_snake_case, list_form_fields_pascal_case): (Vec<Ident>, Vec<Ident>) =
        list_form_names.into_iter().unzip();
    let list_form_fields_type_field_enum: Vec<Ident> = list_form_fields_type
        .iter()
        .map(type_to_field_enum_ident)
        .collect();

    let list_form_fields_add_pascal_case: Vec<Ident> = list_form_fields_pascal_case
        .iter()
        .map(|field_ident| Ident::new(&format!("Add{}", field_ident), field_ident.span()))
        .collect();
    let list_form_fields_remove_pascal_case: Vec<Ident> = list_form_fields_pascal_case
        .iter()
        .map(|field_ident| Ident::new(&format!("Remove{}", field_ident), field_ident.span()))
        .collect();

    let (subform_names, subform_fields_type): (Vec<(Ident, Ident)>, Vec<Type>) = enriched_fields
        .iter()
        .filter_map(|field| match &field.ty {
            FieldType::Subform { subform_type } => Some((field.names(), subform_type.clone())),
            _ => None,
        })
        .unzip();
    let (subform_fields_snake_case, subform_fields_pascal_case): (Vec<Ident>, Vec<Ident>) =
        subform_names.into_iter().unzip();
    let subform_fields_type_field_enum: Vec<Ident> = subform_fields_type
        .iter()
        .map(type_to_field_enum_ident)
        .collect();

    let submit_attempted_fields_snake_case: Vec<Ident> = enriched_fields
        .iter()
        .filter_map(|field| match &field.ty {
            FieldType::SubmitAttempted => Some(field.snake_case_ident.clone()),
            _ => None,
        })
        .collect();

    let field_enum = quote! {
        #[derive(Debug)]
        pub enum #field_enum_ident {
            #(#input_fields_pascal_case,)*
            #(#option_form_fields_toggles_pascal_case,)*
            #(#option_form_fields_pascal_case(#option_form_fields_type_field_enum),)*
            #(#list_form_fields_add_pascal_case,)*
            #(#list_form_fields_pascal_case(usize, #list_form_fields_type_field_enum),)*
            #(#list_form_fields_remove_pascal_case(usize),)*
            #(#subform_fields_pascal_case(#subform_fields_type_field_enum),)*
        }
    };

    let impl_new = if container_attrs.flatten {
        quote! {
            fn new(model: &#model) -> #form_ident {
                #form_ident {
                    #(#input_fields_snake_case: <#input_fields_type>::new(&model),)*
                    #(#submit_attempted_fields_snake_case: false,)*
                }
            }
        }
    } else {
        quote! {
            fn new(model: &#model) -> #form_ident {
                #form_ident {
                    #(#input_fields_snake_case: <#input_fields_type>::new(&model.#input_fields_snake_case),)*
                    #(#option_form_fields_snake_case: model.#option_form_fields_snake_case.as_ref().map(<#option_form_fields_type>::new),)*
                    #(#list_form_fields_snake_case: model.#list_form_fields_snake_case.iter().map(<#list_form_fields_type>::new).collect(),)*
                    #(#subform_fields_snake_case: <#subform_fields_type>::new(&model.#subform_fields_snake_case),)*
                    #(#submit_attempted_fields_snake_case: false,)*
                }
            }
        }
    };

    let impl_submit = container_attrs
        .submit_with
        .map(|submit_with| {
            quote! {
                fn submit(&mut self) -> Result<#model, structform::ParseError> {
                    #(self.#submit_attempted_fields_snake_case = true;)*
                    #submit_with(self)
                }
            }
        })
        .unwrap_or(if container_attrs.flatten {
            quote! {
                fn submit(&mut self) -> Result<#model, structform::ParseError> {
                    #(self.#submit_attempted_fields_snake_case = true;)*
                    #(self.#input_fields_snake_case.submit())*
                }
            }
        } else {
            quote! {
                fn submit(&mut self) -> Result<#model, structform::ParseError> {
                    #(self.#submit_attempted_fields_snake_case = true;)*
                    self.submit_update(<#model>::default())
                }
            }
        });

    let impl_submit_update = if container_attrs.flatten {
        quote! {
            fn submit_update(&mut self, mut model: #model) -> Result<#model, structform::ParseError> {
                #(self.#submit_attempted_fields_snake_case = true;)*
                #(self.#input_fields_snake_case.submit())*
            }
        }
    } else {
        quote! {
            fn submit_update(&mut self, mut model: #model) -> Result<#model, structform::ParseError> {
                #(self.#submit_attempted_fields_snake_case = true;)*

                #(let #input_fields_snake_case = self.#input_fields_snake_case.submit();)*
                #(let #option_form_fields_snake_case = self.#option_form_fields_snake_case.as_mut().map(|inner_form| {
                    model.#option_form_fields_snake_case
                        .clone()
                        .map(|inner_model| inner_form.submit_update(inner_model))
                        .unwrap_or_else(|| inner_form.submit())
                }).transpose();)*
                #(let #list_form_fields_snake_case = self.#list_form_fields_snake_case.iter_mut().enumerate().map(|(i, inner_form)| {
                    model.#list_form_fields_snake_case
                        .get(i)
                        .map(|inner_model| inner_form.submit_update(inner_model.clone()))
                        .unwrap_or_else(|| inner_form.submit())
                }).collect::<Result<Vec<_>,_>>();)*
                #(let #subform_fields_snake_case = self.#subform_fields_snake_case.submit_update(model.#subform_fields_snake_case.clone());)*

                #(model.#input_fields_snake_case = #input_fields_snake_case?;)*
                #(model.#option_form_fields_snake_case = #option_form_fields_snake_case?;)*
                #(model.#list_form_fields_snake_case = #list_form_fields_snake_case?;)*
                #(model.#subform_fields_snake_case = #subform_fields_snake_case?;)*
                Ok(model)
            }
        }
    };

    let impl_set_input = quote! {
        fn set_input(&mut self, field: #field_enum_ident, value: String) {
            match field {
                #(#field_enum_ident::#input_fields_pascal_case => self.#input_fields_snake_case.set_input(value),)*
                #(#field_enum_ident::#option_form_fields_toggles_pascal_case => {
                    if self.#option_form_fields_snake_case.is_some() {
                        self.#option_form_fields_snake_case = None;
                    } else {
                        self.#option_form_fields_snake_case = Some(#option_form_fields_type::default());
                    }
                },)*
                #(#field_enum_ident::#option_form_fields_pascal_case(subfield) => {
                    self.#option_form_fields_snake_case
                        .as_mut()
                        .map(|inner_form| inner_form.set_input(subfield, value));
                },)*
                #(#field_enum_ident::#list_form_fields_add_pascal_case => {
                    self.#list_form_fields_snake_case
                        .push(#list_form_fields_type::default());
                },)*
                #(#field_enum_ident::#list_form_fields_pascal_case(i, subfield) => {
                    self.#list_form_fields_snake_case
                        .get_mut(i)
                        .map(|inner_form| inner_form.set_input(subfield, value));
                },)*
                #(#field_enum_ident::#list_form_fields_remove_pascal_case(i) => {
                    if i < self.#list_form_fields_snake_case.len() {
                        self.#list_form_fields_snake_case.remove(i);
                    }
                },)*

                #(#field_enum_ident::#subform_fields_pascal_case(subfield) => {
                    self.#subform_fields_snake_case.set_input(subfield, value);
                },)*
            }
        }
    };

    let impl_submit_attempted = quote! {
        fn submit_attempted(&self) -> bool {
            false #(|| self.#submit_attempted_fields_snake_case)*
        }
    };

    let impl_is_empty = quote! {
        fn is_empty(&self) -> bool {
            true
            #(&& self.#input_fields_snake_case.is_empty())*
            #(&& self.#option_form_fields_snake_case.as_ref().map(|inner_form| inner_form.is_empty()).unwrap_or(true))*
            #(&& self.#list_form_fields_snake_case.iter().all(|inner_form| inner_form.is_empty()))*
            #(&& self.#subform_fields_snake_case.is_empty())*
        }
    };

    let impl_form = quote! {
        impl structform::StructForm<#model> for #form_ident {
            type Field = #field_enum_ident;

            #impl_new
            #impl_submit
            #impl_submit_update
            #impl_set_input
            #impl_submit_attempted
            #impl_is_empty
        }
    };

    (quote! {
        #field_enum

        #impl_form
    })
    .into()
}

fn snake_to_pascal_case(snake: &str) -> String {
    snake
        .split('_')
        .map(|s| {
            let (head, tail) = s.split_at(1);
            format!("{}{}", head.to_uppercase(), tail)
        })
        .collect::<Vec<_>>()
        .join("")
}

fn is_option(field: &Field) -> bool {
    if let Type::Path(TypePath { path, .. }) = &field.ty {
        let path_ident = &path.segments.first().unwrap().ident;
        path_ident == &Ident::new("Option", path_ident.span())
    } else {
        false
    }
}

fn is_vec(field: &Field) -> bool {
    if let Type::Path(TypePath { path, .. }) = &field.ty {
        let path_ident = &path.segments.first().unwrap().ident;
        path_ident == &Ident::new("Vec", path_ident.span())
    } else {
        false
    }
}

fn parse_option_type_generic_type(option_type: &Type) -> Type {
    match option_type {
        Type::Path(TypePath { path, .. }) => match &path.segments.first().unwrap().arguments {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                match args.first().unwrap() {
                    GenericArgument::Type(generic_type) => generic_type.clone(),
                    _ => panic!("Option's type argument was not a generic type"),
                }
            }
            _ => panic!("Option type did not have an angle bracketed generic argument"),
        },
        _ => panic!("Option type did not have a generic argument"),
    }
}

fn parse_vec_type_generic_type(vec_type: &Type) -> Type {
    match vec_type {
        Type::Path(TypePath { path, .. }) => match &path.segments.first().unwrap().arguments {
            PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => {
                match args.first().unwrap() {
                    GenericArgument::Type(generic_type) => generic_type.clone(),
                    _ => panic!("Vec's type argument was not a generic type"),
                }
            }
            _ => panic!("Vec type did not have an angle bracketed generic argument"),
        },
        _ => panic!("Vec type did not have a generic argument"),
    }
}

fn type_to_field_enum_ident(ty: &Type) -> Ident {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            field_enum_ident_transform(&path.segments.first().unwrap().ident)
        }
        _ => panic!("Option's generic type was not a TypePath"),
    }
}

fn field_enum_ident_transform(ident: &Ident) -> Ident {
    Ident::new(&format!("{}Field", ident), ident.span())
}

struct FormContainerAttribute {
    model: Ident,
    submit_with: Option<Ident>,
    flatten: bool,
}

impl parse::Parse for FormContainerAttribute {
    fn parse(parse_buffer: &syn::parse::ParseBuffer<'_>) -> parse::Result<Self> {
        let meta_list = parse_buffer.parse_terminated::<_, syn::token::Comma>(NestedMeta::parse)?;
        let model: String = meta_list
            .iter()
            .filter_map(|arg| match arg {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. }))
                    if path.is_ident("model") =>
                {
                    match lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => None,
                    }
                }
                _ => None,
            })
            .next()
            .expect(
                "Expected to find an attribute indicating the model type: #[structform(model = \"???\")]",
            );
        let model = Ident::new(&model, parse_buffer.span());
        let submit_with: Option<String> = meta_list
            .iter()
            .filter_map(|arg| match arg {
                NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. }))
                    if path.is_ident("submit_with") =>
                {
                    match lit {
                        Lit::Str(lit) => Some(lit.value()),
                        _ => None,
                    }
                }
                _ => None,
            })
            .next();
        let submit_with =
            submit_with.map(|submit_with| Ident::new(&submit_with, parse_buffer.span()));
        let flatten = meta_list.iter().any(
            |arg| matches!(arg, NestedMeta::Meta(Meta::Path(path)) if path.is_ident("flatten")),
        );

        Ok(FormContainerAttribute {
            model,
            submit_with,
            flatten,
        })
    }
}

#[derive(Default)]
struct FormFieldAttribute {
    submit_attempted: bool,
    subform: bool,
}

impl parse::Parse for FormFieldAttribute {
    fn parse(parse_buffer: &syn::parse::ParseBuffer<'_>) -> parse::Result<Self> {
        let meta_list = parse_buffer.parse_terminated::<_, syn::token::Comma>(NestedMeta::parse)?;
        let submit_attempted = meta_list.iter().any(|arg| matches!(arg, NestedMeta::Meta(Meta::Path(path)) if path.is_ident("submit_attempted")));
        let subform = meta_list.iter().any(
            |arg| matches!(arg, NestedMeta::Meta(Meta::Path(path)) if path.is_ident("subform")),
        );

        Ok(FormFieldAttribute {
            submit_attempted,
            subform,
        })
    }
}

struct RichField {
    snake_case_ident: Ident,
    pascal_case_ident: Ident,
    ty: FieldType,
}

impl RichField {
    fn names(&self) -> (Ident, Ident) {
        (
            self.snake_case_ident.clone(),
            self.pascal_case_ident.clone(),
        )
    }
}

fn enrich_fields(struct_data: &DataStruct) -> Vec<RichField> {
    struct_data
        .fields
        .iter()
        .map(|field| {
            let snake_case_ident = field
                .ident
                .clone()
                .expect("Only normal structs are supported.");
            let pascal_case_ident = Ident::new(
                &snake_to_pascal_case(&snake_case_ident.to_string()),
                snake_case_ident.span(),
            );
            let attrs = field
                .attrs
                .iter()
                .filter(|attr| attr.path.is_ident("structform"))
                .map(|attr| {
                    attr.parse_args::<FormFieldAttribute>()
                        .expect("failed to parse attrs on a field")
                })
                .next()
                .unwrap_or_default();

            let ty = if attrs.submit_attempted {
                FieldType::SubmitAttempted
            } else if attrs.subform {
                FieldType::Subform {
                    subform_type: field.ty.clone(),
                }
            } else if is_option(field) {
                FieldType::OptionalSubform {
                    subform_type: parse_option_type_generic_type(&field.ty),
                }
            } else if is_vec(field) {
                FieldType::ListSubform {
                    subform_type: parse_vec_type_generic_type(&field.ty),
                }
            } else {
                FieldType::Input {
                    input_type: field.ty.clone(),
                }
            };

            RichField {
                snake_case_ident,
                pascal_case_ident,
                ty,
            }
        })
        .collect()
}

enum FieldType {
    Input { input_type: Type },
    Subform { subform_type: Type },
    OptionalSubform { subform_type: Type },
    ListSubform { subform_type: Type },
    SubmitAttempted,
}
