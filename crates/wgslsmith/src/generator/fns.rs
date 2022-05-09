use ast::types::DataType;
use ast::{FnDecl, FnInput, FnOutput};
use rand::Rng;

use super::scope::Scope;

impl<'a> super::Generator<'a> {
    pub fn gen_fn(&mut self, return_type: &DataType) -> FnDecl {
        let saved_expression_depth = self.expression_depth;
        let saved_block_depth = self.block_depth;

        self.expression_depth = 0;
        self.block_depth = 0;

        let name = self.cx.fns.next_fn();

        let arg_count = self.rng.gen_range(0..5);
        let args = (0..arg_count)
            .map(|i| FnInput {
                attrs: vec![],
                name: format!("arg_{}", i),
                data_type: self.cx.types.select(self.rng),
            })
            .collect();

        let stmt_count = self
            .rng
            .gen_range(self.options.fn_min_stmts..=self.options.fn_max_stmts);

        let (_, block) = self.with_scope(Scope::empty(), |this| {
            this.gen_stmt_block_with_return(stmt_count, Some(return_type.clone()))
        });

        self.expression_depth = saved_expression_depth;
        self.block_depth = saved_block_depth;

        FnDecl {
            attrs: vec![],
            name,
            inputs: args,
            output: Some(FnOutput {
                attrs: vec![],
                data_type: return_type.clone(),
            }),
            body: block,
        }
    }
}
