use syn::ImplItem;
use syn::Item::Impl;
use syn::ItemImpl;

use syn::Item::Trait;
use syn::ItemTrait;
use syn::TraitItem;

use syn::visit;
use syn::visit::Visit;

use std::env;
use std::fs;
use std::process;

struct StmtVisitor {
    found_unsafe: bool,
}

impl<'ast> Visit<'ast> for StmtVisitor {
    fn visit_expr_unsafe(&mut self, i: &'ast syn::ExprUnsafe) {
        self.found_unsafe = true;
        visit::visit_expr_unsafe(self, i);
    }
}

fn print_pub_unsafe_and_unsafe_containing_fns(ii: ItemImpl) {
    let mut interesting = false;
    let mut pub_unsafe_fns = Vec::new();
    let mut unsafe_containing_fns = Vec::new();
    for item in &ii.items {
        match item {
            ImplItem::Fn(f) =>
            {
		// record all pub unsafe functions
                if matches!(f.vis, syn::Visibility::Public(_)) && matches!(f.sig.unsafety, Some(_))
                {
                    interesting = true;
                    pub_unsafe_fns.push(format!("--- pub unsafe fn {}", f.sig.ident));
                }
                // record functions that contain unsafe code in their bodies but that are not marked unsafe
                else if matches!(f.sig.unsafety, None) {
                    let mut sv = StmtVisitor {
                        found_unsafe: false,
                    };
                    sv.visit_block(&f.block);
                    if sv.found_unsafe {
                        interesting = true;
                        unsafe_containing_fns
                            .push(format!("--- unsafe-containing fn {}", f.sig.ident));
                    }
                }
            }
            _ => (),
        }
    }
    if interesting {
	// create an empty impl with the same name as ii
	let mut i_copy = ii.clone();
	i_copy.items = Vec::new();
        let file = syn::File {
            attrs: vec![],
            items: vec![Impl(i_copy)],
            shebang: None,
        };
        print!("{}", prettyplease::unparse(&file));
        pub_unsafe_fns.iter().for_each(|s| {
            println!("{}", s);
        });
        unsafe_containing_fns.iter().for_each(|s| {
            println!("{}", s);
        });
        println!();
    } else {
        // println!("--- nothing interesting here");
    }
}

fn print_trait_unsafe_containing_fns(it: ItemTrait) {
    let mut interesting = false;
    let mut unsafe_containing_fns = Vec::new();
    for item in &it.items {
        match item {
            TraitItem::Fn(f) =>
            // record functions that contain unsafe code in their bodies but that are not marked unsafe
            {
                if matches!(f.sig.unsafety, None) {
                    let mut sv = StmtVisitor {
                        found_unsafe: false,
                    };
                    if let Some(d) = &f.default {
                        sv.visit_block(&d);
                    }
                    if sv.found_unsafe {
                        interesting = true;
                        unsafe_containing_fns
                            .push(format!("--- unsafe-containing fn {}", f.sig.ident));
                    }
                }
            }
            _ => (),
        }
    }
    if interesting {
	let mut i_copy = it.clone();
	i_copy.items = Vec::new();
        let file = syn::File {
            attrs: vec![],
            items: vec![Trait(i_copy)],
            shebang: None,
        };
        print!("{}", prettyplease::unparse(&file));
        unsafe_containing_fns.iter().for_each(|s| {
            println!("{}", s);
        });
        println!();
    } else {
        // println!("--- nothing interesting here");
    }
}

fn main() {
    let mut args = env::args();
    let _ = args.next(); // executable name

    let filename = match (args.next(), args.next()) {
        (Some(filename), None) => filename,
        _ => {
            eprintln!("Usage: unsafe-finder path/to/filename.rs");
            process::exit(1);
        }
    };

    let src = fs::read_to_string(&filename).expect("unable to read file");
    let syntax = syn::parse_file(&src).expect("unable to parse file");

    for item in syntax.items {
        match item {
            Impl(im) => print_pub_unsafe_and_unsafe_containing_fns(im),
            Trait(t) => print_trait_unsafe_containing_fns(t),
            _ => (),
        }
    }
    // Debug impl is available if Syn is built with "extra-traits" feature.
    // println!("{:#?}", syntax);
}
