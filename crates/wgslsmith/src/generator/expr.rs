use rand::prelude::SliceRandom;
use rand::Rng;

use ast::types::{DataType, MemoryViewType, ScalarType};
use ast::{
    BinOp, BinOpExpr, Expr, ExprNode, FnCallExpr, FnInput, Lit, Postfix, PostfixExpr, StructDecl,
    TypeConsExpr, UnOp, UnOpExpr, VarDeclStatement, VarExpr,
};
use tap::Pipe;

use super::cx::Func;

#[derive(Clone, Copy, Debug)]
enum ExprType {
    Lit,
    TypeCons,
    Var,
    UnOp,
    BinOp,
    FnCall,
}

impl<'a> super::Generator<'a> {
    pub fn gen_expr(&mut self, ty: &DataType) -> ExprNode {
        let mut allowed = vec![];

        match ty {
            DataType::Scalar(_) => allowed.push(ExprType::Lit),
            DataType::Vector(_, _) => allowed.push(ExprType::TypeCons),
            DataType::Array(_, _) => allowed.push(ExprType::TypeCons),
            DataType::Struct(_) => allowed.push(ExprType::TypeCons),
            DataType::Ptr(view) => return self.gen_pointer_expr(view),
            DataType::Ref(_) => panic!("explicit request to generate ref expression: `{ty}`"),
        }

        if self.expression_depth < 5 {
            // Unary operators are available for all scalars and vectors.
            if matches!(ty, DataType::Scalar(_) | DataType::Vector(_, _)) {
                allowed.push(ExprType::UnOp);
            }

            // Binary operators are available for all scalars, and for {i32,u32,f32} vectors.
            if matches!(
                ty,
                DataType::Scalar(_)
                    | DataType::Vector(_, ScalarType::I32 | ScalarType::U32 | ScalarType::F32)
            ) {
                allowed.push(ExprType::BinOp);
            }

            // Function calls are available if we have a function that returns the target type,
            // or we are able to generate a new function.
            // TODO: naga currently has issues with functions that return arrays:
            // https://github.com/gfx-rs/naga/issues/1930
            // https://github.com/gfx-rs/naga/issues/1910
            if !matches!(ty, DataType::Array(_, _))
                && (self.cx.fns.contains_type(ty) || self.can_gen_fn(ty))
            {
                allowed.push(ExprType::FnCall);
            }
        }

        if !self.scope.of_type(ty).is_empty() {
            allowed.push(ExprType::Var);
        }

        tracing::info!("allowed constructions: {:?}", allowed);

        match *allowed.choose(&mut self.rng).unwrap() {
            ExprType::Lit => self.gen_lit_expr(ty),
            ExprType::TypeCons => self.gen_type_cons_expr(ty),
            ExprType::UnOp => self.gen_un_op_expr(ty),
            ExprType::BinOp => self.gen_bin_op_expr(ty),
            ExprType::Var => self.gen_var_expr(ty),
            ExprType::FnCall => self.gen_fn_call_expr(ty),
        }
    }

    fn can_gen_fn(&self, _return_type: &DataType) -> bool {
        self.cx.fns.len() < self.options.max_fns
    }

    fn gen_pointer_expr(&mut self, mem_view: &MemoryViewType) -> ExprNode {
        let ref_type = DataType::Ref(mem_view.clone());
        let available = self.scope.of_type(&ref_type);

        // If there is a variable available for the target type, we use that.
        // Otherwise we need to introduce a new local variable.
        if let Some((name, data_type)) = available.choose(&mut self.rng) {
            let mut var_expr = VarExpr::new(name).into_node(data_type.clone());

            if var_expr.data_type.dereference() != mem_view.inner.as_ref() {
                var_expr = self.gen_accessor(&ref_type, var_expr);
            }

            UnOpExpr::new(UnOp::AddressOf, var_expr).into()
        } else {
            let ident = self.scope.next_name();
            let initializer = self.gen_expr(mem_view.inner.as_ref());
            self.current_block
                .push(VarDeclStatement::new(ident.clone(), None, Some(initializer)).into());
            UnOpExpr::new(UnOp::AddressOf, VarExpr::new(ident).into_node(ref_type)).into()
        }
    }

    pub fn gen_const_expr(&mut self, ty: &DataType) -> ExprNode {
        match ty {
            DataType::Scalar(_) => self.gen_lit_expr(ty),
            ty => self.gen_const_type_cons_expr(ty),
        }
    }

    fn gen_lit_expr(&mut self, ty: &DataType) -> ExprNode {
        let lit = self.gen_lit(ty);
        ExprNode {
            data_type: ty.clone(),
            expr: Expr::Lit(lit),
        }
    }

    fn gen_type_cons_expr(&mut self, ty: &DataType) -> ExprNode {
        tracing::info!("generating type_cons with {:?}", ty);

        self.expression_depth += 1;

        let args = match ty {
            DataType::Scalar(t) => vec![self.gen_expr(&DataType::Scalar(*t))],
            DataType::Vector(n, t) => (0..*n)
                .map(|_| self.gen_expr(&DataType::Scalar(*t)))
                .collect(),
            DataType::Array(_, _) => vec![],
            DataType::Struct(decl) => decl
                .members
                .iter()
                .map(|it| self.gen_expr(&it.data_type))
                .collect(),
            DataType::Ptr(_) | DataType::Ref(_) => unimplemented!("no type constructor for `{ty}`"),
        };

        self.expression_depth -= 1;

        TypeConsExpr::new(ty.clone(), args).into()
    }

    fn gen_const_type_cons_expr(&mut self, ty: &DataType) -> ExprNode {
        let args = match ty {
            DataType::Scalar(t) => vec![self.gen_expr(&DataType::Scalar(*t))],
            DataType::Vector(n, t) => (0..*n)
                .map(|_| self.gen_const_expr(&DataType::Scalar(*t)))
                .collect(),
            DataType::Array(ty, Some(n)) => (0..*n).map(|_| self.gen_const_expr(&*ty)).collect(),
            DataType::Array(_, None) => panic!("runtime sized array is not constructable"),
            DataType::Struct(decl) => decl
                .members
                .iter()
                .map(|it| self.gen_const_expr(&it.data_type))
                .collect(),
            DataType::Ptr(_) | DataType::Ref(_) => unimplemented!("no type constructor for `{ty}`"),
        };

        TypeConsExpr::new(ty.clone(), args).into()
    }

    fn gen_un_op_expr(&mut self, ty: &DataType) -> ExprNode {
        self.expression_depth += 1;

        let op = self.gen_un_op(ty);
        let expr = self.gen_expr(ty);

        self.expression_depth -= 1;

        UnOpExpr::new(op, expr).into()
    }

    fn gen_bin_op_expr(&mut self, ty: &DataType) -> ExprNode {
        self.expression_depth += 1;

        let op = self.gen_bin_op(ty);
        let l_ty = match op {
            // These operators work on scalar/vector integers.
            // The result type depends on the operand type.
            | BinOp::Plus
            | BinOp::Minus
            | BinOp::Times
            | BinOp::Divide
            | BinOp::Mod
            | BinOp::BitXOr
            | BinOp::LShift
            | BinOp::RShift => ty.clone(),

            // These operators work on any scalar/vector.
            // The result type depends on the operand type.
            BinOp::BitAnd | BinOp::BitOr => ty.clone(),

            // These operators only work on scalar bools.
            BinOp::LogAnd | BinOp::LogOr => ty.clone(),

            // These operators work on scalar/vector integers.
            // The number of components in the result type depends on the operands, but the
            // actual type does not.
            BinOp::Less | BinOp::LessEqual | BinOp::Greater | BinOp::GreaterEqual => ty.map(
                [ScalarType::I32, ScalarType::U32, ScalarType::F32]
                    .choose(&mut self.rng)
                    .copied()
                    .unwrap(),
            ),

            // These operators work on scalar/vector integers and bools.
            // The number of components in the result type depends on the operands, but the
            // actual type does not.
            BinOp::Equal | BinOp::NotEqual => ty.map(
                [
                    ScalarType::I32,
                    ScalarType::U32,
                    ScalarType::F32,
                    ScalarType::Bool,
                ]
                .choose(&mut self.rng)
                .copied()
                .unwrap(),
            ),
        };

        let l = self.gen_expr(&l_ty);
        let r_ty = match op {
            // For shifts, right operand must be u32
            BinOp::LShift | BinOp::RShift => l_ty.map(ScalarType::U32),
            // For everything else right operand must be same type as left
            _ => l_ty.clone(),
        };

        let r = self.gen_expr(&r_ty);

        self.expression_depth -= 1;

        BinOpExpr::new(op, l, r).into()
    }

    fn gen_var_expr(&mut self, ty: &DataType) -> ExprNode {
        tracing::info!("generating var with {:?}, scope={:?}", ty, self.scope);

        let (name, data_type) = self.scope.of_type(ty).choose(&mut self.rng).unwrap();
        let expr = VarExpr::new(name).into_node(data_type.clone());

        if expr.data_type.dereference() == ty {
            return expr;
        }

        // Variable does not have the same type as the target, so we need to generate an
        // accessor to get an appropriate field
        self.gen_accessor(ty, expr)
    }

    fn gen_fn_call_expr(&mut self, ty: &DataType) -> ExprNode {
        let expr = self.gen_raw_fn_call_expr(ty);

        if expr.data_type == *ty {
            return expr;
        }

        // Variable does not have the same type as the target, so we need to generate an
        // accessor to get an appropriate field
        self.gen_accessor(ty, expr)
    }

    fn gen_raw_fn_call_expr(&mut self, ty: &DataType) -> ExprNode {
        // Produce a function call with p=0.8 or p=1 if max functions reached
        if self.cx.fns.len() > self.options.max_fns || self.rng.gen_bool(0.8) {
            if let Some(func) = self.cx.fns.select(self.rng, ty) {
                let (name, params, return_type) = match func.as_ref() {
                    Func::Builtin(builtin, overload) => (
                        builtin.as_ref(),
                        overload.params.as_slice(),
                        Some(&overload.return_type),
                    ),
                    Func::User(sig) => (
                        sig.ident.as_str(),
                        sig.params.as_slice(),
                        sig.return_type.as_ref(),
                    ),
                };

                self.expression_depth += 1;
                let args = params.iter().map(|ty| self.gen_expr(ty)).collect();
                self.expression_depth -= 1;

                return FnCallExpr::new(name, args).into_node(return_type.unwrap().clone());
            }
        }

        // Otherwise generate a new function with the target return type

        let arg_count: i32 = self.rng.gen_range(0..5);

        let mut params = vec![];
        let mut args = vec![];

        for i in 0..arg_count {
            let expr = if self.options.enable_pointers
                && self.scope.has_references()
                && self.rng.gen_bool(0.2)
            {
                let (name, mem_view) = self.scope.choose_reference(self.rng);
                let var_expr = VarExpr::new(name).into_node(DataType::Ref(mem_view.clone()));
                UnOpExpr::new(UnOp::AddressOf, var_expr).into()
            } else {
                self.expression_depth += 1;
                let data_type = self.cx.types.select(self.rng);
                let expr = self.gen_expr(&data_type);
                self.expression_depth -= 1;
                expr
            };

            params.push(FnInput {
                attrs: vec![],
                data_type: expr.data_type.dereference().clone(),
                name: format!("arg_{i}"),
            });

            args.push(expr);
        }

        let decl = self.gen_fn(params, ty);

        // Add the new function to the context
        let func = self.cx.fns.insert(decl);

        FnCallExpr::new(func.ident(), args).into_node(ty.clone())
    }

    fn gen_accessor(&mut self, target: &DataType, expr: ExprNode) -> ExprNode {
        match expr.data_type.dereference() {
            DataType::Scalar(_) => unreachable!(),
            DataType::Vector(n, _) => self.gen_vector_accessor(*n, target, expr),
            DataType::Array(_, _) => self.gen_array_accessor(target, expr),
            DataType::Struct(decl) => self.gen_struct_accessor(&decl.clone(), target, expr),
            DataType::Ptr(_) => self.gen_pointer_deref(target, expr),
            DataType::Ref(_) => todo!(),
        }
    }

    fn gen_vector_accessor(&mut self, size: u8, target: &DataType, expr: ExprNode) -> ExprNode {
        let accessor = super::utils::gen_vector_accessor(self.rng, size, target);
        PostfixExpr::new(expr, Postfix::member(accessor)).into()
    }

    fn gen_array_accessor(&mut self, target: &DataType, expr: ExprNode) -> ExprNode {
        let index = self.gen_expr(&ScalarType::I32.into());
        let expr: ExprNode = PostfixExpr::new(expr, Postfix::index(index)).into();

        if expr.data_type.dereference() == target {
            return expr;
        }

        self.gen_accessor(target, expr)
    }

    fn gen_struct_accessor(
        &mut self,
        decl: &StructDecl,
        target: &DataType,
        expr: ExprNode,
    ) -> ExprNode {
        let member = decl.accessors_of(target).choose(self.rng).unwrap();
        let expr = PostfixExpr::new(expr, Postfix::member(&member.name)).into();

        if member.data_type.dereference() == target {
            return expr;
        }

        self.gen_accessor(target, expr)
    }

    fn gen_pointer_deref(&mut self, target: &DataType, expr: ExprNode) -> ExprNode {
        let expr = ExprNode::from(UnOpExpr::new(UnOp::Deref, expr));

        if expr.data_type.dereference() == target {
            return expr;
        }

        self.gen_accessor(target, expr)
    }

    #[tracing::instrument(skip(self))]
    fn gen_lit(&mut self, ty: &DataType) -> Lit {
        tracing::info!("generating lit with {:?}", ty);

        match ty {
            DataType::Scalar(t) => match t {
                ScalarType::Bool => Lit::Bool(self.rng.gen()),
                ScalarType::I32 => Lit::I32(self.rng.gen()),
                ScalarType::U32 => Lit::U32(self.rng.gen()),
                ScalarType::F32 => Lit::F32(self.rng.gen_range(-1000i32..1000i32).pipe(|it| {
                    if it == 0 {
                        1
                    } else {
                        it
                    }
                }) as f32),
            },
            _ => unreachable!(),
        }
    }

    #[tracing::instrument(skip(self))]
    fn gen_un_op(&mut self, ty: &DataType) -> UnOp {
        tracing::info!("generating un_op with {:?}", ty);

        let scalar_ty = match ty {
            DataType::Scalar(ty) => ty,
            DataType::Vector(_, ty) => ty,
            DataType::Array(_, _) => unreachable!(),
            DataType::Struct(_) => unreachable!(),
            DataType::Ptr(_) => todo!(),
            DataType::Ref(_) => todo!(),
        };

        match scalar_ty {
            ScalarType::Bool => UnOp::Not,
            ScalarType::U32 => UnOp::BitNot,
            ScalarType::I32 => [UnOp::Neg, UnOp::BitNot]
                .choose(&mut self.rng)
                .copied()
                .unwrap(),
            ScalarType::F32 => UnOp::Neg,
        }
    }

    #[tracing::instrument(skip(self))]
    fn gen_bin_op(&mut self, ty: &DataType) -> BinOp {
        let scalar_ty = match ty {
            DataType::Scalar(ty) => ty,
            DataType::Vector(_, ty) => ty,
            DataType::Array(_, _) => unreachable!(),
            DataType::Struct(_) => unreachable!(),
            DataType::Ptr(_) => todo!(),
            DataType::Ref(_) => todo!(),
        };

        let allowed: &[BinOp] = match scalar_ty {
            ScalarType::Bool => &[
                BinOp::Equal,
                BinOp::NotEqual,
                BinOp::Less,
                BinOp::LessEqual,
                BinOp::Greater,
                BinOp::GreaterEqual,
                BinOp::BitAnd,
                BinOp::BitOr,
            ],
            ScalarType::I32 | ScalarType::U32 => &[
                BinOp::Plus,
                BinOp::Minus,
                BinOp::Times,
                BinOp::Divide,
                BinOp::Mod,
                BinOp::BitAnd,
                BinOp::BitOr,
                BinOp::BitXOr,
                BinOp::LShift,
                BinOp::RShift,
            ],
            ScalarType::F32 => &[BinOp::Plus, BinOp::Minus, BinOp::Times],
        };

        let mut allowed = allowed.to_vec();

        if let DataType::Scalar(ScalarType::Bool) = ty {
            allowed.extend_from_slice(&[BinOp::LogAnd, BinOp::LogOr]);
        }

        *allowed.choose(&mut self.rng).unwrap()
    }
}
