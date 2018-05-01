extern crate proc_macro;
extern crate proc_macro2;
extern crate syn;

#[macro_use] extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::*;

struct TaggedField {
    field: Ident,
    ty: Type,
    method: Ident,
    vis: Visibility,
}

#[proc_macro_derive(PinAccessor, attributes(pin_accessor))]
pub fn pin_accessor(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();

    let name = ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    let fields = match ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
        _ => panic!("PinAccessor can only be derived for structs with named fields."),
    };

    // Get info about any fields tagged to create accessors
    let tagged_fields = fields.iter().filter_map(|field| {
        field.attrs.iter().find(is_pin_accessor_attr).map(|attr| {
            TaggedField {
                field: field.ident.clone().unwrap(),
                ty: field.ty.clone(),
                method: get_method_name(attr).unwrap_or_else(|| {
                    let name = format!("{}_pinned", field.ident.clone().unwrap());
                    Ident::new(&name, Span::call_site())
                }),
                vis: get_visibility(attr).unwrap_or_else(|| field.vis.clone()),
            }
        })
    });

    let accessors = tagged_fields.into_iter().map(|tagged_field| {
        let TaggedField { field, ty, method, vis } = tagged_field;
        quote! {
           #vis fn #method<'a>(self: &'a mut ::std::mem::Pin<Self>) -> ::std::mem::Pin<'a, #ty> {
               unsafe {
                   ::std::mem::Pin::map(self, |this| &mut this.#field)
               }
           }
        }
    });

    (quote! {
        impl #impl_generics #name #type_generics #where_clause {
            #(#accessors)*
        }
    }).into()
}

fn is_pin_accessor_attr(attr: &&Attribute) -> bool {
    attr.path.segments.last().map_or(false, |segment| {
        segment.value().ident == "pin_accessor"
    })
}

fn get_visibility(attr: &Attribute) -> Option<Visibility> {
    let lit = find_attribute_member(attr, "vis")?;
    if let Lit::Str(s) = lit {
        Some(s.parse().unwrap_or_else(|e| {
            panic!("Could not parse `vis` argument as a visibility: {}", e)
        }))
    } else { panic!("Invalid literal for `vis` argument to `pin_accessor` attribute.") }

}

fn get_method_name(attr: &Attribute) -> Option<Ident> {
    let lit = find_attribute_member(attr, "name")?;
    if let Lit::Str(s) = lit {
        Some(s.parse().unwrap_or_else(|e| {
            panic!("Could not parse `name` argument as a method name: {}", e)
        }))
    } else { panic!("Invalid literal for `name` argument to `pin_accessor` attribute.") }
}

// Given an attribute:
//      #[foo(bar = "baz", ..)]
//
// find_attribute_member(attr, "bar") will return the "baz" literal.
// (Essentially look up the value from the key in the arguments to the attr.)
fn find_attribute_member(attr: &Attribute, name: &str) -> Option<Lit> {
    if let Some(Meta::List(MetaList { nested, .. })) = attr.interpret_meta() {
        let nested = nested.iter().find(|nested| match nested {
            NestedMeta::Meta(meta)  => meta.name() == name,
            _                       => false,
        })?;

        match nested {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit, .. }))    => Some(lit.clone()),
            _                                                               => None,
        }
    } else { None }
}
