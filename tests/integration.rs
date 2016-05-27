#![feature(plugin)]
#![feature(custom_derive)]
#![plugin(derive_gift)]
extern crate giftr;

use giftr::refs::GiftRef;
use giftr::refs::dummy::Ref as Ref;
use giftr::refs::imperative::Ref as ImpRef;
use giftr::refs::functional::Ref as FunRef;

// ===================================================================

#[derive(gift)]
mod Ctr {
    struct Counter {
        x : Ref<i32>,
    }
}

#[test]
fn foo() {
    let _a = GiftImpCounter { x: ImpRef::new(0) };
    let _b = GiftFunCounter { x: FunRef::new(0) };
}
