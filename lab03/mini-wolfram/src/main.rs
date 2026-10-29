use crate::Const::Numeric;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Var {
    X,
    Y,
    Z,
}

impl Var {
    fn to_string(&self) -> String {
        match self {
            Var::X => "X".to_string(),
            Var::Y => "Y".to_string(),
            Var::Z => "Z".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
enum Const {
    Numeric(i64),
    Named(String),
}

impl Const {
    fn to_string(&self) -> String {
        match self {
            Const::Numeric(num) => num.to_string(),
            Const::Named(str) => str.clone(),
        }
    }
}

#[derive(Clone, Debug)]
enum E {
    Add(Box<E>, Box<E>),
    Neg(Box<E>),
    Mul(Box<E>, Box<E>),
    Inv(Box<E>),
    Const(Const),
    Func { name: String, arg: Box<E> },
    Var(Var),
}

impl E {
    fn add(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        Box::new(E::Add(a, b))
    }
    fn neg(a: Box<Self>) -> Box<Self> {
        Box::new(E::Neg(a))
    }
    fn mul(a: Box<Self>, b: Box<Self>) -> Box<Self> {
        Box::new(E::Mul(a, b))
    }
    fn inv(a: Box<Self>) -> Box<Self> {
        Box::new(E::Inv(a))
    }
    fn constant(a: Const) -> Box<Self> {
        Box::new(E::Const(a))
    }
    fn func(str: String, a: Box<E>) -> Box<Self> {
        Box::new(E::Func { name: str, arg: a })
    }
    fn var(v: Var) -> Box<Self> {
        Box::new(E::Var(v))
    }

    fn to_string(&self) -> String {
        match self {
            E::Add(a, b) => format!("({} + {})", a.to_string(), b.to_string()),
            E::Neg(e) => format!("-({})", e.to_string()),
            E::Mul(a, b) => format!("({} * {})", a.to_string(), b.to_string()),
            E::Inv(a) => format!("1/({})", a.to_string()),
            E::Const(c) => c.to_string(),
            E::Func { name, arg } => format!("{}({})", name, arg.to_string()),
            E::Var(v) => v.to_string(),
        }
    }

    fn arg_count(&self) -> u32 {
        match self {
            E::Add(..) | E::Mul(..) => 2,
            E::Neg(_) | E::Inv(_) | E::Func { .. } => 1,
            _ => 0,
        }
    }

    fn diff(self, by: Var) -> Box<Self> {
        match self {
            E::Add(a, b) => E::add(a.diff(by), b.diff(by)),
            E::Neg(e) => E::neg(e.diff(by)),
            E::Mul(a, b) => E::add(E::mul(a.clone().diff(by), b.clone()), E::mul(a, b.diff(by))),
            E::Inv(e) => E::mul(E::neg(E::inv(E::mul(e.clone(), e.clone()))), e.diff(by)),
            E::Const(_) => E::constant(Const::Numeric(0)),
            E::Func { name, arg } => E::mul(
                E::func(format!("{}_{}", name, by.to_string()), arg.clone()),
                arg.diff(by),
            ),
            E::Var(var) => {
                if var == by {
                    E::constant(Numeric(1))
                } else {
                    E::constant(Numeric(0))
                }
            }
        }
    }

    fn unpack_inv_inv(self) -> Option<Box<Self>> {
        let Self::Inv(outer) = self else {
            return None;
        };
        let Self::Inv(inner) = *outer else {
            return None;
        };

        Some(inner)
    }

    fn uninv(self: Box<Self>) -> Box<Self> {
        let mut exp = self;

        while let Some(inner) = E::unpack_inv_inv(*exp.clone()) {
            exp = inner;
        }

        exp
    }

    fn unpack_neg_neg(self) -> Option<Box<Self>> {
        if let E::Neg(outer) = self
            && let E::Neg(inner) = *outer
        {
            return Some(inner);
        }
        None
    }

    fn unneg(self: Box<Self>) -> Box<Self> {
        let mut exp = self;

        while let Some(inner) = E::unpack_neg_neg(*exp.clone()) {
            exp = inner;
        }

        exp
    }

    fn substitute(self, name: &str, value: Box<Self>) -> Box<Self> {
        match self {
            Self::Const(Const::Named(n)) if n == name => value,

            Self::Add(e1, e2) => E::add(
                e1.substitute(name, value.clone()),
                e2.substitute(name, value),
            ),

            Self::Neg(e) => E::neg(e.substitute(name, value)),

            Self::Mul(e1, e2) => E::mul(
                e1.substitute(name, value.clone()),
                e2.substitute(name, value),
            ),

            Self::Inv(e) => E::inv(e.substitute(name, value)),

            Self::Func {
                name: func_name,
                arg,
            } => E::func(func_name, arg.substitute(name, value)),

            other => Box::new(other),
        }
    }
}

fn main() {
    let expr1 = E::add(E::constant(Const::Numeric(2)), E::var(Var::X));
    let expr2 = E::mul(E::neg(E::var(Var::Y)), E::inv(E::var(Var::Z)));

    let complex = E::add(E::func("f".into(), expr1), E::func("g".into(), expr2));

    println!("exp: {}", complex.to_string());
    println!("num of arg: {}", complex.arg_count());

    let d = complex.clone().diff(Var::X);
    println!("d/dX:                 {}", d.to_string());

    let with_named = E::add(E::constant(Const::Named("a".into())), E::var(Var::X));
    let substituted = with_named.substitute("a", E::constant(Const::Numeric(10)));
    println!("a=10: {}", substituted.to_string());

    let double_inv = E::inv(E::inv(E::var(Var::X)));
    println!("1/(1/(X)) = {}", double_inv.uninv().to_string());

    let double_neg = E::neg(E::neg(E::neg(E::neg(E::var(Var::Y)))));
    println!("-(-(-(-(Y)))) = {}", double_neg.unneg().to_string());

    let di = E::inv(E::inv(E::var(Var::Z)));
    if let Some(inner) = di.unpack_inv_inv() {
        println!("unpack_inv_inv = {}", inner.to_string());
    }

    let dn = E::neg(E::neg(E::var(Var::Z)));
    if let Some(inner) = dn.unpack_neg_neg() {
        println!("unpack_neg_neg  = {}", inner.to_string());
    }

    println!("Var::X = {}", Var::X.to_string());
    println!(
        "Const::Named(\"k\") = {}",
        Const::Named("k".into()).to_string()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_to_string() {
        let c_num = Const::Numeric(42);
        let c_name = Const::Named("a".into());
        assert_eq!(c_num.to_string(), "42");
        assert_eq!(c_name.to_string(), "a");
    }

    #[test]
    fn test_var_to_string() {
        assert_eq!(Var::X.to_string(), "X");
        assert_eq!(Var::Y.to_string(), "Y");
        assert_eq!(Var::Z.to_string(), "Z");
    }

    #[test]
    fn test_builder_constant_var() {
        let e_const = E::constant(Const::Numeric(5));
        let e_var = E::var(Var::X);
        assert_eq!(e_const.to_string(), "5");
        assert_eq!(e_var.to_string(), "X");
    }

    #[test]
    fn test_builder_add() {
        let expr = E::add(E::constant(Const::Numeric(2)), E::var(Var::X));
        assert_eq!(expr.to_string(), "(2 + X)");
    }

    #[test]
    fn test_builder_neg() {
        let expr = E::neg(E::var(Var::X));
        assert_eq!(expr.to_string(), "-(X)");
    }

    #[test]
    fn test_builder_mul() {
        let expr = E::mul(E::var(Var::X), E::var(Var::Y));
        assert_eq!(expr.to_string(), "(X * Y)");
    }

    #[test]
    fn test_builder_inv() {
        let expr = E::inv(E::var(Var::X));
        assert_eq!(expr.to_string(), "1/(X)");
    }

    #[test]
    fn test_builder_func() {
        let expr = E::func("f".into(), E::var(Var::X));
        assert_eq!(expr.to_string(), "f(X)");
    }

    #[test]
    fn test_expr_to_string_complex() {
        let expr1 = E::add(E::constant(Const::Numeric(2)), E::var(Var::X));
        let expr2 = E::mul(E::neg(E::var(Var::Y)), E::inv(E::var(Var::Z)));
        let complex = E::add(
            E::func("f".into(), expr1.clone()),
            E::func("g".into(), expr2.clone()),
        );
        assert_eq!(complex.to_string(), "(f((2 + X)) + g((-(Y) * 1/(Z))))");
    }

    #[test]
    fn test_diff_add_vars() {
        let expr = E::add(E::var(Var::X), E::var(Var::Y));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "(1 + 0)");
    }

    #[test]
    fn test_unpack_inv_inv() {
        let double_inv = E::inv(E::inv(E::var(Var::X)));
        let inner = double_inv.clone().unpack_inv_inv().unwrap();
        assert_eq!(inner.to_string(), "X");
    }

    #[test]
    fn test_unpack_neg_neg() {
        let double_neg = E::neg(E::neg(E::neg(E::neg(E::neg(E::var(Var::Y))))));
        let inner = double_neg.clone().unneg();
        assert_eq!(inner.to_string(), "-(Y)");
    }

    #[test]
    fn test_simplify_double_inv() {
        let double_inv = E::inv(E::inv(E::var(Var::X)));
        let simplified = double_inv.uninv();
        assert_eq!(simplified.to_string(), "X");
    }

    #[test]
    fn test_simplify_double_neg() {
        let double_neg = E::neg(E::neg(E::var(Var::X)));
        let simplified = double_neg.unneg();
        assert_eq!(simplified.to_string(), "X");
    }

    #[test]
    fn test_substitute_named_constant() {
        let expr = E::add(E::constant(Const::Named("a".into())), E::var(Var::X));
        let substituted = expr.substitute("a", E::constant(Const::Numeric(10)));
        assert_eq!(substituted.to_string(), "(10 + X)");
    }

    #[test]
    fn test_substitute_deep() {
        let expr = E::mul(
            E::constant(Const::Named("a".into())),
            E::func("f".into(), E::constant(Const::Named("a".into()))),
        );
        let substituted = expr.substitute("a", E::constant(Const::Numeric(3)));
        assert_eq!(substituted.to_string(), "(3 * f(3))");
    }

    #[test]
    fn test_diff_neg() {
        let expr = E::neg(E::var(Var::X));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "-(1)");
    }

    #[test]
    fn test_diff_mul() {
        let expr = E::mul(E::var(Var::X), E::var(Var::Y));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "((1 * Y) + (X * 0))");
    }

    #[test]
    fn test_diff_inv() {
        let expr = E::inv(E::var(Var::X));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "(-(1/((X * X))) * 1)");
    }

    #[test]
    fn test_diff_const_numeric() {
        let expr = E::constant(Const::Numeric(7));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "0");
    }

    #[test]
    fn test_diff_const_named() {
        let expr = E::constant(Const::Named("a".into()));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "0");
    }

    #[test]
    fn test_diff_func() {
        let expr = E::func("f".into(), E::var(Var::X));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "(f_X(X) * 1)");
    }

    #[test]
    fn test_diff_var_same() {
        let d = E::var(Var::X).diff(Var::X);
        assert_eq!(d.to_string(), "1");
    }

    #[test]
    fn test_diff_var_other() {
        let d = E::var(Var::Y).diff(Var::X);
        assert_eq!(d.to_string(), "0");
    }

    #[test]
    fn test_diff_big_expression() {
        // (((X + -(Y)) * 1/(Z)) + (f((X * Y)) + g(1/(X))))
        let part1 = E::add(E::var(Var::X), E::neg(E::var(Var::Y)));
        let part2 = E::inv(E::var(Var::Z));
        let a = E::mul(part1.clone(), part2.clone());
        let xy = E::mul(E::var(Var::X), E::var(Var::Y));
        let b = E::func("f".into(), xy);
        let inv_x = E::inv(E::var(Var::X));
        let c = E::func("g".into(), inv_x);
        let big = E::add(a.clone(), E::add(b.clone(), c.clone()));

        assert_eq!(
            big.to_string(),
            "(((X + -(Y)) * 1/(Z)) + (f((X * Y)) + g(1/(X))))"
        );

        let d = big.diff(Var::X);
        assert_eq!(
            d.to_string(),
            "((((1 + -(0)) * 1/(Z)) + ((X + -(Y)) * (-(1/((Z * Z))) * 0))) + ((f_X((X * Y)) * ((1 * Y) + (X * 0))) + (g_X(1/(X)) * (-(1/((X * X))) * 1))))"
        );
    }

    #[test]
    fn test_arg_count_zeroary() {
        assert_eq!(E::constant(Const::Numeric(1)).arg_count(), 0);
        assert_eq!(E::var(Var::X).arg_count(), 0);
    }

    #[test]
    fn test_arg_count_unary() {
        assert_eq!(E::neg(E::var(Var::X)).arg_count(), 1);
        assert_eq!(E::inv(E::var(Var::X)).arg_count(), 1);
        assert_eq!(E::func("f".into(), E::var(Var::X)).arg_count(), 1);
    }

    #[test]
    fn test_arg_count_binary() {
        assert_eq!(E::add(E::var(Var::X), E::var(Var::Y)).arg_count(), 2);
        assert_eq!(E::mul(E::var(Var::X), E::var(Var::Z)).arg_count(), 2);
    }
}
