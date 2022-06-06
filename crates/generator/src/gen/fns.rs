use std::mem;

use ast::types::DataType;
use ast::{FnDecl, FnInput, FnOutput};
use rand::Rng;

impl<'a> super::Generator<'a> {
    pub fn gen_fn(&mut self, params: Vec<FnInput>, return_type: &DataType) -> FnDecl {
        let saved_state = mem::take(&mut self.fn_state);

        let name = self.cx.fns.next_fn();

        let stmt_count = self
            .rng
            .gen_range(self.options.fn_min_stmts..=self.options.fn_max_stmts);

        let mut function_scope = self.global_scope.clone();

        for param in &params {
            function_scope.insert_readonly(param.name.clone(), param.data_type.clone());
        }

        let (_, block) = self.with_scope(function_scope, |this| {
            this.gen_stmt_block_with_return(stmt_count, Some(return_type.clone()))
        });

        self.fn_state = saved_state;

        FnDecl {
            attrs: vec![],
            name,
            inputs: params,
            output: Some(FnOutput {
                attrs: vec![],
                data_type: return_type.clone(),
            }),
            body: block,
        }
    }
}
