use ast::types::DataType;
use ast::{AttrList, FnDecl, FnInput, FnOutput};
use rand::prelude::StdRng;
use rand::Rng;

use crate::Options;

use super::cx::Context;
use super::scope::Scope;
use super::stmt;

#[tracing::instrument(skip(rng, cx, options))]
pub fn gen_fn(rng: &mut StdRng, cx: &Context, options: &Options, return_ty: &DataType) -> FnDecl {
    let name = cx.fns.borrow_mut().next_fn();

    let arg_count = rng.gen_range(0..5);
    let args = (0..arg_count)
        .map(|i| FnInput {
            attrs: AttrList(vec![]),
            name: format!("arg_{}", i),
            data_type: cx.types.borrow().select(rng),
        })
        .collect();

    let stmt_count = rng.gen_range(options.fn_min_stmts..=options.fn_max_stmts);
    // TODO: Global scope should be passed here to allow access to global variables
    let block = stmt::gen_block_with_return(
        rng,
        cx,
        &Scope::empty(),
        Some(return_ty.clone()),
        options,
        stmt_count,
    );

    FnDecl {
        attrs: AttrList(vec![]),
        name,
        inputs: args,
        output: Some(FnOutput {
            attrs: AttrList(vec![]),
            data_type: return_ty.clone(),
        }),
        body: block,
    }
}
