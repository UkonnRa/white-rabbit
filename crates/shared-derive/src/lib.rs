use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Ident, Type};

#[proc_macro_derive(DomainModel)]
pub fn domain_model(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);
    match expand_domain_model(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_domain_model(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(syn::Error::new_spanned(
                    name,
                    "DomainModel can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                name,
                "DomainModel can only be derived for structs",
            ));
        }
    };

    let id_ty = format_ident!("{}Id", name);

    let _id_field = field_type(&fields, "id")?;
    let _version_field = field_type(&fields, "version")?;
    let _created_at_field = field_type(&fields, "created_at")?;
    let _last_modified_at_field = field_type(&fields, "last_modified_at")?;

    let tokens = quote! {
        #[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct #id_ty(String);

        impl ::shared::EntityId for #id_ty {
            type Entity = #name;

            const TYPE_NAME: &'static str = stringify!(#name);

            fn value(&self) -> &str {
                &self.0
            }

            fn from_value(value: impl Into<String>) -> Self {
                Self(value.into())
            }
        }

        impl ::std::fmt::Display for #id_ty {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl Default for #id_ty {
            fn default() -> Self {
                Self(::uuid::Uuid::now_v7().to_string())
            }
        }

        impl From<String> for #id_ty {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for #id_ty {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }

        impl AsRef<str> for #id_ty {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ::shared::Entity for #name {
            type Id = #id_ty;

            fn id(&self) -> &Self::Id {
                &self.id
            }

            fn version(&self) -> usize {
                self.version
            }

            fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                self.created_at
            }

            fn last_modified_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
                self.last_modified_at
            }
        }

    };

    Ok(tokens)
}

fn field_type(
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
    name: &str,
) -> syn::Result<Type> {
    fields
        .iter()
        .find(|field| {
            field
                .ident
                .as_ref()
                .map(|ident| ident == name)
                .unwrap_or(false)
        })
        .map(|field| field.ty.clone())
        .ok_or_else(|| {
            syn::Error::new_spanned(
                Ident::new(name, proc_macro2::Span::call_site()),
                format!("Field '{}' is required for DomainModel", name),
            )
        })
}
