use crate::{
    CheckFailure, Dimension, Domain, Scalar, Type,
    checker::{
        CheckResult, Checked, CheckedKind, CheckedNode, ElementwiseFunction, Fold, NumberValue,
        Pending, Reduction, Rounding,
    },
    function::Function,
    lexer::{NumberLiteral, NumberSuffix},
    parser::{BinaryOperator, ComparisonOperator, LogicalOperator, SyntaxNode, UnaryOperator},
};
use std::collections::HashMap;

/// Checks an expression tree against the names in scope, answering a typed
/// node or a pending subtree whose bare literals still await a type.
pub(crate) fn check_node(node: &SyntaxNode, scope: &HashMap<String, Type>) -> CheckResult<Checked> {
    Checker { scope }.node(node)
}

struct Checker<'a> {
    scope: &'a HashMap<String, Type>,
}

impl Checker<'_> {
    fn node(&self, node: &SyntaxNode) -> CheckResult<Checked> {
        match node {
            SyntaxNode::Binary {
                operator,
                left,
                right,
            } => self.binary(*operator, left, right),

            SyntaxNode::Bool(value) => Ok(plain(CheckedKind::Bool(*value), Scalar::Bool)),

            SyntaxNode::Call {
                function,
                arguments,
            } => self.call(*function, arguments),

            SyntaxNode::Comparison {
                operator,
                left,
                right,
            } => self.comparison(*operator, left, right),

            SyntaxNode::Default { name, fallback } => self.default(name, fallback),
            SyntaxNode::Index { source, index } => self.index(source, index),

            SyntaxNode::Logical {
                operator,
                left,
                right,
            } => self.logical(*operator, left, right),

            SyntaxNode::Name(name) => self.name(name),
            SyntaxNode::Number(literal) => number(literal),

            SyntaxNode::StringLiteral(text) => Ok(plain(
                CheckedKind::StringLiteral(text.clone()),
                Scalar::String,
            )),

            SyntaxNode::Swizzle { source, member } => self.swizzle(source, member),
            SyntaxNode::Unary { operator, operand } => self.unary(*operator, operand),
        }
    }

    fn binary(
        &self,
        operator: BinaryOperator,
        left: &SyntaxNode,
        right: &SyntaxNode,
    ) -> CheckResult<Checked> {
        let left = self.node(left)?;
        let right = self.node(right)?;
        let operation = operator.to_string();
        let dimensions = [left.dimension(), right.dimension()];
        let dimension = match operator {
            BinaryOperator::Add | BinaryOperator::Subtract => same(dimensions[0], dimensions[1]),
            BinaryOperator::Divide => right_vec1(dimensions[0], dimensions[1]),
            BinaryOperator::Multiply => either_vec1(dimensions[0], dimensions[1]),
        }
        .ok_or_else(|| mismatch(&operation, &dimensions))?;
        let domain = left.domain().max(right.domain());

        keep(
            &operation,
            vec![left, right],
            domain,
            dimension,
            move |nodes| {
                let [left, right] = fixed(nodes);

                CheckedKind::Binary {
                    operator,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            },
        )
    }

    fn call(&self, function: Function, arguments: &[SyntaxNode]) -> CheckResult<Checked> {
        let operation = function.name();

        if let Some(fold) = Fold::from_function(function) {
            let [argument] = arguments else {
                unreachable!("the parser checks arity");
            };

            return self.fold(fold, operation, argument);
        }

        let checked = arguments
            .iter()
            .map(|argument| self.node(argument))
            .collect::<CheckResult<Vec<_>>>()?;
        let dimensions = checked.iter().map(Checked::dimension).collect::<Vec<_>>();
        let domain = checked
            .iter()
            .map(Checked::domain)
            .max()
            .expect("every function takes an argument");
        let mismatched = || mismatch(operation, &dimensions);

        match function {
            // The constructors.
            Function::R | Function::Rg | Function::Rgb | Function::Rgba => {
                require_each(operation, Dimension::Vec1, &dimensions)?;

                let dimension =
                    Dimension::from_width(dimensions.len()).expect("the parser checks arity");

                f32_call(function, checked, domain, dimension)
            }

            // The elementwise functions over one type.
            Function::Abs
            | Function::Ceil
            | Function::Floor
            | Function::Normalize
            | Function::Round => f32_call(function, checked, domain, dimensions[0]),

            Function::Length => f32_call(function, checked, domain, Dimension::Vec1),

            Function::Dot | Function::Distance => {
                same(dimensions[0], dimensions[1]).ok_or_else(mismatched)?;

                f32_call(function, checked, domain, Dimension::Vec1)
            }

            Function::Cross
            | Function::OklabFromRgb
            | Function::OklchFromRgb
            | Function::RgbFromOklab
            | Function::RgbFromOklch => {
                require_each(operation, Dimension::Vec3, &dimensions)?;

                f32_call(function, checked, domain, Dimension::Vec3)
            }

            Function::Pow => {
                let dimension = right_vec1(dimensions[0], dimensions[1]).ok_or_else(mismatched)?;

                f32_call(function, checked, domain, dimension)
            }

            Function::Mod => {
                let dimension = right_vec1(dimensions[0], dimensions[1]).ok_or_else(mismatched)?;

                keep_call(function, checked, domain, dimension)
            }

            Function::Clamp => {
                let dimension = same(dimensions[1], dimensions[2])
                    .and_then(|bound| bounds(dimensions[0], bound))
                    .ok_or_else(mismatched)?;

                f32_call(function, checked, domain, dimension)
            }

            Function::Smoothstep => {
                let dimension = same(dimensions[0], dimensions[1])
                    .and_then(|bound| bounds(dimensions[2], bound))
                    .ok_or_else(mismatched)?;

                f32_call(function, checked, domain, dimension)
            }

            Function::Lerp => {
                let dimension = same(dimensions[0], dimensions[1])
                    .and_then(|pair| bounds(pair, dimensions[2]))
                    .ok_or_else(mismatched)?;

                f32_call(function, checked, domain, dimension)
            }

            Function::Step => {
                let dimension = bounds(dimensions[1], dimensions[0]).ok_or_else(mismatched)?;

                f32_call(function, checked, domain, dimension)
            }

            Function::Max if checked.len() == 1 => {
                reduce(operation, Reduction::Max, Domain::Plain, only(checked))
            }

            Function::Min if checked.len() == 1 => {
                reduce(operation, Reduction::Min, Domain::Plain, only(checked))
            }

            Function::Max | Function::Min => {
                let dimension = either_vec1(dimensions[0], dimensions[1]).ok_or_else(mismatched)?;

                keep_call(function, checked, domain, dimension)
            }

            Function::Mix => mix(checked, domain),

            // The reductions.
            Function::Avg => reduce(operation, Reduction::Avg, Domain::Plain, only(checked)),
            Function::Sum => reduce(operation, Reduction::Sum, Domain::Plain, only(checked)),
            Function::FaceAvg => reduce(operation, Reduction::Avg, Domain::Face, only(checked)),
            Function::FaceMax => reduce(operation, Reduction::Max, Domain::Face, only(checked)),
            Function::FaceMin => reduce(operation, Reduction::Min, Domain::Face, only(checked)),
            Function::FaceSum => reduce(operation, Reduction::Sum, Domain::Face, only(checked)),
            Function::VoxelAvg => reduce(operation, Reduction::Avg, Domain::Voxel, only(checked)),
            Function::VoxelMax => reduce(operation, Reduction::Max, Domain::Voxel, only(checked)),
            Function::VoxelMin => reduce(operation, Reduction::Min, Domain::Voxel, only(checked)),
            Function::VoxelSum => reduce(operation, Reduction::Sum, Domain::Voxel, only(checked)),
            Function::SwatchAvg => reduce(operation, Reduction::Avg, Domain::Swatch, only(checked)),
            Function::SwatchMax => reduce(operation, Reduction::Max, Domain::Swatch, only(checked)),
            Function::SwatchMin => reduce(operation, Reduction::Min, Domain::Swatch, only(checked)),
            Function::SwatchSum => reduce(operation, Reduction::Sum, Domain::Swatch, only(checked)),

            // The climbs.
            Function::Swatch => climb(operation, Domain::Swatch, only(checked)),
            Function::Voxel => climb(operation, Domain::Voxel, only(checked)),
            Function::Face => climb(operation, Domain::Face, only(checked)),
            Function::Corner => climb(operation, Domain::Corner, only(checked)),

            // The conversions.
            Function::F32 => convert(operation, Scalar::F32, None, only(checked)),
            Function::U8 => convert(operation, Scalar::U8, None, only(checked)),
            Function::U16 => convert(operation, Scalar::U16, None, only(checked)),
            Function::U32 => convert(operation, Scalar::U32, None, only(checked)),
            Function::CeilU8 => convert(operation, Scalar::U8, Some(Rounding::Ceil), only(checked)),
            Function::CeilU16 => {
                convert(operation, Scalar::U16, Some(Rounding::Ceil), only(checked))
            }
            Function::CeilU32 => {
                convert(operation, Scalar::U32, Some(Rounding::Ceil), only(checked))
            }
            Function::FloorU8 => {
                convert(operation, Scalar::U8, Some(Rounding::Floor), only(checked))
            }
            Function::FloorU16 => {
                convert(operation, Scalar::U16, Some(Rounding::Floor), only(checked))
            }
            Function::FloorU32 => {
                convert(operation, Scalar::U32, Some(Rounding::Floor), only(checked))
            }
            Function::RoundU8 => {
                convert(operation, Scalar::U8, Some(Rounding::Round), only(checked))
            }
            Function::RoundU16 => {
                convert(operation, Scalar::U16, Some(Rounding::Round), only(checked))
            }
            Function::RoundU32 => {
                convert(operation, Scalar::U32, Some(Rounding::Round), only(checked))
            }

            Function::All | Function::Any | Function::Default => {
                unreachable!("the folds return above and the parser lowers default to its own node")
            }
        }
    }

    fn comparison(
        &self,
        operator: ComparisonOperator,
        left: &SyntaxNode,
        right: &SyntaxNode,
    ) -> CheckResult<Checked> {
        let left = self.node(left)?;
        let right = self.node(right)?;
        let domain = left.domain().max(right.domain());
        let (left, right) = compare(operator, left, right, false)?;

        Ok(bool_node(
            CheckedKind::Comparison {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            domain,
        ))
    }

    fn default(&self, name: &str, fallback: &SyntaxNode) -> CheckResult<Checked> {
        let operation = "default";
        let fallback = self.node(fallback)?;

        let Some(existing) = self.scope.get(name).copied() else {
            let fallback = fallback.resolve(Scalar::F32)?;
            let output = fallback.output;

            return Ok(Checked::Typed(CheckedNode {
                kind: CheckedKind::Default {
                    name: name.to_owned(),
                    bound: None,
                    fallback: Box::new(fallback),
                },
                output,
            }));
        };

        let dimensions = [existing.dimension, fallback.dimension()];

        if dimensions[0] != dimensions[1] {
            return Err(mismatch(operation, &dimensions));
        }

        let fallback = match fallback.scalar() {
            None if existing.scalar.is_numeric() => fallback.resolve(existing.scalar)?,
            Some(scalar) if scalar == existing.scalar => typed(fallback),
            found => {
                return Err(CheckFailure::MixedScalars {
                    operation: operation.to_owned(),
                    found: vec![existing.scalar, found.unwrap_or(Scalar::F32)],
                });
            }
        };
        let output = Type {
            domain: existing.domain.max(fallback.output.domain),
            dimension: existing.dimension,
            scalar: existing.scalar,
        };

        Ok(Checked::Typed(CheckedNode {
            kind: CheckedKind::Default {
                name: name.to_owned(),
                bound: Some(existing),
                fallback: Box::new(fallback),
            },
            output,
        }))
    }

    fn fold(&self, fold: Fold, operation: &str, argument: &SyntaxNode) -> CheckResult<Checked> {
        let SyntaxNode::Comparison {
            operator,
            left,
            right,
        } = argument
        else {
            return Err(CheckFailure::FoldNeedsComparison {
                operation: operation.to_owned(),
            });
        };
        let left = self.node(left)?;
        let right = self.node(right)?;
        let domain = left.domain().max(right.domain());
        let (left, right) = compare(*operator, left, right, true)?;

        Ok(bool_node(
            CheckedKind::Fold {
                fold,
                operator: *operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            domain,
        ))
    }

    fn index(&self, source: &SyntaxNode, index: &SyntaxNode) -> CheckResult<Checked> {
        let operation = "[]";
        let source = self.node(source)?;
        let index = self.node(index)?;

        if !source.domain().is_array() {
            return Err(CheckFailure::RequiresArray {
                operation: operation.to_owned(),
            });
        }

        if index.domain().is_array() {
            return Err(CheckFailure::RequiresPlain {
                operation: operation.to_owned(),
                found: index.domain(),
            });
        }

        if index.dimension() != Dimension::Vec1 {
            return Err(CheckFailure::RequiresDimension {
                operation: operation.to_owned(),
                expected: Dimension::Vec1,
                found: index.dimension(),
            });
        }

        let index = match index.scalar() {
            None => index.resolve(Scalar::U32)?,
            Some(scalar) if scalar.is_unsigned() => typed(index),
            Some(found) => return Err(CheckFailure::IndexScalar { found }),
        };
        let dimension = source.dimension();

        Ok(carry(source, Domain::Plain, dimension, move |source| {
            CheckedKind::Index {
                source: Box::new(source),
                index: Box::new(index),
            }
        }))
    }

    fn logical(
        &self,
        operator: LogicalOperator,
        left: &SyntaxNode,
        right: &SyntaxNode,
    ) -> CheckResult<Checked> {
        let operation = operator.to_string();
        let left = bool_operand(&operation, self.node(left)?)?;
        let right = bool_operand(&operation, self.node(right)?)?;
        let domain = left.output.domain.max(right.output.domain);

        Ok(bool_node(
            CheckedKind::Logical {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            domain,
        ))
    }

    fn name(&self, name: &str) -> CheckResult<Checked> {
        let output = *self
            .scope
            .get(name)
            .ok_or_else(|| CheckFailure::UnknownName {
                name: name.to_owned(),
            })?;

        Ok(Checked::Typed(CheckedNode {
            kind: CheckedKind::Name(name.to_owned()),
            output,
        }))
    }

    fn swizzle(&self, source: &SyntaxNode, member: &str) -> CheckResult<Checked> {
        let source = self.node(source)?;
        let operation = format!(".{member}");
        let components = swizzle_components(member, source.dimension())?;
        let dimension = Dimension::from_width(components.len())
            .expect("a swizzle holds one to four components");
        let domain = source.domain();

        keep(&operation, vec![source], domain, dimension, move |nodes| {
            CheckedKind::Swizzle {
                source: Box::new(only(nodes)),
                components,
            }
        })
    }

    fn unary(&self, operator: UnaryOperator, operand: &SyntaxNode) -> CheckResult<Checked> {
        let operand = self.node(operand)?;
        let operation = operator.to_string();
        let domain = operand.domain();
        let dimension = operand.dimension();
        let (operand, scalar) = match operator {
            UnaryOperator::Negate => (only(f32_only(&operation, vec![operand])?), Scalar::F32),
            UnaryOperator::Not => (bool_operand(&operation, operand)?, Scalar::Bool),
        };

        Ok(Checked::Typed(CheckedNode {
            kind: CheckedKind::Unary {
                operator,
                operand: Box::new(operand),
            },
            output: Type {
                domain,
                dimension,
                scalar,
            },
        }))
    }
}

/// The numeric operands of one operation, unified under one type.
enum Unified {
    /// No operand was typed; every one still awaits the type.
    Pending(Vec<Pending>),

    /// The operands settled to one type.
    Settled {
        scalar: Scalar,
        nodes: Vec<CheckedNode>,
    },
}

/// Unifies numeric operands under one type, resolving pending literals to it.
fn unify(operation: &str, operands: Vec<Checked>) -> CheckResult<Unified> {
    let mut scalars = Vec::new();

    for operand in &operands {
        if let Some(scalar) = operand.scalar() {
            if !scalar.is_numeric() {
                return Err(CheckFailure::NonNumericOperand {
                    operation: operation.to_owned(),
                    found: scalar,
                });
            }

            if !scalars.contains(&scalar) {
                scalars.push(scalar);
            }
        }
    }

    match scalars[..] {
        [] => {
            let pending = operands
                .into_iter()
                .map(Checked::into_pending)
                .collect::<Option<Vec<_>>>()
                .expect("every operand is pending when none is typed");

            Ok(Unified::Pending(pending))
        }

        [scalar] => {
            let nodes = operands
                .into_iter()
                .map(|operand| operand.resolve(scalar))
                .collect::<CheckResult<Vec<_>>>()?;

            Ok(Unified::Settled { scalar, nodes })
        }

        _ => Err(CheckFailure::MixedScalars {
            operation: operation.to_owned(),
            found: scalars,
        }),
    }
}

/// Builds a type-keeping node over numeric operands, staying pending while
/// every operand is.
fn keep(
    operation: &str,
    operands: Vec<Checked>,
    domain: Domain,
    dimension: Dimension,
    build: impl FnOnce(Vec<CheckedNode>) -> CheckedKind + 'static,
) -> CheckResult<Checked> {
    match unify(operation, operands)? {
        Unified::Pending(pending) => Ok(Checked::Pending(Pending {
            domain,
            dimension,
            build: Box::new(move |scalar| {
                let nodes = pending
                    .into_iter()
                    .map(|operand| operand.resolve(scalar))
                    .collect::<CheckResult<Vec<_>>>()?;

                Ok(CheckedNode {
                    kind: build(nodes),
                    output: Type {
                        domain,
                        dimension,
                        scalar,
                    },
                })
            }),
        })),

        Unified::Settled { scalar, nodes } => Ok(Checked::Typed(CheckedNode {
            kind: build(nodes),
            output: Type {
                domain,
                dimension,
                scalar,
            },
        })),
    }
}

/// Builds a node over one operand of any type, staying pending while the
/// operand is.
fn carry(
    operand: Checked,
    domain: Domain,
    dimension: Dimension,
    build: impl FnOnce(CheckedNode) -> CheckedKind + 'static,
) -> Checked {
    match operand {
        Checked::Pending(pending) => Checked::Pending(Pending {
            domain,
            dimension,
            build: Box::new(move |scalar| {
                let node = pending.resolve(scalar)?;

                Ok(CheckedNode {
                    kind: build(node),
                    output: Type {
                        domain,
                        dimension,
                        scalar,
                    },
                })
            }),
        }),

        Checked::Typed(node) => {
            let scalar = node.output.scalar;

            Checked::Typed(CheckedNode {
                kind: build(node),
                output: Type {
                    domain,
                    dimension,
                    scalar,
                },
            })
        }
    }
}

/// Unifies numeric operands, erroring where nothing typed them.
fn settled(operation: &str, operands: Vec<Checked>) -> CheckResult<(Scalar, Vec<CheckedNode>)> {
    match unify(operation, operands)? {
        Unified::Pending(_) => Err(CheckFailure::UntypedLiteral),
        Unified::Settled { scalar, nodes } => Ok((scalar, nodes)),
    }
}

/// Takes numeric operands as `f32` alone, typing pending literals `f32`.
fn f32_only(operation: &str, operands: Vec<Checked>) -> CheckResult<Vec<CheckedNode>> {
    operands
        .into_iter()
        .map(|operand| match operand.scalar() {
            None | Some(Scalar::F32) => operand.resolve(Scalar::F32),
            Some(found) if found.is_numeric() => Err(CheckFailure::RequiresF32 {
                operation: operation.to_owned(),
                found,
            }),
            Some(found) => Err(CheckFailure::NonNumericOperand {
                operation: operation.to_owned(),
                found,
            }),
        })
        .collect()
}

/// Takes an operand as a bool.
fn bool_operand(operation: &str, operand: Checked) -> CheckResult<CheckedNode> {
    match operand.scalar() {
        None => Err(CheckFailure::UntypedLiteral),
        Some(Scalar::Bool) => Ok(typed(operand)),
        Some(found) => Err(CheckFailure::NonBoolOperand {
            operation: operation.to_owned(),
            found,
        }),
    }
}

/// Checks a comparison's sides: two strings under `==` or `!=`, or two
/// numbers at vec1. Wider sides broadcast inside a fold alone.
fn compare(
    operator: ComparisonOperator,
    left: Checked,
    right: Checked,
    folded: bool,
) -> CheckResult<(CheckedNode, CheckedNode)> {
    let operation = operator.to_string();

    if left.scalar() == Some(Scalar::String) && right.scalar() == Some(Scalar::String) {
        if !operator.is_equality() {
            return Err(CheckFailure::StringOrder { operation });
        }

        return Ok((typed(left), typed(right)));
    }

    let dimensions = [left.dimension(), right.dimension()];

    if folded {
        either_vec1(dimensions[0], dimensions[1])
            .ok_or_else(|| mismatch(&operation, &dimensions))?;
    } else if let Some(found) = dimensions
        .into_iter()
        .find(|dimension| *dimension != Dimension::Vec1)
    {
        return Err(CheckFailure::WideComparison { operation, found });
    }

    let (_, nodes) = settled(&operation, vec![left, right])?;
    let [left, right] = fixed(nodes);

    Ok((left, right))
}

/// Checks `mix`: a bool choosing between two branches of one type. A
/// numeric pair keeps the type it is given.
fn mix(arguments: Vec<Checked>, domain: Domain) -> CheckResult<Checked> {
    let operation = "mix";
    let [first, second, chooser] = fixed(arguments);
    let chooser = bool_operand(operation, chooser)?;
    let dimensions = [first.dimension(), second.dimension()];
    let dimension =
        same(dimensions[0], dimensions[1]).ok_or_else(|| mismatch(operation, &dimensions))?;

    match (first.scalar(), second.scalar()) {
        (Some(scalar), Some(other)) if scalar == other && !scalar.is_numeric() => {
            Ok(Checked::Typed(CheckedNode {
                kind: CheckedKind::Mix {
                    first: Box::new(typed(first)),
                    second: Box::new(typed(second)),
                    chooser: Box::new(chooser),
                },
                output: Type {
                    domain,
                    dimension,
                    scalar,
                },
            }))
        }

        (Some(scalar), Some(other)) if scalar != other => Err(CheckFailure::MixedScalars {
            operation: operation.to_owned(),
            found: vec![scalar, other],
        }),

        _ => keep(
            operation,
            vec![first, second],
            domain,
            dimension,
            move |nodes| {
                let [first, second] = fixed(nodes);

                CheckedKind::Mix {
                    first: Box::new(first),
                    second: Box::new(second),
                    chooser: Box::new(chooser),
                }
            },
        ),
    }
}

/// Checks a reduction. The source sits above the destination. `avg`
/// answers `f32`; the others keep the type.
fn reduce(
    operation: &str,
    reduction: Reduction,
    target: Domain,
    operand: Checked,
) -> CheckResult<Checked> {
    let source = operand.domain();

    if source <= target {
        return Err(if target == Domain::Plain {
            CheckFailure::RequiresArray {
                operation: operation.to_owned(),
            }
        } else {
            CheckFailure::ReductionSource {
                operation: operation.to_owned(),
                found: source,
            }
        });
    }

    let dimension = operand.dimension();

    match reduction {
        Reduction::Avg => {
            let (_, nodes) = settled(operation, vec![operand])?;

            Ok(Checked::Typed(CheckedNode {
                kind: CheckedKind::Reduce {
                    reduction,
                    target,
                    operand: Box::new(only(nodes)),
                },
                output: Type {
                    domain: target,
                    dimension,
                    scalar: Scalar::F32,
                },
            }))
        }

        Reduction::Max | Reduction::Min | Reduction::Sum => {
            keep(operation, vec![operand], target, dimension, move |nodes| {
                CheckedKind::Reduce {
                    reduction,
                    target,
                    operand: Box::new(only(nodes)),
                }
            })
        }
    }
}

/// Checks a climb: the operand sits at or below the named domain.
fn climb(operation: &str, target: Domain, operand: Checked) -> CheckResult<Checked> {
    let found = operand.domain();

    if found > target {
        return Err(CheckFailure::StepDown {
            operation: operation.to_owned(),
            found,
        });
    }

    let dimension = operand.dimension();

    Ok(carry(operand, target, dimension, move |operand| {
        CheckedKind::Climb {
            target,
            operand: Box::new(operand),
        }
    }))
}

/// Checks a conversion. The rounding forms take `f32`; the exact forms
/// take any settled number.
fn convert(
    operation: &str,
    target: Scalar,
    rounding: Option<Rounding>,
    operand: Checked,
) -> CheckResult<Checked> {
    let domain = operand.domain();
    let dimension = operand.dimension();
    let operand = match rounding {
        None => only(settled(operation, vec![operand])?.1),
        Some(_) => only(f32_only(operation, vec![operand])?),
    };

    Ok(Checked::Typed(CheckedNode {
        kind: CheckedKind::Convert {
            target,
            rounding,
            operand: Box::new(operand),
        },
        output: Type {
            domain,
            dimension,
            scalar: target,
        },
    }))
}

/// Builds an elementwise call over `f32` arguments.
fn f32_call(
    function: Function,
    arguments: Vec<Checked>,
    domain: Domain,
    dimension: Dimension,
) -> CheckResult<Checked> {
    let arguments = f32_only(function.name(), arguments)?;

    Ok(Checked::Typed(CheckedNode {
        kind: CheckedKind::Call {
            function: elementwise(function),
            arguments,
        },
        output: Type {
            domain,
            dimension,
            scalar: Scalar::F32,
        },
    }))
}

/// Builds an elementwise call that keeps its arguments' numeric type.
fn keep_call(
    function: Function,
    arguments: Vec<Checked>,
    domain: Domain,
    dimension: Dimension,
) -> CheckResult<Checked> {
    let elementwise = elementwise(function);

    keep(
        function.name(),
        arguments,
        domain,
        dimension,
        move |arguments| CheckedKind::Call {
            function: elementwise,
            arguments,
        },
    )
}

/// Checks a literal. A decimal point or suffix fixes its type; a bare
/// whole number waits for its context.
fn number(literal: &NumberLiteral) -> CheckResult<Checked> {
    let text = literal.text.clone();
    let pinned = match literal.suffix {
        Some(NumberSuffix::F32) => Some(Scalar::F32),
        Some(NumberSuffix::U8) => Some(Scalar::U8),
        Some(NumberSuffix::U16) => Some(Scalar::U16),
        Some(NumberSuffix::U32) => Some(Scalar::U32),
        None if literal.fraction => Some(Scalar::F32),
        None => None,
    };

    match pinned {
        None => Ok(Checked::Pending(Pending {
            domain: Domain::Plain,
            dimension: Dimension::Vec1,
            build: Box::new(move |scalar| number_node(&text, scalar)),
        })),
        Some(scalar) => Ok(Checked::Typed(number_node(&text, scalar)?)),
    }
}

/// Builds a literal node of the given type.
fn number_node(text: &str, scalar: Scalar) -> CheckResult<CheckedNode> {
    Ok(CheckedNode {
        kind: CheckedKind::Number(NumberValue::parse(text, scalar)?),
        output: Type {
            domain: Domain::Plain,
            dimension: Dimension::Vec1,
            scalar,
        },
    })
}

/// Reads a swizzle member as component positions over one alphabet, with
/// every component present in the source.
fn swizzle_components(member: &str, source: Dimension) -> CheckResult<Vec<usize>> {
    let mut alphabet = None;
    let mut components = Vec::new();

    for character in member.chars() {
        let (found, position) = match ("rgba".find(character), "xyzw".find(character)) {
            (Some(position), _) => ("rgba", position),
            (None, Some(position)) => ("xyzw", position),
            (None, None) => {
                return Err(CheckFailure::SwizzleCharacter {
                    member: member.to_owned(),
                    character,
                });
            }
        };

        if alphabet.is_some_and(|alphabet| alphabet != found) {
            return Err(CheckFailure::SwizzleAlphabets {
                member: member.to_owned(),
            });
        }

        alphabet = Some(found);

        if position >= source.width() {
            return Err(CheckFailure::SwizzleComponent {
                member: member.to_owned(),
                character,
                found: source,
            });
        }

        components.push(position);
    }

    if !(1..=4).contains(&components.len()) {
        return Err(CheckFailure::SwizzleLength {
            member: member.to_owned(),
        });
    }

    Ok(components)
}

/// The elementwise function a call names.
fn elementwise(function: Function) -> ElementwiseFunction {
    ElementwiseFunction::from_function(function).expect("the call is elementwise")
}

/// A plain vec1 node of a fixed type.
fn plain(kind: CheckedKind, scalar: Scalar) -> Checked {
    Checked::Typed(CheckedNode {
        kind,
        output: Type {
            domain: Domain::Plain,
            dimension: Dimension::Vec1,
            scalar,
        },
    })
}

/// A bool node over the given domain.
fn bool_node(kind: CheckedKind, domain: Domain) -> Checked {
    Checked::Typed(CheckedNode {
        kind,
        output: Type {
            domain,
            dimension: Dimension::Vec1,
            scalar: Scalar::Bool,
        },
    })
}

/// The typed node an operand with a known scalar holds.
fn typed(operand: Checked) -> CheckedNode {
    operand
        .into_typed()
        .expect("an operand with a known scalar is typed")
}

/// The operands as a fixed-arity array.
fn fixed<T, const N: usize>(items: Vec<T>) -> [T; N] {
    items.try_into().ok().expect("the parser checks arity")
}

/// The one operand.
fn only<T>(items: Vec<T>) -> T {
    let [item] = fixed(items);

    item
}

/// Equal dimensions.
fn same(left: Dimension, right: Dimension) -> Option<Dimension> {
    (left == right).then_some(left)
}

/// Equal dimensions, or a vec1 on either side broadcasting to the other.
fn either_vec1(left: Dimension, right: Dimension) -> Option<Dimension> {
    (left == right || left == Dimension::Vec1 || right == Dimension::Vec1)
        .then_some(left.max(right))
}

/// Equal dimensions, or a vec1 on the right broadcasting across the left.
fn right_vec1(left: Dimension, right: Dimension) -> Option<Dimension> {
    (left == right || right == Dimension::Vec1).then_some(left)
}

/// The value's dimension, where the bound equals it or is a vec1.
fn bounds(value: Dimension, bound: Dimension) -> Option<Dimension> {
    (bound == value || bound == Dimension::Vec1).then_some(value)
}

/// Checks that every dimension is the one the operation takes.
fn require_each(operation: &str, expected: Dimension, dimensions: &[Dimension]) -> CheckResult<()> {
    match dimensions.iter().find(|dimension| **dimension != expected) {
        None => Ok(()),
        Some(&found) => Err(CheckFailure::RequiresDimension {
            operation: operation.to_owned(),
            expected,
            found,
        }),
    }
}

/// The dimension failure for an operation over the given dimensions.
fn mismatch(operation: &str, found: &[Dimension]) -> CheckFailure {
    CheckFailure::DimensionMismatch {
        operation: operation.to_owned(),
        found: found.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        CheckFailure, CheckedProgram, Dimension, Domain, Error, Scalar, Type, TypeEnvironment,
        check, check_expression, parse, parse_expression,
    };

    fn environment() -> TypeEnvironment {
        let names = [
            ("color", Domain::Swatch, Dimension::Vec4, Scalar::F32),
            ("rough", Domain::Swatch, Dimension::Vec1, Scalar::F32),
            ("tag", Domain::Swatch, Dimension::Vec1, Scalar::String),
            ("glowing", Domain::Swatch, Dimension::Vec1, Scalar::Bool),
            ("count", Domain::Swatch, Dimension::Vec1, Scalar::U32),
            ("position", Domain::Voxel, Dimension::Vec3, Scalar::U32),
            ("occlusion", Domain::Corner, Dimension::Vec1, Scalar::F32),
            ("faceValue", Domain::Face, Dimension::Vec1, Scalar::F32),
            ("plain3", Domain::Plain, Dimension::Vec3, Scalar::F32),
            ("plainCount", Domain::Plain, Dimension::Vec1, Scalar::U32),
            ("wide", Domain::Voxel, Dimension::Vec2, Scalar::U8),
        ];

        TypeEnvironment {
            types: names
                .into_iter()
                .map(|(name, domain, dimension, scalar)| {
                    (name.to_owned(), ty(domain, dimension, scalar))
                })
                .collect(),
        }
    }

    fn scope() -> CheckedProgram {
        check(parse("").unwrap(), &environment()).unwrap()
    }

    fn ty(domain: Domain, dimension: Dimension, scalar: Scalar) -> Type {
        Type {
            domain,
            dimension,
            scalar,
        }
    }

    fn typed(text: &str) -> Type {
        match check_expression(&parse_expression(text).unwrap(), &scope()) {
            Ok(checked) => checked.to_type(),
            Err(error) => panic!("{text} failed: {error}"),
        }
    }

    fn failure(text: &str) -> CheckFailure {
        match check_expression(&parse_expression(text).unwrap(), &scope()) {
            Err(Error::Check {
                binding: None,
                failure,
            }) => failure,
            other => panic!("{text} gave {other:?}"),
        }
    }

    fn rendered(text: &str) -> String {
        check_expression(&parse_expression(text).unwrap(), &scope())
            .unwrap()
            .root
            .render()
    }

    fn dimensions(operation: &str, found: &[Dimension]) -> CheckFailure {
        CheckFailure::DimensionMismatch {
            operation: operation.to_owned(),
            found: found.to_vec(),
        }
    }

    fn requires_f32(operation: &str, found: Scalar) -> CheckFailure {
        CheckFailure::RequiresF32 {
            operation: operation.to_owned(),
            found,
        }
    }

    fn non_numeric(operation: &str, found: Scalar) -> CheckFailure {
        CheckFailure::NonNumericOperand {
            operation: operation.to_owned(),
            found,
        }
    }

    fn non_bool(operation: &str, found: Scalar) -> CheckFailure {
        CheckFailure::NonBoolOperand {
            operation: operation.to_owned(),
            found,
        }
    }

    fn mixed(operation: &str, found: &[Scalar]) -> CheckFailure {
        CheckFailure::MixedScalars {
            operation: operation.to_owned(),
            found: found.to_vec(),
        }
    }

    // Literals.

    #[test]
    fn a_decimal_point_or_suffix_fixes_a_literal() {
        assert_eq!(rendered("0.5"), "0.5f32");
        assert_eq!(rendered(".5"), "0.5f32");
        assert_eq!(rendered("2."), "2f32");
        assert_eq!(rendered("2f32"), "2f32");
        assert_eq!(rendered("2u8"), "2u8");
        assert_eq!(rendered("2u16"), "2u16");
        assert_eq!(rendered("2u32"), "2u32");
        assert_eq!(typed("2u8"), ty(Domain::Plain, Dimension::Vec1, Scalar::U8));
    }

    #[test]
    fn a_bare_literal_takes_the_type_beside_it() {
        assert_eq!(rendered("count * 2"), "(* `count` 2u32)");
        assert_eq!(rendered("2 * count"), "(* 2u32 `count`)");
        assert_eq!(rendered("rough + 1"), "(+ `rough` 1f32)");
        assert_eq!(
            rendered("position.y * 2 + 1"),
            "(+ (* (. `position` 1) 2u32) 1u32)"
        );
        assert_eq!(rendered("wide.x - 1"), "(- (. `wide` 0) 1u8)");
        assert_eq!(
            rendered("mod(position.y, 2)"),
            "(mod (. `position` 1) 2u32)"
        );
        assert_eq!(rendered("rough < 1"), "(< `rough` 1f32)");
        assert_eq!(rendered("count == 2"), "(== `count` 2u32)");
    }

    #[test]
    fn a_literal_nothing_types_errors() {
        for text in [
            "1",
            "1 + 2",
            "(1 + 2) * 3",
            "f32(1)",
            "u8(1)",
            "face(1)",
            "1 < 2",
            "avg(face(1))",
            "max(face(1))",
            "2.rr",
            "max(1, 2)",
            "mod(4, 3)",
            "face(1)[0]",
            "mix(0, 1, glowing)",
        ] {
            assert_eq!(failure(text), CheckFailure::UntypedLiteral, "{text}");
        }
    }

    #[test]
    fn a_literal_out_of_range_errors() {
        assert_eq!(
            failure("count * 5000000000"),
            CheckFailure::LiteralOutOfRange {
                text: "5000000000".to_owned(),
                scalar: Scalar::U32
            }
        );
        assert_eq!(
            failure("wide.x + 300"),
            CheckFailure::LiteralOutOfRange {
                text: "300".to_owned(),
                scalar: Scalar::U8
            }
        );
        assert_eq!(
            failure("300u8"),
            CheckFailure::LiteralOutOfRange {
                text: "300".to_owned(),
                scalar: Scalar::U8
            }
        );
    }

    #[test]
    fn f32_alone_functions_type_their_literals() {
        assert_eq!(rendered("rgb(1, 1, 1)"), "(rgb 1f32 1f32 1f32)");
        assert_eq!(rendered("pow(rough, 2)"), "(pow `rough` 2f32)");
        assert_eq!(
            rendered("lerp(0.8, 1, rough)"),
            "(lerp 0.8f32 1f32 `rough`)"
        );
        assert_eq!(rendered("clamp(rough, 0, 1)"), "(clamp `rough` 0f32 1f32)");
        assert_eq!(
            rendered("mix(0f32, 1, glowing)"),
            "(mix 0f32 1f32 `glowing`)"
        );
        assert_eq!(rendered("-1"), "(- 1f32)");
        assert_eq!(
            rendered("round_u8(rough * 255)"),
            "(round_u8 (* `rough` 255f32))"
        );
    }

    #[test]
    fn an_index_literal_reads_as_u32() {
        assert_eq!(rendered("color[0]"), "([] `color` 0u32)");
        assert_eq!(rendered("tag[1]"), "([] `tag` 1u32)");
    }

    #[test]
    fn type_keeping_operations_carry_a_pending_literal_to_its_context() {
        assert_eq!(
            rendered("swatchSum(face(1)) * count"),
            "(* (swatchSum (face 1u32)) `count`)"
        );
        assert_eq!(rendered("max(2, 3) + count"), "(+ (max 2u32 3u32) `count`)");
        assert_eq!(rendered("mod(4, 3) * rough"), "(* (mod 4f32 3f32) `rough`)");
        assert_eq!(
            rendered("2.rr + rg(1, 2)"),
            "(+ (. 2f32 00) (rg 1f32 2f32))"
        );
        assert_eq!(rendered("(1 + 2) * wide"), "(* (+ 1u8 2u8) `wide`)");
        assert_eq!(
            rendered("face(2)[0] * count"),
            "(* ([] (face 2u32) 0u32) `count`)"
        );
    }

    // Numbers.

    #[test]
    fn numeric_types_never_mix() {
        assert_eq!(
            failure("position.y * 0.5"),
            mixed("*", &[Scalar::U32, Scalar::F32])
        );
        assert_eq!(
            failure("count + rough"),
            mixed("+", &[Scalar::U32, Scalar::F32])
        );
        assert_eq!(
            failure("count < rough"),
            mixed("<", &[Scalar::U32, Scalar::F32])
        );
        assert_eq!(
            failure("max(count, rough)"),
            mixed("max", &[Scalar::U32, Scalar::F32])
        );
        assert_eq!(
            failure("wide.x + count"),
            mixed("+", &[Scalar::U8, Scalar::U32])
        );
        assert_eq!(
            typed("f32(position.y) * 0.5"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::F32)
        );
    }

    #[test]
    fn the_type_keeping_operations_take_any_numeric_type() {
        assert_eq!(
            typed("mod(count, 2)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("min(count, 2)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("max(count)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("sum(count)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("count / 2"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("wide * wide"),
            ty(Domain::Voxel, Dimension::Vec2, Scalar::U8)
        );
        assert_eq!(
            typed("position.xy"),
            ty(Domain::Voxel, Dimension::Vec2, Scalar::U32)
        );
        assert_eq!(
            typed("swatchMax(position)"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::U32)
        );
        assert_eq!(
            typed("voxelSum(face(count))"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::U32)
        );
    }

    #[test]
    fn every_average_returns_f32() {
        assert_eq!(
            typed("avg(count)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("swatchAvg(position)"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("voxelAvg(face(wide))"),
            ty(Domain::Voxel, Dimension::Vec2, Scalar::F32)
        );
    }

    #[test]
    fn the_f32_alone_functions_reject_unsigned() {
        assert_eq!(failure("pow(count, 2)"), requires_f32("pow", Scalar::U32));
        assert_eq!(failure("abs(count)"), requires_f32("abs", Scalar::U32));
        assert_eq!(failure("floor(wide)"), requires_f32("floor", Scalar::U8));
        assert_eq!(failure("ceil(count)"), requires_f32("ceil", Scalar::U32));
        assert_eq!(failure("round(count)"), requires_f32("round", Scalar::U32));
        assert_eq!(
            failure("lerp(count, count, 0.5)"),
            requires_f32("lerp", Scalar::U32)
        );
        assert_eq!(failure("-count"), requires_f32("-", Scalar::U32));
        assert_eq!(
            failure("length(position)"),
            requires_f32("length", Scalar::U32)
        );
        assert_eq!(
            failure("clamp(count, 0, 1)"),
            requires_f32("clamp", Scalar::U32)
        );
        assert_eq!(
            failure("step(0.5, count)"),
            requires_f32("step", Scalar::U32)
        );
        assert_eq!(
            failure("rgb(count, 1, 1)"),
            requires_f32("rgb", Scalar::U32)
        );
        assert_eq!(
            failure("smoothstep(0, 1, count)"),
            requires_f32("smoothstep", Scalar::U32)
        );
    }

    #[test]
    fn conversions_change_the_type() {
        assert_eq!(
            typed("f32(count)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("f32(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("u8(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U8)
        );
        assert_eq!(
            typed("u16(count)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U16)
        );
        assert_eq!(
            typed("u32(wide)"),
            ty(Domain::Voxel, Dimension::Vec2, Scalar::U32)
        );
        assert_eq!(
            typed("round_u8(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U8)
        );
        assert_eq!(
            typed("ceil_u16(color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::U16)
        );
        assert_eq!(
            typed("floor_u32(occlusion)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("u8(swatchSum(face(1u32)))"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U8)
        );
        assert_eq!(
            failure("round_u8(count)"),
            requires_f32("round_u8", Scalar::U32)
        );
        assert_eq!(
            failure("ceil_u32(wide)"),
            requires_f32("ceil_u32", Scalar::U8)
        );
        assert_eq!(
            failure("floor_u16(count)"),
            requires_f32("floor_u16", Scalar::U32)
        );
        assert_eq!(failure("f32(glowing)"), non_numeric("f32", Scalar::Bool));
        assert_eq!(failure("u32(tag)"), non_numeric("u32", Scalar::String));
    }

    #[test]
    fn arithmetic_pairs_dimensions_under_each_operators_rule() {
        assert_eq!(
            typed("color + color"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("color - color"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("color * rough"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("rough * color"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("color / rough"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("color * color"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            failure("color + color.rgb"),
            dimensions("+", &[Dimension::Vec4, Dimension::Vec3])
        );
        assert_eq!(
            failure("color - rough"),
            dimensions("-", &[Dimension::Vec4, Dimension::Vec1])
        );
        assert_eq!(
            failure("rough / color"),
            dimensions("/", &[Dimension::Vec1, Dimension::Vec4])
        );
        assert_eq!(
            failure("color * color.rg"),
            dimensions("*", &[Dimension::Vec4, Dimension::Vec2])
        );
    }

    #[test]
    fn arithmetic_rejects_bools_and_strings() {
        assert_eq!(failure("glowing + 1"), non_numeric("+", Scalar::Bool));
        assert_eq!(failure("tag * 2"), non_numeric("*", Scalar::String));
        assert_eq!(failure("rough + tag"), non_numeric("+", Scalar::String));
        assert_eq!(failure("-glowing"), non_numeric("-", Scalar::Bool));
        assert_eq!(failure("-tag"), non_numeric("-", Scalar::String));
    }

    // Shapes and domains.

    #[test]
    fn elementwise_operations_climb_the_lower_domain() {
        assert_eq!(
            typed("0.5 * rough"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("color * occlusion"),
            ty(Domain::Corner, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("rough + faceValue"),
            ty(Domain::Face, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("position.x + count"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("plain3 + color.rgb"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("lerp(1, occlusion, 0.8)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("glowing && faceValue > 0.5"),
            ty(Domain::Face, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("mix(0.5, 1, occlusion < 0.5)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
    }

    #[test]
    fn climbs_lift_and_never_step_down() {
        assert_eq!(
            typed("face(rough)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("corner(0.5)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("swatch(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("voxel(color)"),
            ty(Domain::Voxel, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("face(glowing)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("face(tag)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("face(1u32)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            failure("face(occlusion)"),
            CheckFailure::StepDown {
                operation: "face".to_owned(),
                found: Domain::Corner
            }
        );
        assert_eq!(
            failure("swatch(position)"),
            CheckFailure::StepDown {
                operation: "swatch".to_owned(),
                found: Domain::Voxel
            }
        );
        assert_eq!(
            failure("voxel(faceValue)"),
            CheckFailure::StepDown {
                operation: "voxel".to_owned(),
                found: Domain::Face
            }
        );
    }

    #[test]
    fn reductions_step_down_from_above_their_destination() {
        assert_eq!(
            typed("faceAvg(occlusion)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("faceSum(occlusion)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("faceMin(occlusion)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("faceMax(occlusion)"),
            ty(Domain::Face, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("voxelAvg(faceValue)"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("voxelMin(occlusion)"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("swatchAvg(occlusion)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("swatchSum(face(1u32))"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("swatchMin(voxelMin(occlusion))"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );

        let source = |operation: &str, found: Domain| CheckFailure::ReductionSource {
            operation: operation.to_owned(),
            found,
        };

        assert_eq!(
            failure("faceAvg(faceValue)"),
            source("faceAvg", Domain::Face)
        );
        assert_eq!(
            failure("faceSum(position)"),
            source("faceSum", Domain::Voxel)
        );
        assert_eq!(
            failure("voxelSum(position)"),
            source("voxelSum", Domain::Voxel)
        );
        assert_eq!(
            failure("voxelMax(rough)"),
            source("voxelMax", Domain::Swatch)
        );
        assert_eq!(
            failure("swatchSum(rough)"),
            source("swatchSum", Domain::Swatch)
        );
        assert_eq!(
            failure("swatchAvg(0.5)"),
            source("swatchAvg", Domain::Plain)
        );
    }

    #[test]
    fn the_plain_reductions_take_an_array() {
        assert_eq!(
            typed("max(rough)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("min(color)"),
            ty(Domain::Plain, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("sum(occlusion)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("avg(rough + 1)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("avg(position)"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );

        for text in ["max(0.5)", "min(plain3)", "sum(plainCount)", "avg(0.5)"] {
            assert!(
                matches!(failure(text), CheckFailure::RequiresArray { .. }),
                "{text}"
            );
        }
    }

    #[test]
    fn indexing_samples_an_array_into_a_plain_value() {
        assert_eq!(
            typed("color[0]"),
            ty(Domain::Plain, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("color[0].rgb"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("color.rgb[0]"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("tag[1u8]"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("glowing[0]"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("position[plainCount]"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::U32)
        );
        assert_eq!(
            typed("occlusion[3u16]"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            failure("0.5[0]"),
            CheckFailure::RequiresArray {
                operation: "[]".to_owned()
            }
        );
        assert_eq!(
            failure("color[rough]"),
            CheckFailure::RequiresPlain {
                operation: "[]".to_owned(),
                found: Domain::Swatch
            }
        );
        assert_eq!(
            failure("color[0.5]"),
            CheckFailure::IndexScalar { found: Scalar::F32 }
        );
        assert_eq!(
            failure("color[true]"),
            CheckFailure::IndexScalar {
                found: Scalar::Bool
            }
        );
        assert_eq!(
            failure("color[plain3]"),
            CheckFailure::RequiresDimension {
                operation: "[]".to_owned(),
                expected: Dimension::Vec1,
                found: Dimension::Vec3
            }
        );
    }

    // Booleans.

    #[test]
    fn comparisons_make_bools_at_vec1() {
        assert_eq!(
            typed("rough < 0.5"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("rough <= 0.5"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("rough > 0.5"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("rough >= 0.5"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("count == 2"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("count != 2"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("0.1 < 0.2"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("faceValue < rough"),
            ty(Domain::Face, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            failure("color < 0.5"),
            CheckFailure::WideComparison {
                operation: "<".to_owned(),
                found: Dimension::Vec4
            }
        );
        assert_eq!(
            failure("rough == color.rgb"),
            CheckFailure::WideComparison {
                operation: "==".to_owned(),
                found: Dimension::Vec3
            }
        );
    }

    #[test]
    fn comparisons_take_numbers_or_string_equality() {
        assert_eq!(
            typed("tag == \"glass\""),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("\"a\" != tag"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("\"a\" == \"b\""),
            ty(Domain::Plain, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            failure("tag < \"x\""),
            CheckFailure::StringOrder {
                operation: "<".to_owned()
            }
        );
        assert_eq!(failure("tag == 1"), non_numeric("==", Scalar::String));
        assert_eq!(failure("rough == tag"), non_numeric("==", Scalar::String));
        assert_eq!(failure("glowing == true"), non_numeric("==", Scalar::Bool));
        assert_eq!(
            failure("glowing != glowing"),
            non_numeric("!=", Scalar::Bool)
        );
    }

    #[test]
    fn a_comparison_chain_errors() {
        assert_eq!(failure("rough < 0.5 < 1.0"), non_numeric("<", Scalar::Bool));
        assert_eq!(
            failure("1.0 == 1.0 == true"),
            non_numeric("==", Scalar::Bool)
        );
    }

    #[test]
    fn folds_take_a_comparison_written_in_place() {
        assert_eq!(
            typed("all(color.rgb > 0.9)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("any(color == color)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("any(0.5 < color)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("all(rough > 0.5)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("any((rough > 0.5))"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("all(tag == \"x\")"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("any(position > 2)"),
            ty(Domain::Voxel, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            rendered("all(color.rgb > 0.9)"),
            "(all (> (. `color` 012) 0.9f32))"
        );
        assert_eq!(
            failure("any(color.rg > color.rgb)"),
            dimensions(">", &[Dimension::Vec2, Dimension::Vec3])
        );

        for text in ["any(glowing)", "all(rough)", "any(true)", "all(!glowing)"] {
            assert!(
                matches!(failure(text), CheckFailure::FoldNeedsComparison { .. }),
                "{text}"
            );
        }

        assert_eq!(
            failure("any(tag < \"x\")"),
            CheckFailure::StringOrder {
                operation: "<".to_owned()
            }
        );
        assert_eq!(failure("all(1 < 2)"), CheckFailure::UntypedLiteral);
    }

    #[test]
    fn the_logical_operators_take_bools() {
        assert_eq!(
            typed("!glowing"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("glowing && rough > 0.5"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("glowing ^ true"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("glowing || false"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("!true && false"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(failure("!rough"), non_bool("!", Scalar::F32));
        assert_eq!(failure("!tag"), non_bool("!", Scalar::String));
        assert_eq!(failure("glowing || tag"), non_bool("||", Scalar::String));
        assert_eq!(failure("rough && glowing"), non_bool("&&", Scalar::F32));
        assert_eq!(failure("glowing ^ count"), non_bool("^", Scalar::U32));
        assert_eq!(failure("glowing && 1"), CheckFailure::UntypedLiteral);
        assert_eq!(failure("!1"), CheckFailure::UntypedLiteral);
    }

    #[test]
    fn a_bool_never_meets_a_number() {
        assert_eq!(
            failure("rgb(glowing, 0, 0)"),
            non_numeric("rgb", Scalar::Bool)
        );
        assert_eq!(failure("glowing * 2"), non_numeric("*", Scalar::Bool));
        assert_eq!(failure("pow(glowing, 2)"), non_numeric("pow", Scalar::Bool));
        assert_eq!(failure("max(glowing)"), non_numeric("max", Scalar::Bool));
        assert_eq!(failure("sum(glowing)"), non_numeric("sum", Scalar::Bool));
        assert_eq!(failure("avg(glowing)"), non_numeric("avg", Scalar::Bool));
        assert_eq!(failure("glowing.r"), non_numeric(".r", Scalar::Bool));
        assert_eq!(
            failure("faceAvg(corner(glowing))"),
            non_numeric("faceAvg", Scalar::Bool)
        );
        assert_eq!(
            failure("swatchSum(face(glowing))"),
            non_numeric("swatchSum", Scalar::Bool)
        );
        assert_eq!(
            failure("round_u8(glowing)"),
            non_numeric("round_u8", Scalar::Bool)
        );
        assert_eq!(failure("abs(true)"), non_numeric("abs", Scalar::Bool));
    }

    #[test]
    fn mix_bridges_a_bool_to_values_of_one_type() {
        assert_eq!(
            typed("mix(0f32, 1, glowing)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("mix(1, 2, glowing) * rough"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            rendered("mix(1, 2, glowing) * count"),
            "(* (mix 1u32 2u32 `glowing`) `count`)"
        );
        assert_eq!(
            typed("mix(color, color, glowing)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("mix(plain3, plain3, true)"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("mix(count, 0, glowing)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("mix(wide, wide, true)"),
            ty(Domain::Voxel, Dimension::Vec2, Scalar::U8)
        );
        assert_eq!(
            typed("mix(\"a\", \"b\", glowing)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("mix(\"a\", tag, true)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("mix(glowing, false, glowing)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("mix(0.5, 1, occlusion < 0.5)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            failure("mix(color, color.rgb, glowing)"),
            dimensions("mix", &[Dimension::Vec4, Dimension::Vec3])
        );
        assert_eq!(
            failure("mix(count, 1.5, glowing)"),
            mixed("mix", &[Scalar::U32, Scalar::F32])
        );
        assert_eq!(
            failure("mix(\"a\", true, glowing)"),
            mixed("mix", &[Scalar::String, Scalar::Bool])
        );
        assert_eq!(
            failure("mix(\"a\", 1, glowing)"),
            non_numeric("mix", Scalar::String)
        );
        assert_eq!(
            failure("mix(glowing, 1, glowing)"),
            non_numeric("mix", Scalar::Bool)
        );
        assert_eq!(failure("mix(1, 2, rough)"), non_bool("mix", Scalar::F32));
        assert_eq!(failure("mix(1, 2, tag)"), non_bool("mix", Scalar::String));
        assert_eq!(failure("mix(1, 2, 3)"), CheckFailure::UntypedLiteral);
        assert_eq!(failure("mix(1, 2, glowing)"), CheckFailure::UntypedLiteral);
    }

    // Strings.

    #[test]
    fn a_string_takes_five_operations_and_nothing_else() {
        assert_eq!(
            typed("tag == \"glass\""),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("mix(\"a\", \"b\", glowing)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("tag[0]"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("default(missing, \"x\")"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("corner(tag)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("swatch(tag) == \"x\""),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(failure("tag + \"x\""), non_numeric("+", Scalar::String));
        assert_eq!(failure("tag.r"), non_numeric(".r", Scalar::String));
        assert_eq!(failure("max(tag)"), non_numeric("max", Scalar::String));
        assert_eq!(
            failure("faceAvg(corner(tag))"),
            non_numeric("faceAvg", Scalar::String)
        );
        assert_eq!(
            failure("length(tag)"),
            non_numeric("length", Scalar::String)
        );
        assert_eq!(
            failure("rgb(tag, 1, 1)"),
            non_numeric("rgb", Scalar::String)
        );
        assert_eq!(failure("f32(\"1\")"), non_numeric("f32", Scalar::String));
    }

    // Swizzles.

    #[test]
    fn a_swizzle_draws_from_one_alphabet_within_its_source() {
        assert_eq!(
            typed("color.rgb"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("color.xyz"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("color.a"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("color.wzyx"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("rough.r"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("rough.rr"),
            ty(Domain::Swatch, Dimension::Vec2, Scalar::F32)
        );
        assert_eq!(
            typed("rough.xxxx"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("0.5.rrr"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("color.rrgg"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("plain3.zy"),
            ty(Domain::Plain, Dimension::Vec2, Scalar::F32)
        );
        assert_eq!(
            typed("2.rr * count"),
            ty(Domain::Swatch, Dimension::Vec2, Scalar::U32)
        );
        assert_eq!(
            typed("(color + color).x"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(rendered("color.wzyx"), "(. `color` 3210)");
        assert_eq!(
            failure("color.xg"),
            CheckFailure::SwizzleAlphabets {
                member: "xg".to_owned()
            }
        );
        assert_eq!(
            failure("color.q"),
            CheckFailure::SwizzleCharacter {
                member: "q".to_owned(),
                character: 'q'
            }
        );
        assert_eq!(
            failure("color.rgbar"),
            CheckFailure::SwizzleLength {
                member: "rgbar".to_owned()
            }
        );
        assert_eq!(
            failure("rough.g"),
            CheckFailure::SwizzleComponent {
                member: "g".to_owned(),
                character: 'g',
                found: Dimension::Vec1
            }
        );
        assert_eq!(
            failure("plain3.xyzw"),
            CheckFailure::SwizzleComponent {
                member: "xyzw".to_owned(),
                character: 'w',
                found: Dimension::Vec3
            }
        );
    }

    // Functions.

    #[test]
    fn the_constructors_take_vec1_parts() {
        assert_eq!(
            typed("r(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("rg(rough, 0.5)"),
            ty(Domain::Swatch, Dimension::Vec2, Scalar::F32)
        );
        assert_eq!(
            typed("rgb(rough, rough, occlusion)"),
            ty(Domain::Corner, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("rgba(1, 1, 1, 1)"),
            ty(Domain::Plain, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            failure("rgb(color.rg, 1, 1)"),
            CheckFailure::RequiresDimension {
                operation: "rgb".to_owned(),
                expected: Dimension::Vec1,
                found: Dimension::Vec2
            }
        );
        assert_eq!(
            failure("r(plain3)"),
            CheckFailure::RequiresDimension {
                operation: "r".to_owned(),
                expected: Dimension::Vec1,
                found: Dimension::Vec3
            }
        );
    }

    #[test]
    fn binary_min_and_max_broadcast_from_either_side() {
        assert_eq!(
            typed("max(color, 0.5)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("min(0.5, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("max(color, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("max(count, 2)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("max(occlusion, 0.2)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            failure("min(color, color.rgb)"),
            dimensions("min", &[Dimension::Vec4, Dimension::Vec3])
        );
    }

    #[test]
    fn pow_and_mod_broadcast_their_right_side_alone() {
        assert_eq!(
            typed("pow(color, 2)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("pow(color, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("mod(position, 2)"),
            ty(Domain::Voxel, Dimension::Vec3, Scalar::U32)
        );
        assert_eq!(
            typed("mod(rough + 0.618, 1)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            failure("pow(2, color)"),
            dimensions("pow", &[Dimension::Vec1, Dimension::Vec4])
        );
        assert_eq!(
            failure("mod(color, color.rgb)"),
            dimensions("mod", &[Dimension::Vec4, Dimension::Vec3])
        );
    }

    #[test]
    fn clamp_and_smoothstep_pair_their_bounds() {
        assert_eq!(
            typed("clamp(color, 0, 1)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("clamp(color, color, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("smoothstep(0.2, 0.8, occlusion)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("smoothstep(color, color, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            failure("clamp(color, 0, color)"),
            dimensions(
                "clamp",
                &[Dimension::Vec4, Dimension::Vec1, Dimension::Vec4]
            )
        );
        assert_eq!(
            failure("clamp(rough, color, color)"),
            dimensions(
                "clamp",
                &[Dimension::Vec1, Dimension::Vec4, Dimension::Vec4]
            )
        );
        assert_eq!(
            failure("smoothstep(color.rg, color.rg, rough)"),
            dimensions(
                "smoothstep",
                &[Dimension::Vec2, Dimension::Vec2, Dimension::Vec1]
            )
        );
    }

    #[test]
    fn lerp_and_step_broadcast_one_argument() {
        assert_eq!(
            typed("lerp(color, color, 0.5)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("lerp(color, color, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("step(0.5, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("step(color, color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            failure("lerp(color, rough, 0.5)"),
            dimensions("lerp", &[Dimension::Vec4, Dimension::Vec1, Dimension::Vec1])
        );
        assert_eq!(
            failure("lerp(rough, rough, color)"),
            dimensions("lerp", &[Dimension::Vec1, Dimension::Vec1, Dimension::Vec4])
        );
        assert_eq!(
            failure("step(color, rough)"),
            dimensions("step", &[Dimension::Vec4, Dimension::Vec1])
        );
    }

    #[test]
    fn the_vector_functions_fold_components_and_never_broadcast() {
        assert_eq!(
            typed("dot(color, color)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("dot(rough, rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("length(color)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("length(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("distance(color.rgb, plain3)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("normalize(color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("cross(color.rgb, plain3)"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            failure("dot(color, rough)"),
            dimensions("dot", &[Dimension::Vec4, Dimension::Vec1])
        );
        assert_eq!(
            failure("distance(color, rough)"),
            dimensions("distance", &[Dimension::Vec4, Dimension::Vec1])
        );
        assert_eq!(
            failure("cross(color, color)"),
            CheckFailure::RequiresDimension {
                operation: "cross".to_owned(),
                expected: Dimension::Vec3,
                found: Dimension::Vec4
            }
        );
    }

    #[test]
    fn the_color_conversions_take_vec3() {
        assert_eq!(
            typed("oklabFromRgb(color.rgb)"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("rgbFromOklab(plain3)"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("oklchFromRgb(color.rgb).z"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("rgbFromOklch(oklchFromRgb(color.rgb) + rgb(0, 0, 0.1))"),
            ty(Domain::Swatch, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            failure("oklchFromRgb(color)"),
            CheckFailure::RequiresDimension {
                operation: "oklchFromRgb".to_owned(),
                expected: Dimension::Vec3,
                found: Dimension::Vec4
            }
        );
        assert_eq!(
            failure("rgbFromOklch(position)"),
            requires_f32("rgbFromOklch", Scalar::U32)
        );
    }

    #[test]
    fn rounding_and_abs_keep_f32_and_dimension() {
        assert_eq!(
            typed("abs(color)"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("floor(rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("ceil(occlusion)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("round(plain3)"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("round(rough * 4) / 4"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
    }

    #[test]
    fn default_fills_an_unbound_name_from_its_fallback() {
        assert_eq!(
            typed("default(missing, 1)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(rendered("default(missing, 1)"), "(default `missing`? 1f32)");
        assert_eq!(
            typed("default(missing, rgb(0, 0, 0))"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            typed("default(missing, \"x\")"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("default(missing, true)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("default(missing, 1u32)"),
            ty(Domain::Plain, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("default(missing, count)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(
            typed("default(missing, rough)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("swatch(default(missing, 1))"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            failure("default(missing, 1) * count"),
            mixed("*", &[Scalar::F32, Scalar::U32])
        );
    }

    #[test]
    fn default_reads_a_bound_name_beside_a_fallback_of_its_type() {
        assert_eq!(
            typed("default(rough, 1)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(rendered("default(rough, 1)"), "(default `rough` 1f32)");
        assert_eq!(
            typed("default(count, 1)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::U32)
        );
        assert_eq!(rendered("default(count, 1)"), "(default `count` 1u32)");
        assert_eq!(
            typed("default(wide, wide)"),
            ty(Domain::Voxel, Dimension::Vec2, Scalar::U8)
        );
        assert_eq!(
            typed("default(color, rgba(1, 1, 1, 1))"),
            ty(Domain::Swatch, Dimension::Vec4, Scalar::F32)
        );
        assert_eq!(
            typed("default(tag, \"x\")"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::String)
        );
        assert_eq!(
            typed("default(glowing, true)"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
        assert_eq!(
            typed("default(rough, occlusion)"),
            ty(Domain::Corner, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("default(plain3, plain3)"),
            ty(Domain::Plain, Dimension::Vec3, Scalar::F32)
        );
        assert_eq!(
            failure("default(rough, color)"),
            dimensions("default", &[Dimension::Vec1, Dimension::Vec4])
        );
        assert_eq!(
            failure("default(tag, 1)"),
            mixed("default", &[Scalar::String, Scalar::F32])
        );
        assert_eq!(
            failure("default(glowing, 1)"),
            mixed("default", &[Scalar::Bool, Scalar::F32])
        );
        assert_eq!(
            failure("default(rough, \"x\")"),
            mixed("default", &[Scalar::F32, Scalar::String])
        );
        assert_eq!(
            failure("default(count, 1.5)"),
            mixed("default", &[Scalar::U32, Scalar::F32])
        );
        assert_eq!(
            failure("default(count, 1u8)"),
            mixed("default", &[Scalar::U32, Scalar::U8])
        );
        assert_eq!(
            failure("default(rough, glowing)"),
            mixed("default", &[Scalar::F32, Scalar::Bool])
        );
        assert_eq!(
            failure("default(count, 5000000000)"),
            CheckFailure::LiteralOutOfRange {
                text: "5000000000".to_owned(),
                scalar: Scalar::U32
            }
        );
    }

    #[test]
    fn an_unknown_name_errors() {
        assert_eq!(
            failure("missing"),
            CheckFailure::UnknownName {
                name: "missing".to_owned()
            }
        );
        assert_eq!(
            failure("rough + missing"),
            CheckFailure::UnknownName {
                name: "missing".to_owned()
            }
        );
        assert_eq!(
            failure("`min`"),
            CheckFailure::UnknownName {
                name: "min".to_owned()
            }
        );
    }

    #[test]
    fn parentheses_group_without_changing_the_type() {
        assert_eq!(
            typed("(rough + 1) * 2"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::F32)
        );
        assert_eq!(
            typed("((glowing))"),
            ty(Domain::Swatch, Dimension::Vec1, Scalar::Bool)
        );
    }
}
