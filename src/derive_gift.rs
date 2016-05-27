#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]

#[macro_use]
extern crate rustc;
extern crate rustc_plugin;
extern crate syntax;

use syntax::codemap::Span;
use rustc_plugin::Registry;

//////////////////////////////////////////////////////////////////////
// Derive custom implementations where all Ref<T> types are replaced by ImpRef or FunRef

use syntax::ast;
use syntax::ast::{Item, ItemKind, StructField, Ident, VariantData, MetaItem, Generics};
use syntax::ext::base::{Annotatable, MultiItemDecorator, SyntaxExtension, ExtCtxt};
use syntax::ext::build::AstBuilder;
use syntax::parse::token;
use syntax::ptr::P;

struct DeriveGiftStructs;

use syntax::print::pprust::{item_to_string, path_to_string};

fn generate_gift_trait(cx: &ExtCtxt, ann: &Annotatable, span: Span) -> Annotatable {
    let mut ret = ann.clone();
    if let Annotatable::Item(ref item) = *ann {
        if let ast::ItemKind::Struct(ref struct_def, ref generics) = item.node {
            if let ast::VariantData::Struct(_, ref _node_id) = *struct_def {
                //                    let fun_ty : Option<P<ast::Ty>> = None;
                //                    let imp_ty : Option<P<ast::Ty>> = None;
                let new_trait_data : Vec<ast::TraitItem> = vec![
                    //                        ast::TraitItem{
                    //                            id:    ast::DUMMY_NODE_ID,
                    //                            ident: Ident::with_empty_ctxt(token::intern("Fun")),
                    //                            attrs: Vec::new(),
                    //                            span:  item.span,
                    //                            node:  ast::TraitItemKind::Type(P::from_vec(Vec::new()), fun_ty)
                    //                        },
                    //                        ast::TraitItem{
                    //                            id:    ast::DUMMY_NODE_ID,
                    //                            ident: Ident::with_empty_ctxt(token::intern("Imp")),
                    //                            attrs: Vec::new(),
                    //                            span:  item.span,
                    //                            node:  ast::TraitItemKind::Type(P::from_vec(Vec::new()), imp_ty)
                    //                    }
                ];
                //ast::VariantData::Struct(Vec::new(), node_id.clone());

                let new_item_kind : ast::ItemKind = ast::ItemKind::Trait(ast::Unsafety::Normal,
                                                                         generics.clone(),
                                                                         P::from_vec(Vec::new()), // TyParamBounds
                                                                         new_trait_data);
                let mut new_item : ast::Item = (**item).clone();
                let prefix : String = "Gift".to_string();
                let strct_name = format!("{}", new_item.ident);
                new_item.ident = Ident::with_empty_ctxt(token::intern(&(prefix+&strct_name)));
                new_item.node = new_item_kind;

                println!("{}", item_to_string(&new_item));

                let new_annotatable : Annotatable = Annotatable::Item(P(new_item));
                ret = new_annotatable;
            } else {
                cx.span_err(item.span, "can only derive(gift) on structs")
            }
        } else {
            cx.span_err(item.span, "can only derive(gift) on structs")
        }
    } else {
        cx.span_err(span, "can only derive(gift) on structs")
    }
    ret
}

fn generate_version(cx: &ExtCtxt, strct: &Item, version_name: &str) -> Annotatable {

    let prefix : String = "Gift".to_string();
    let prefix = prefix+version_name;
    let strct_name = format!("{}", strct.ident);
    let ident = Ident::with_empty_ctxt(token::intern(&(prefix+&strct_name)));

    let struct_def = get_struct_variant_data(strct).clone();
    let generics = get_struct_generics(strct).clone();

    let mut fields = Vec::new();
    fields.extend_from_slice(struct_def.fields());
    for ref mut f in &mut fields {
        let mut ref_name : String = String::new();
        ref_name = ref_name+version_name+"Ref";
        wrap_field_types_in_ref(cx, f, ref_name.as_str());
    }
    let mut x : ast::Item = strct.clone();
    x.ident = ident;
    x.node = ItemKind::Struct(VariantData::Struct(fields, ast::DUMMY_NODE_ID), generics);
    println!("{}", item_to_string(&x));
    Annotatable::Item(P(x))
}

fn is_gift_derived(iden: &Ident) -> bool {
    let nam = format!("{}", iden);
    nam.starts_with("Gift")
}

fn wrap_field_types_in_ref(cx: &ExtCtxt, field: &mut StructField, ref_name: &str) {
    //let mut ret : StructField = field.clone();

    if let ast::TyKind::Path(_, ref path) = field.ty.node.clone() {
        if path_to_string(path).starts_with("Ref<") {
            let mut new_path : ast::Path = path.clone();
            new_path.segments[0].identifier = Ident::with_empty_ctxt(token::intern(ref_name));
            field.ty = cx.ty_path(new_path);
        }
    }
}

fn get_struct_variant_data(itm: &Item) -> &VariantData {
    match itm.node {
        ItemKind::Struct(ref variant_data, _) => {
            match variant_data.clone() {
                VariantData::Struct(_, _) => variant_data,
                _ => panic!("not a struct")
            }
        }
        _ => panic!("not a struct")
    }
}

fn get_struct_generics(itm: &Item) -> &Generics {
    match itm.node {
        ItemKind::Struct(_, ref generics) => {
            generics
        }
        _ => panic!("not a struct")
    }
}

impl MultiItemDecorator for DeriveGiftStructs {
    fn expand(&self,
              cx: &mut ExtCtxt,
              _span: Span,
              _mitem: &MetaItem,
              ann: &Annotatable,
              push: &mut FnMut(Annotatable)) {
        match *ann {
            Annotatable::Item(ref p) => {
                let inner_item : &Item = p;
                match inner_item.node {
                    ItemKind::Struct(ref _variant_data, ref _generics) => {
                        if ! is_gift_derived(&inner_item.ident) {
                            push(generate_version(cx, inner_item, "Imp"));
                            push(generate_version(cx, inner_item, "Fun"));
                            push(generate_gift_trait(cx, ann, inner_item.span));
                        }
                    }
                    _ => /* ignore */ ()

                }
            }
            Annotatable::TraitItem(_) => /* ignore */ (),
            Annotatable::ImplItem(_) => /* ignore */ (),
        }
    }
}

//
//////////////////////////////////////////////////////////////////////

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_syntax_extension(token::intern("derive_gift"),
                                  SyntaxExtension::MultiDecorator(Box::new(DeriveGiftStructs)));
}
