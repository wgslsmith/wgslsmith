use rand::distributions::uniform::{SampleRange, SampleUniform};
use rand::prelude::{IteratorRandom, SliceRandom, StdRng};
use rand::Rng;
use rpds::HashTrieMap;

use crate::ast::{
    AssignmentLhs, BinOp, Expr, ExprNode, FnAttr, FnDecl, Lit, Module, ShaderStage, Statement, UnOp,
};
use crate::types::{DataType, ScalarType, TypeConstraints};

pub struct Generator {
    rng: StdRng,
}

#[derive(Clone, Debug)]
struct Scope {
    next_name: u32,
    consts: HashTrieMap<String, DataType>,
    const_types: TypeConstraints,
    vars: HashTrieMap<String, DataType>,
    var_types: TypeConstraints,
}

impl Scope {
    fn empty() -> Scope {
        Scope {
            next_name: 0,
            consts: HashTrieMap::new(),
            const_types: TypeConstraints::empty(),
            vars: HashTrieMap::new(),
            var_types: TypeConstraints::empty(),
        }
    }

    fn has_vars(&self) -> bool {
        !self.vars.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = (&String, &DataType)> {
        self.consts.iter().chain(self.vars.iter())
    }

    fn choose_var(&self, rng: &mut impl Rng) -> (&String, &DataType) {
        self.vars.iter().choose(rng).unwrap()
    }

    fn insert_let(&mut self, name: String, data_type: DataType) {
        self.consts.insert_mut(name, data_type);
        self.const_types.insert(data_type);
    }

    fn insert_var(&mut self, name: String, data_type: DataType) {
        self.vars.insert_mut(name, data_type);
        self.var_types.insert(data_type);
    }

    fn next_name(&mut self) -> String {
        let next = self.next_name;
        self.next_name += 1;
        format!("var_{}", next)
    }

    fn intersects(&self, constraints: &TypeConstraints) -> bool {
        constraints.intersects(&self.const_types.union(&self.var_types))
    }
}

#[derive(Clone, Copy)]
enum StatementType {
    LetDecl,
    VarDecl,
    Assignment,
    Compound,
    If,
}

#[derive(Clone, Copy, Debug)]
enum ExprType {
    Lit,
    TypeCons,
    Var,
    UnOp,
    BinOp,
}

impl Generator {
    pub fn new(rng: StdRng) -> Self {
        Generator { rng }
    }

    pub fn gen_module(&mut self) -> Module {
        log::info!("generating module");

        let stmt_count: u32 = self.rand_range(0..50);
        let mut stmts = ScopedStmtGenerator::new(&mut self.rng).gen_block(stmt_count);

        log::info!("generating output assignment");

        stmts.push(Statement::Assignment(
            AssignmentLhs::ArrayIndex {
                name: "output.data".to_owned(),
                index: ExprNode {
                    data_type: DataType::Scalar(ScalarType::U32),
                    expr: Expr::Lit(Lit::UInt(0)),
                },
            },
            ExprGenerator::new(&mut self.rng, &mut Scope::empty()).gen_expr(TypeConstraints::U32()),
        ));

        Module {
            entrypoint: FnDecl {
                attrs: vec![
                    FnAttr::Stage(ShaderStage::Compute),
                    FnAttr::WorkgroupSize(1),
                ],
                name: "main".to_owned(),
                inputs: vec![],
                output: None,
                body: stmts,
            },
        }
    }

    fn rand_range<T: SampleUniform, R: SampleRange<T>>(&mut self, r: R) -> T {
        self.rng.gen_range(r)
    }
}

struct ScopedStmtGenerator<'a> {
    rng: &'a mut StdRng,
    scope: Scope,
}

impl<'a> ScopedStmtGenerator<'a> {
    pub fn new(rng: &mut StdRng) -> ScopedStmtGenerator {
        ScopedStmtGenerator {
            rng,
            scope: Scope::empty(),
        }
    }

    fn new_scope(&mut self) -> ScopedStmtGenerator {
        ScopedStmtGenerator {
            rng: self.rng,
            scope: self.scope.clone(),
        }
    }

    pub fn gen_stmt(&mut self) -> Statement {
        log::info!("generating statement");

        let mut allowed = vec![
            StatementType::LetDecl,
            StatementType::VarDecl,
            StatementType::Compound,
            StatementType::If,
        ];

        if self.scope.has_vars() {
            allowed.push(StatementType::Assignment);
        }

        match allowed.choose(&mut self.rng).unwrap() {
            StatementType::LetDecl => Statement::LetDecl(
                self.scope.next_name(),
                ExprGenerator::new(&mut self.rng, &mut self.scope)
                    .gen_expr(TypeConstraints::Unconstrained()),
            ),
            StatementType::VarDecl => Statement::VarDecl(
                self.scope.next_name(),
                ExprGenerator::new(&mut self.rng, &mut self.scope)
                    .gen_expr(TypeConstraints::Unconstrained()),
            ),
            StatementType::Assignment => {
                let (name, &data_type) = self.scope.choose_var(&mut self.rng);
                Statement::Assignment(
                    AssignmentLhs::SimpleVar(name.clone()),
                    ExprGenerator::new(&mut self.rng, &mut self.scope).gen_expr(&data_type.into()),
                )
            }
            StatementType::Compound => Statement::Compound(self.new_scope().gen_block(1)),
            StatementType::If => Statement::If(
                ExprGenerator::new(&mut self.rng, &mut self.scope)
                    .gen_expr(TypeConstraints::Bool()),
                self.new_scope().gen_block(1),
            ),
        }
    }

    pub fn gen_block(&mut self, count: u32) -> Vec<Statement> {
        log::info!("generating block of {} statements", count);

        let mut stmts = vec![];

        for _ in 0..count {
            let stmt = self.gen_stmt();

            // If we generated a variable declaration, track it in the environment
            if let Statement::LetDecl(name, expr) = &stmt {
                self.scope.insert_let(name.clone(), expr.data_type);
            } else if let Statement::VarDecl(name, expr) = &stmt {
                self.scope.insert_var(name.clone(), expr.data_type);
            }

            stmts.push(stmt);
        }

        stmts
    }
}

struct ExprGenerator<'a> {
    rng: &'a mut StdRng,
    scope: &'a mut Scope,
    depth: u32,
}

impl<'a> ExprGenerator<'a> {
    pub fn new(rng: &'a mut StdRng, scope: &'a mut Scope) -> ExprGenerator<'a> {
        ExprGenerator {
            rng,
            scope,
            depth: 0,
        }
    }

    pub fn gen_expr(&mut self, constraints: &TypeConstraints) -> ExprNode {
        log::info!(
            "generating expr with {:?}, depth={}",
            constraints,
            self.depth
        );

        let mut allowed = vec![];

        if constraints.intersects(TypeConstraints::Scalar()) {
            allowed.push(ExprType::Lit);
        }

        if constraints.intersects(TypeConstraints::Vec()) {
            allowed.push(ExprType::TypeCons);
        }

        if self.depth < 5 {
            allowed.push(ExprType::UnOp);

            if constraints.intersects(&TypeConstraints::Scalar().union(TypeConstraints::VecInt())) {
                allowed.push(ExprType::BinOp);
            }

            if self.scope.intersects(constraints) {
                allowed.push(ExprType::Var);
            }
        }

        log::info!("allowed constructions: {:?}", allowed);

        match *allowed.choose(&mut self.rng).unwrap() {
            ExprType::Lit => {
                let (lit, t) = self.gen_lit(constraints);
                ExprNode {
                    data_type: t,
                    expr: Expr::Lit(lit),
                }
            }
            ExprType::TypeCons => {
                log::info!("generating type_cons with {:?}", constraints);

                let data_type = constraints
                    .intersection(TypeConstraints::Vec())
                    .select(&mut self.rng);

                log::info!("generating type cons with t={}", data_type);

                let mut args = vec![];

                let (n, t) = match data_type {
                    DataType::Scalar(t) => (1, t),
                    DataType::Vector(n, t) => (n, t),
                };

                let constraints = DataType::Scalar(t).into();
                for _ in 0..n {
                    args.push(self.gen_expr(&constraints))
                }

                ExprNode {
                    data_type,
                    expr: Expr::TypeCons(data_type, args),
                }
            }
            ExprType::UnOp => {
                self.depth += 1;

                let op = self.gen_un_op(constraints);
                let constraints = match op {
                    UnOp::Neg => constraints
                        .intersection(&TypeConstraints::I32().union(TypeConstraints::VecI32())),
                    UnOp::Not => constraints
                        .intersection(&TypeConstraints::Bool().union(TypeConstraints::VecBool())),
                    UnOp::BitNot => constraints
                        .intersection(&TypeConstraints::Int().union(TypeConstraints::VecInt())),
                };

                let expr = self.gen_expr(&constraints);

                self.depth -= 1;

                ExprNode {
                    data_type: expr.data_type,
                    expr: Expr::UnOp(op, Box::new(expr)),
                }
            }
            ExprType::BinOp => {
                self.depth += 1;

                let op = self.gen_bin_op(constraints);
                let lconstraints = match op {
                    BinOp::Plus
                    | BinOp::Minus
                    | BinOp::Times
                    | BinOp::Divide
                    | BinOp::Mod
                    | BinOp::BitAnd
                    | BinOp::BitOr
                    | BinOp::BitXOr
                    | BinOp::LShift
                    | BinOp::RShift => constraints
                        .intersection(&TypeConstraints::Int().union(TypeConstraints::VecInt())),
                    BinOp::LogAnd | BinOp::LogOr => {
                        constraints.intersection(TypeConstraints::Bool())
                    }
                };

                let l = self.gen_expr(&lconstraints);
                let rconstraints = match op {
                    // For shifts, right operand must be u32
                    BinOp::LShift | BinOp::RShift => match l.data_type {
                        DataType::Scalar(_) => TypeConstraints::U32().clone(),
                        DataType::Vector(n, _) => DataType::Vector(n, ScalarType::U32).into(),
                    },
                    // For everything else right operand must be same type as left
                    _ => l.data_type.into(),
                };

                let r = self.gen_expr(&rconstraints);

                self.depth -= 1;

                ExprNode {
                    data_type: l.data_type,
                    expr: Expr::BinOp(op, Box::new(l), Box::new(r)),
                }
            }
            ExprType::Var => {
                log::info!(
                    "generating var with {:?}, scope={:?}",
                    constraints,
                    self.scope
                );

                let (name, &data_type) = self
                    .scope
                    .iter()
                    .filter(|(_, t)| constraints.intersects(&(*t).into()))
                    .choose(&mut self.rng)
                    .unwrap();

                ExprNode {
                    data_type,
                    expr: Expr::Var(name.to_owned()),
                }
            }
        }
    }

    fn gen_lit(&mut self, constraints: &TypeConstraints) -> (Lit, DataType) {
        log::info!("generating lit with {:?}", constraints);

        // Select a random concrete type from the constraints
        let t = constraints
            .intersection(TypeConstraints::Scalar())
            .select(&mut self.rng);

        log::info!("generating lit with t={}", t);

        let lit = match t {
            DataType::Scalar(t) => match t {
                ScalarType::Bool => Lit::Bool(self.rng.gen()),
                ScalarType::I32 => Lit::Int(self.rng.gen()),
                ScalarType::U32 => Lit::UInt(self.rng.gen()),
            },
            _ => unreachable!(),
        };

        (lit, t)
    }

    fn gen_un_op(&mut self, constraints: &TypeConstraints) -> UnOp {
        log::info!("generating un_op with {:?}", constraints);

        let mut allowed = vec![];

        if constraints.intersects(&TypeConstraints::I32().union(TypeConstraints::VecI32())) {
            allowed.push(UnOp::Neg);
        }

        if constraints.intersects(&TypeConstraints::Bool().union(TypeConstraints::VecBool())) {
            allowed.push(UnOp::Not);
        }

        if constraints.intersects(&TypeConstraints::Int().union(TypeConstraints::VecInt())) {
            allowed.push(UnOp::BitNot)
        }

        log::info!("allowed constructions: {:?}", allowed);

        *allowed.choose(&mut self.rng).unwrap()
    }

    fn gen_bin_op(&mut self, constraints: &TypeConstraints) -> BinOp {
        log::info!("generating bin_op with {:?}", constraints);

        let mut allowed = vec![];

        if constraints.intersects(&TypeConstraints::Int().union(TypeConstraints::VecInt())) {
            allowed.extend_from_slice(&[
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
            ]);
        }

        if constraints.intersects(TypeConstraints::Bool()) {
            // TODO: Non short-circuiting logical & and | are currently broken in naga
            // https://github.com/gfx-rs/naga/issues/1574
            allowed.extend_from_slice(&[BinOp::LogAnd, BinOp::LogOr]);
        }

        log::info!("allowed constructions: {:?}", allowed);

        *allowed.choose(&mut self.rng).unwrap()
    }
}
