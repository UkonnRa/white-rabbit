use super::{Specification, SpecificationExpression};

/// A minimal test spec for exercising the expression tree.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Color {
    Red,
    Green,
    Blue,
}

impl Specification for Color {}

type ColorExpr = SpecificationExpression<Color>;

fn matches(expr: &ColorExpr, color: &Color) -> bool {
    expr.evaluate(&|leaf| leaf == color)
}

#[test]
fn test_leaf_matches() {
    let spec: ColorExpr = Color::Red.into();
    assert!(matches(&spec, &Color::Red));
    assert!(!matches(&spec, &Color::Blue));
}

#[test]
fn test_from_wraps_in_leaf() {
    let spec: ColorExpr = Color::Green.into();
    assert!(matches(&spec, &Color::Green));
    assert!(!matches(&spec, &Color::Red));
}

#[test]
fn test_bitand_requires_all() {
    // Red AND Green — nothing can be both, so always false for a single color
    let spec = ColorExpr::Leaf(Color::Red) & Color::Green;
    assert!(!matches(&spec, &Color::Red));
    assert!(!matches(&spec, &Color::Green));
    assert!(!matches(&spec, &Color::Blue));
}

#[test]
fn test_bitor_requires_any() {
    let spec = ColorExpr::Leaf(Color::Red) | Color::Blue;
    assert!(matches(&spec, &Color::Red));
    assert!(matches(&spec, &Color::Blue));
    assert!(!matches(&spec, &Color::Green));
}

#[test]
fn test_not_negates() {
    let spec = !ColorExpr::Leaf(Color::Red);
    assert!(!matches(&spec, &Color::Red));
    assert!(matches(&spec, &Color::Green));
    assert!(matches(&spec, &Color::Blue));
}

#[test]
fn test_complex_expression() {
    // (Red OR Green) AND NOT Blue
    let spec = (ColorExpr::Leaf(Color::Red) | Color::Green) & !ColorExpr::Leaf(Color::Blue);

    assert!(matches(&spec, &Color::Red));
    assert!(matches(&spec, &Color::Green));
    assert!(!matches(&spec, &Color::Blue));
}

#[test]
fn test_nested_all_empty_is_vacuously_true() {
    let spec = ColorExpr::All(vec![]);
    assert!(matches(&spec, &Color::Red));
}

#[test]
fn test_nested_any_empty_is_vacuously_false() {
    let spec = ColorExpr::Any(vec![]);
    assert!(!matches(&spec, &Color::Red));
}

#[test]
fn test_double_negation() {
    let spec = !!ColorExpr::Leaf(Color::Red);
    assert!(matches(&spec, &Color::Red));
    assert!(!matches(&spec, &Color::Blue));
}
