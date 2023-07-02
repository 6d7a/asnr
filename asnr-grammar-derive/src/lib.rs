extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, PathArguments, Type};

#[proc_macro_derive(Declare)]
pub fn declare_trait(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let struct_name = &input.ident;

    let declare_impl = match &input.data {
        syn::Data::Struct(s) => {
            // if struct contains of anonymous fields
            if s.fields
                .iter()
                .next()
                .map(|f| f.ident.as_ref())
                .flatten()
                .is_none()
            {
                todo!()
            } else {
                generate_struct_impl(s, struct_name)
            }
        }
        syn::Data::Enum(_e) => {
          // e.variants.first().unwrap().fields.iter().next().unwrap().
          todo!()
        },
        syn::Data::Union(_) => todo!(),
    };

    // Convert the generated implementation back into tokens
    let output = quote! {
        #declare_impl
    };

    // Return the generated tokens as a TokenStream
    output.into()
}

fn generate_struct_impl(
    s: &syn::DataStruct,
    struct_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let fields = wrap_fields(s);

    let formatting_string = format!("{} {{{{ {fields} }}}}", struct_name.to_string());
    let field_values = format_field_values(s);
    quote! {
      impl Declare for #struct_name {
          fn declare(&self) -> String {
              format!(#formatting_string, #(#field_values),*)
          }
      }
    }
}

fn format_field_values(s: &syn::DataStruct) -> Vec<proc_macro2::TokenStream> {
    s.fields.iter().map(|f| {
        let fident = f.ident.clone().unwrap();
        if let Type::Path(p) = &f.ty {
            let type_declarer = declare_type(&p);
            let first_vec = p.path
                .segments.iter().enumerate().find(|(_i, s)| s.ident.to_string().contains("Vec")).map(|(i, _s)| i);
            let first_option = p.path
              .segments.iter().enumerate().find(|(_i, s)| s.ident.to_string().contains("Option")).map(|(i, _s)| i);
            if first_vec.is_some() && first_option.is_none() {
                quote! { self.#fident.iter().map(|x| #type_declarer).collect::<Vec<String>>().join(", ") }
            } else if first_vec.is_none() && first_option.is_some()
            {
                quote! { self.#fident.as_ref().map_or(String::from("None"), |x| String::from("Some(") + &#type_declarer + ")") }
            } else if first_vec.is_some() && first_option.is_some()
            {
                quote! { self.#fident.as_ref().map_or(String::from("None"), |y| String::from("Some(vec![") + &y.iter().map(|x| #type_declarer).collect::<Vec<String>>().join(", ") + "])") }
            } else {
                quote! { self.#fident }
            }
        } else {
            quote! { self.#fident }
        }
    }).collect::<Vec<proc_macro2::TokenStream>>()
}

fn declare_type(ty: &syn::TypePath) -> proc_macro2::TokenStream {
    let innermost_type = if let PathArguments::AngleBracketed(ref params) =
        ty.path.segments.last().unwrap().arguments
    {
        if let Some(syn::GenericArgument::Type(Type::Path(p))) = params.args.first() {
            Some(p)
        } else {
            None
        }
    } else {
        None
    }
    .unwrap_or(ty);
    match innermost_type
        .path
        .segments
        .last()
        .unwrap()
        .ident
        .to_string()
        .as_str()
    {
        "bool" | "usize" | "i128" | "u128" | "u8" => quote! { x.to_string() },
        "String" => quote! { format!("\"{}\"", x) },
        _ => quote! { x.declare() },
    }
}

fn wrap_fields(s: &syn::DataStruct) -> String {
    s.fields
        .iter()
        .map(|f| {
            let t = if let Type::Path(p) = &f.ty {
                let first_vec = p
                    .path
                    .segments
                    .iter()
                    .enumerate()
                    .find(|(_i, s)| s.ident.to_string().contains("Vec"))
                    .map(|(i, _s)| i);
                let first_string = p
                    .path
                    .segments
                    .iter()
                    .enumerate()
                    .find(|(_i, s)| s.ident.to_string().contains("String"))
                    .map(|(i, _s)| i);
                if first_vec.is_some() && first_string.is_none() {
                    "vec![{}]"
                } else if first_vec.is_none() && first_string.is_some() {
                    r#"String::from("{}")"#
                } else {
                    "{}"
                }
            } else {
                "{}"
            };
            format!("{}: {}", f.ident.as_ref().unwrap().to_string(), t)
        })
        .collect::<Vec<String>>()
        .join(", ")
}
