//! `obfuscate_macro`
//!
//! An attribute proc-macro that walks the AST of a function body and splices
//! in randomly-generated, behavior-preserving "junk": dead arithmetic, dead
//! if/else, and dead match blocks. It's source-level noise, not a security
//! boundary -- see the caveats in the crate that calls this, and the note
//! at the bottom of this file on what this class of technique can't do.
//!
//! ```ignore
//! #[obfuscate]                          // density 0.35, black_box only
//! fn foo() { ... }
//!
//! #[obfuscate(0.6)]                     // custom density (0.0..=1.0)
//! fn bar() { ... }
//!
//! #[obfuscate(0.6, unsafe_ok)]          // also rotate in a volatile-based
//! fn baz() { ... }                      // opacifier for codegen variety
//!
//! #[obfuscate(0.6, unsafe_ok, allow_inline)]  // opt out of auto no-inline
//! fn hot_path() { ... }
//! ```
//!
//! ## Reducing the codegen fingerprint
//!
//! Every seed and every "sink" used to previously go through exactly one
//! call shape: `std::hint::black_box`. That's correct (the compiler really
//! can't fold through it) but it means every single junk site in the whole
//! binary produces the *identical* instruction sequence -- which is itself
//! a trivially greppable/diffable signature for anyone comparing binaries
//! across your releases or writing a one-time script to strip it.
//!
//! When `unsafe_ok` is passed, each seed/sink independently and randomly
//! picks between two mechanisms with genuinely different codegen:
//!   - `std::hint::black_box` (always available, always sound)
//!   - a manual volatile round-trip through a stack local (sound, but
//!     requires an `unsafe` block, so it's opt-in -- it will break crates
//!     that `#![forbid(unsafe_code)]`)
//!
//! Six junk shapes (up from four) further dilutes any single fingerprint,
//! and variable names are drawn from a small pool of ordinary-looking
//! prefixes instead of an obvious `_junk_` tag -- mainly relevant if this
//! ever gets inspected with debug info attached, since names don't survive
//! into a stripped release binary at all either way.

use proc_macro::TokenStream;
use quote::quote;
use rand::Rng;
use syn::parse::{Parse, ParseStream};
use syn::visit_mut::{self, VisitMut};
use syn::{
    Arm, Block, Expr, ExprClosure, Ident, ItemFn, LitFloat, Stmt, Token, parse_macro_input,
    parse_quote,
};

/// Parses the attribute args: an optional density float, plus any of
/// `allow_inline` / `unsafe_ok`, comma-separated, in any order.
struct ObfuscateArgs {
    density: f64,
    allow_inline: bool,
    unsafe_ok: bool,
}

impl Parse for ObfuscateArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut density = 0.35_f64;
        let mut density_set = false;
        let mut allow_inline = false;
        let mut unsafe_ok = false;

        let mut first = true;
        while !input.is_empty() {
            if !first {
                input.parse::<Token![,]>()?;
            }
            first = false;

            if input.peek(LitFloat) {
                if density_set {
                    return Err(input.error("density specified twice"));
                }
                density = input
                    .parse::<LitFloat>()?
                    .base10_parse::<f64>()?
                    .clamp(0.0, 1.0);
                density_set = true;
            } else {
                let ident: Ident = input.parse()?;
                if ident == "allow_inline" {
                    allow_inline = true;
                } else if ident == "unsafe_ok" {
                    unsafe_ok = true;
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "unknown `obfuscate` option, expected a density float, \
                         `allow_inline`, or `unsafe_ok`",
                    ));
                }
            }
        }

        Ok(ObfuscateArgs {
            density,
            allow_inline,
            unsafe_ok,
        })
    }
}

#[proc_macro_attribute]
pub fn obfuscate(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as ObfuscateArgs);
    let mut input_fn = parse_macro_input!(item as ItemFn);

    let mut injector = JunkInjector {
        rng: rand::thread_rng(),
        density: args.density,
        unsafe_ok: args.unsafe_ok,
    };
    injector.visit_block_mut(&mut input_fn.block);

    // Auto no-inline: stops the optimizer from inlining this function into
    // a caller and re-analyzing the junk in a wider window under LTO. This
    // is the surgical version of "disable optimizations" -- costs nothing
    // elsewhere in the crate, unlike a blanket opt-level drop.
    let already_has_inline_attr = input_fn.attrs.iter().any(|a| a.path().is_ident("inline"));
    if !args.allow_inline && !already_has_inline_attr {
        input_fn.attrs.push(parse_quote!(#[inline(never)]));
    }

    quote! { #input_fn }.into()
}

/// Walks every block in the function -- body, if/else arms, loop bodies,
/// match arms, closures -- and randomly inserts junk statements. Also wraps
/// expression-bodied match arms (`pat => value,`) and expression-bodied
/// closures (`|x| x + 1`) in a block first, since those have no `Block`
/// node otherwise and were previously unreachable injection points.
struct JunkInjector<R: Rng> {
    rng: R,
    density: f64,
    unsafe_ok: bool,
}

impl<R: Rng> JunkInjector<R> {
    fn inject(&mut self, block: &mut Block) {
        let has_tail_expr = matches!(block.stmts.last(), Some(Stmt::Expr(_, None)));

        let mut new_stmts: Vec<Stmt> = Vec::with_capacity(block.stmts.len() * 2);
        for stmt in block.stmts.drain(..) {
            if self.rng.gen_bool(self.density) {
                new_stmts.extend(random_junk_stmts(&mut self.rng, self.unsafe_ok));
            }
            new_stmts.push(stmt);
        }
        if !has_tail_expr && self.rng.gen_bool(self.density) {
            new_stmts.extend(random_junk_stmts(&mut self.rng, self.unsafe_ok));
        }
        block.stmts = new_stmts;
    }
}

impl<R: Rng> VisitMut for JunkInjector<R> {
    fn visit_block_mut(&mut self, block: &mut Block) {
        visit_mut::visit_block_mut(self, block);
        self.inject(block);
    }

    fn visit_arm_mut(&mut self, arm: &mut Arm) {
        visit_mut::visit_arm_mut(self, arm);
        let already_block = matches!(&*arm.body, Expr::Block(eb) if eb.label.is_none());
        if !already_block {
            let block = ensure_block(&mut *arm.body);
            self.inject(block);
        }
    }

    fn visit_expr_closure_mut(&mut self, closure: &mut ExprClosure) {
        visit_mut::visit_expr_closure_mut(self, closure);
        let already_block = matches!(&*closure.body, Expr::Block(eb) if eb.label.is_none());
        if !already_block {
            let block = ensure_block(&mut *closure.body);
            self.inject(block);
        }
    }
}

/// If `expr` isn't already a `{ .. }` block, wraps it in one, preserving it
/// as the block's tail expression, and returns a splice point for junk.
fn ensure_block(expr: &mut Expr) -> &mut Block {
    let needs_wrap = !matches!(expr, Expr::Block(eb) if eb.label.is_none());
    if needs_wrap {
        let inner = std::mem::replace(expr, Expr::Verbatim(proc_macro2::TokenStream::new()));
        *expr = parse_quote!({ #inner });
    }
    match expr {
        Expr::Block(eb) => &mut eb.block,
        _ => unreachable!("ensure_block: failed to wrap into a block expression"),
    }
}

/// Ordinary-looking name pool for junk locals -- avoids an obvious `_junk_`
/// tag that would stand out to anyone who ever sees these with debug info
/// attached (irrelevant for a stripped release binary, where local names
/// don't survive at all either way).
const NAME_POOL: &[&str] = &[
    "tmp", "acc", "idx", "chk", "aux", "scr", "cur", "buf", "val", "cnt", "off", "flag",
];

fn gen_name(rng: &mut impl Rng) -> Ident {
    let prefix = NAME_POOL[rng.gen_range(0..NAME_POOL.len())];
    let id: u32 = rng.r#gen();
    Ident::new(&format!("{prefix}_{id:x}"), proc_macro2::Span::call_site())
}

/// Emits `let #name: #ty = <opaque #value>;`. Rotates between `black_box`
/// and a volatile round-trip (only when `unsafe_ok`) so seed sites don't
/// all produce one identical, greppable instruction sequence.
fn opaque_seed(
    rng: &mut impl Rng,
    unsafe_ok: bool,
    name: &Ident,
    ty: proc_macro2::TokenStream,
    value: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if unsafe_ok && rng.gen_bool(0.5) {
        let tmp = gen_name(rng);
        quote! {
            let #name: #ty = {
                let #tmp: #ty = #value;
                unsafe { ::std::ptr::read_volatile(&#tmp) }
            };
        }
    } else {
        quote! {
            let #name: #ty = ::std::hint::black_box(#value);
        }
    }
}

/// Same rotation for the final "consume this so it isn't dead" sink, so
/// junk blocks don't all end in an identical `let _ = black_box(..)`.
fn opaque_sink(
    rng: &mut impl Rng,
    unsafe_ok: bool,
    value: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if unsafe_ok && rng.gen_bool(0.5) {
        let tmp = gen_name(rng);
        quote! {
            {
                let #tmp = #value;
                unsafe { let _ = ::std::ptr::read_volatile(&#tmp); }
            }
        }
    } else {
        quote! {
            let _ = ::std::hint::black_box(#value);
        }
    }
}

/// Generates one randomly-chosen junk snippet. Every seed literal goes
/// through `opaque_seed` *before* it's used in arithmetic/conditions, so
/// the optimizer can't fold through it or prove a branch dead.
fn random_junk_stmts(rng: &mut impl Rng, unsafe_ok: bool) -> Vec<Stmt> {
    let var = gen_name(rng);
    let seed_a = gen_name(rng);
    let seed_b = gen_name(rng);
    let mut tokens = proc_macro2::TokenStream::new();

    match rng.gen_range(0..6) {
        // Dead arithmetic.
        0 => {
            let a: i64 = rng.gen_range(1..1000);
            let b: i64 = rng.gen_range(1..1000);
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_a,
                quote!(i64),
                quote!(#a),
            ));
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_b,
                quote!(i64),
                quote!(#b),
            ));
            tokens.extend(quote! {
                let #var: i64 = #seed_a * #seed_b + #seed_a - #seed_b;
            });
            tokens.extend(opaque_sink(rng, unsafe_ok, quote!(#var)));
        }
        // Dead if/else.
        1 => {
            let a: i64 = rng.gen_range(1..1000);
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_a,
                quote!(i64),
                quote!(#a),
            ));
            tokens.extend(quote! {
                let #var: i64 = if #seed_a % 2 == 0 { #seed_a / 2 } else { #seed_a * 3 + 1 };
            });
            tokens.extend(opaque_sink(rng, unsafe_ok, quote!(#var)));
        }
        // Dead if/else-if/else (three real branches).
        2 => {
            let a: i64 = rng.gen_range(1..1000);
            let b: i64 = rng.gen_range(1..1000);
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_a,
                quote!(i64),
                quote!(#a),
            ));
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_b,
                quote!(i64),
                quote!(#b),
            ));
            tokens.extend(quote! {
                let #var: i64 = if #seed_a > #seed_b {
                    #seed_a - #seed_b
                } else if #seed_a < #seed_b {
                    #seed_b - #seed_a
                } else {
                    0
                };
            });
            tokens.extend(opaque_sink(rng, unsafe_ok, quote!(#var)));
        }
        // Dead floating point math.
        3 => {
            let a: i64 = rng.gen_range(1..1000);
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_a,
                quote!(f64),
                quote!(#a as f64),
            ));
            tokens.extend(quote! {
                let #var: f64 = (#seed_a.sin() * #seed_a.cos()).abs();
            });
            tokens.extend(opaque_sink(rng, unsafe_ok, quote!(#var)));
        }
        // Dead bitwise mix -- different instruction mix (rol/xor/and) from
        // the arithmetic shapes above.
        4 => {
            let a: i64 = rng.gen_range(1..1000);
            let b: i64 = rng.gen_range(1..1000);
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_a,
                quote!(i64),
                quote!(#a),
            ));
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_b,
                quote!(i64),
                quote!(#b),
            ));
            tokens.extend(quote! {
                let #var: i64 = ((#seed_a ^ #seed_b).rotate_left((#seed_a & 0x3f) as u32))
                    .wrapping_add(#seed_b) & 0xFFFF;
            });
            tokens.extend(opaque_sink(rng, unsafe_ok, quote!(#var)));
        }
        // Dead lookup/match -- jump-table-ish codegen, distinct again.
        _ => {
            let a: i64 = rng.gen_range(1..1000);
            tokens.extend(opaque_seed(
                rng,
                unsafe_ok,
                &seed_a,
                quote!(i64),
                quote!(#a),
            ));
            tokens.extend(quote! {
                let #var: i64 = match #seed_a.rem_euclid(5) {
                    0 => #seed_a + 11,
                    1 => #seed_a - 7,
                    2 => #seed_a * 2,
                    3 => #seed_a / 3,
                    _ => #seed_a ^ 0x5A,
                };
            });
            tokens.extend(opaque_sink(rng, unsafe_ok, quote!(#var)));
        }
    }

    let block: Block = syn::parse2(quote! { { #tokens } })
        .expect("obfuscate_macro: failed to parse generated junk");
    block.stmts
}
