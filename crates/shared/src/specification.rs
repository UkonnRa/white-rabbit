use std::fmt::Debug;
use std::ops::{BitAnd, BitOr, Not};

#[cfg(test)]
mod test;

/// Marker trait for domain specification enums (leaf predicates).
pub trait Specification: Send + Sync + Debug {}

/// A composable expression tree wrapping a [`Specification`].
///
/// `Leaf` holds entity-specific predicates, while `All`, `Any`, and `Not`
/// provide logical composition — without adding variants to each entity spec.
#[derive(Debug, Clone)]
pub enum SpecificationExpression<S: Specification> {
    Leaf(S),
    All(Vec<SpecificationExpression<S>>),
    Any(Vec<SpecificationExpression<S>>),
    Not(Box<SpecificationExpression<S>>),
}

impl<S: Specification> Specification for SpecificationExpression<S> {}

impl<S: Specification> SpecificationExpression<S> {
    /// Evaluate the expression tree, delegating leaf cases to a closure.
    /// `evaluate` calls itself recursively, the compiler tries to monomorphize infinitely
    pub fn evaluate(&self, leaf_fn: &dyn Fn(&S) -> bool) -> bool {
        match self {
            Self::Leaf(s) => leaf_fn(s),
            Self::All(specs) => specs.iter().all(|s| s.evaluate(leaf_fn)),
            Self::Any(specs) => specs.iter().any(|s| s.evaluate(leaf_fn)),
            Self::Not(s) => !s.evaluate(leaf_fn),
        }
    }
}

impl<S: Specification, Rhs: Into<SpecificationExpression<S>>> BitAnd<Rhs>
    for SpecificationExpression<S>
{
    type Output = Self;
    fn bitand(self, rhs: Rhs) -> Self {
        Self::All(vec![self, rhs.into()])
    }
}

impl<S: Specification, Rhs: Into<SpecificationExpression<S>>> BitOr<Rhs>
    for SpecificationExpression<S>
{
    type Output = Self;
    fn bitor(self, rhs: Rhs) -> Self {
        Self::Any(vec![self, rhs.into()])
    }
}

impl<S: Specification> Not for SpecificationExpression<S> {
    type Output = Self;
    fn not(self) -> Self {
        Self::Not(Box::new(self))
    }
}

impl<S: Specification> From<S> for SpecificationExpression<S> {
    fn from(spec: S) -> Self {
        Self::Leaf(spec)
    }
}
