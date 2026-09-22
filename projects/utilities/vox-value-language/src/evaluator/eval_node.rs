use crate::{
    Components, EvalFailure, Groupings, Scalar, Type, Value,
    checker::{CheckedKind, CheckedNode, ElementwiseFunction, Fold, NumberValue},
    evaluator::{
        EntryPairTransform, EntryTransform, EvalResult, Lengths, Operand, Unsigned, ValueExt,
        climb, componentwise, convert, oklab_from_rgb, oklch_from_rgb, per_entry, reduce,
        rgb_from_oklab, rgb_from_oklch, transform_entries, transform_entry_pairs,
    },
    parser::{BinaryOperator, ComparisonOperator, LogicalOperator, UnaryOperator},
};
use std::collections::HashMap;

/// Computes a checked node's value over the names in scope.
pub(crate) fn eval_node(
    node: &CheckedNode,
    scope: &HashMap<String, Value>,
    groupings: &Groupings,
    lengths: &Lengths,
) -> EvalResult<Value> {
    Evaluator {
        scope,
        groupings,
        lengths,
    }
    .node(node)
}

struct Evaluator<'a> {
    scope: &'a HashMap<String, Value>,
    groupings: &'a Groupings,
    lengths: &'a Lengths,
}

impl Evaluator<'_> {
    fn node(&self, node: &CheckedNode) -> EvalResult<Value> {
        let output = node.output;

        match &node.kind {
            CheckedKind::Binary {
                operator,
                left,
                right,
            } => self.binary(*operator, left, right, output),

            CheckedKind::Bool(value) => Ok(build(output, Components::Bool(vec![*value]))),

            CheckedKind::Call {
                function,
                arguments,
            } => self.call(*function, arguments, output),

            CheckedKind::Climb { target, operand } => {
                self.lifted(operand, output.with_domain(*target))
            }

            CheckedKind::Comparison {
                operator,
                left,
                right,
            } => self.comparison(*operator, left, right, output),

            CheckedKind::Convert {
                target,
                rounding,
                operand,
            } => convert(&self.node(operand)?, *target, *rounding),

            CheckedKind::Default {
                name,
                bound,
                fallback,
            } => {
                if bound.is_some() {
                    self.lift(self.named(name), output)
                } else {
                    self.lifted(fallback, output)
                }
            }

            CheckedKind::Fold { fold, operand } => self.fold(*fold, operand, output),

            CheckedKind::Index { source, index } => {
                index_entry(&self.node(source)?, &self.node(index)?, output)
            }

            CheckedKind::Logical {
                operator,
                left,
                right,
            } => self.logical(*operator, left, right, output),

            CheckedKind::Mix {
                first,
                second,
                chooser,
            } => self.mix(first, second, chooser, output),

            CheckedKind::Name(name) => Ok(self.named(name).clone()),
            CheckedKind::Number(value) => Ok(build(output, number(*value))),

            CheckedKind::Reduce {
                reduction,
                target,
                operand,
            } => reduce(
                &self.node(operand)?,
                *reduction,
                *target,
                self.groupings,
                self.lengths,
            ),

            CheckedKind::StringLiteral(text) => {
                Ok(build(output, Components::String(vec![text.clone()])))
            }

            CheckedKind::Swizzle { source, components } => {
                swizzle(&self.node(source)?, components, output)
            }

            CheckedKind::Unary { operator, operand } => self.unary(*operator, operand, output),
        }
    }

    /// The value a name holds.
    fn named(&self, name: &str) -> &Value {
        self.scope
            .get(name)
            .expect("the checker binds every name it reads")
    }

    /// The value lifted to the output's domain.
    fn lift(&self, value: &Value, output: Type) -> EvalResult<Value> {
        climb(value, output.domain, self.groupings, self.lengths)
    }

    /// The node's value lifted to the output's domain.
    fn lifted(&self, node: &CheckedNode, output: Type) -> EvalResult<Value> {
        self.lift(&self.node(node)?, output)
    }

    fn binary(
        &self,
        operator: BinaryOperator,
        left: &CheckedNode,
        right: &CheckedNode,
        output: Type,
    ) -> EvalResult<Value> {
        let left = self.lifted(left, output)?;
        let right = self.lifted(right, output)?;
        let entries = self.lengths.of(output.domain);
        let width = output.dimension.width();
        let components = match output.scalar {
            Scalar::F32 => {
                let values =
                    componentwise([left.f32s(), right.f32s()], entries, width, |[a, b]| {
                        Ok(match operator {
                            BinaryOperator::Add => a + b,
                            BinaryOperator::Divide => a / b,
                            BinaryOperator::Multiply => a * b,
                            BinaryOperator::Subtract => a - b,
                        })
                    })?;

                Components::F32(finite(values, &operator.to_string())?)
            }

            Scalar::U8 => Components::U8(unsigned_binary(
                operator,
                [left.u8s(), right.u8s()],
                entries,
                width,
            )?),

            Scalar::U16 => Components::U16(unsigned_binary(
                operator,
                [left.u16s(), right.u16s()],
                entries,
                width,
            )?),

            Scalar::U32 => Components::U32(unsigned_binary(
                operator,
                [left.u32s(), right.u32s()],
                entries,
                width,
            )?),

            Scalar::Bool | Scalar::String => {
                unreachable!("the checker rejects arithmetic on bools and strings")
            }
        };

        Ok(build(output, components))
    }

    fn call(
        &self,
        function: ElementwiseFunction,
        arguments: &[CheckedNode],
        output: Type,
    ) -> EvalResult<Value> {
        let values = arguments
            .iter()
            .map(|argument| self.lifted(argument, output))
            .collect::<EvalResult<Vec<_>>>()?;
        let entries = self.lengths.of(output.domain);
        let width = output.dimension.width();
        let components = match function {
            ElementwiseFunction::Max | ElementwiseFunction::Min | ElementwiseFunction::Mod => {
                keep_call(function, &values, output.scalar, entries, width)?
            }

            // Only a constructor answers a bool, packing its vec1 parts.
            _ if output.scalar == Scalar::Bool => {
                let parts: Vec<_> = values.iter().map(ValueExt::bools).collect();
                let mut packed = Vec::with_capacity(entries * width);

                for entry in 0..entries {
                    packed.extend(parts.iter().map(|part| *part.component(entry, 0)));
                }

                Components::Bool(packed)
            }

            _ => Components::F32(finite(
                f32_call(function, &values, entries, width)?,
                function.to_function().name(),
            )?),
        };

        Ok(build(output, components))
    }

    fn comparison(
        &self,
        operator: ComparisonOperator,
        left: &CheckedNode,
        right: &CheckedNode,
        output: Type,
    ) -> EvalResult<Value> {
        let left = self.lifted(left, output)?;
        let right = self.lifted(right, output)?;
        let entries = self.lengths.of(output.domain);
        let width = output.dimension.width();
        let answers = compare_values(operator, &left, &right, entries, width)?;

        Ok(build(output, Components::Bool(answers)))
    }

    fn fold(&self, fold: Fold, operand: &CheckedNode, output: Type) -> EvalResult<Value> {
        let operand = self.node(operand)?;
        let width = operand.dimension().width();
        let folded = operand
            .bools()
            .components
            .chunks(width)
            .map(|entry| match fold {
                Fold::All => entry.iter().all(|&answer| answer),
                Fold::Any => entry.iter().any(|&answer| answer),
            })
            .collect();

        Ok(build(output, Components::Bool(folded)))
    }

    fn logical(
        &self,
        operator: LogicalOperator,
        left: &CheckedNode,
        right: &CheckedNode,
        output: Type,
    ) -> EvalResult<Value> {
        let left = self.lifted(left, output)?;
        let right = self.lifted(right, output)?;
        let entries = self.lengths.of(output.domain);
        let width = output.dimension.width();
        let answers = componentwise([left.bools(), right.bools()], entries, width, |[a, b]| {
            Ok(match operator {
                LogicalOperator::And => *a && *b,
                LogicalOperator::Or => *a || *b,
                LogicalOperator::Xor => *a ^ *b,
            })
        })?;

        Ok(build(output, Components::Bool(answers)))
    }

    fn mix(
        &self,
        first: &CheckedNode,
        second: &CheckedNode,
        chooser: &CheckedNode,
        output: Type,
    ) -> EvalResult<Value> {
        let first = self.lifted(first, output)?;
        let second = self.lifted(second, output)?;
        let chooser = self.lifted(chooser, output)?;
        let transform = Mix {
            width: output.dimension.width(),
            chooser_width: chooser.dimension().width(),
            chooser: chooser.bools().components,
        };

        Ok(build(
            output,
            transform_entry_pairs(first.components(), second.components(), &transform),
        ))
    }

    fn unary(
        &self,
        operator: UnaryOperator,
        operand: &CheckedNode,
        output: Type,
    ) -> EvalResult<Value> {
        let operand = self.node(operand)?;
        let components = match operator {
            UnaryOperator::Negate => Components::F32(
                operand
                    .f32s()
                    .components
                    .iter()
                    .map(|value| -value)
                    .collect(),
            ),
            UnaryOperator::Not => Components::Bool(
                operand
                    .bools()
                    .components
                    .iter()
                    .map(|value| !value)
                    .collect(),
            ),
        };

        Ok(build(output, components))
    }
}

/// Picks each component from the second operand where the chooser holds
/// and the first elsewhere. A vec1 chooser picks whole entries.
struct Mix<'a> {
    width: usize,
    chooser_width: usize,
    chooser: &'a [bool],
}

impl EntryPairTransform for Mix<'_> {
    fn apply<T: Clone>(&self, first: &[T], second: &[T]) -> Vec<T> {
        first
            .iter()
            .zip(second)
            .enumerate()
            .map(|(index, (first, second))| {
                let chosen = if self.chooser_width == 1 {
                    self.chooser[index / self.width]
                } else {
                    self.chooser[index]
                };

                if chosen {
                    second.clone()
                } else {
                    first.clone()
                }
            })
            .collect()
    }
}

/// Reorders each entry's components by the swizzle's positions.
struct Swizzle<'a> {
    width: usize,
    positions: &'a [usize],
}

impl EntryTransform for Swizzle<'_> {
    fn apply<T: Clone + PartialEq>(&self, components: &[T]) -> EvalResult<Vec<T>> {
        Ok(components
            .chunks(self.width)
            .flat_map(|entry| {
                self.positions
                    .iter()
                    .map(|&position| entry[position].clone())
            })
            .collect())
    }
}

/// Takes one entry.
struct Take {
    width: usize,
    entry: usize,
}

impl EntryTransform for Take {
    fn apply<T: Clone + PartialEq>(&self, components: &[T]) -> EvalResult<Vec<T>> {
        Ok(components[self.entry * self.width..(self.entry + 1) * self.width].to_vec())
    }
}

/// Swizzles a value's entries.
fn swizzle(source: &Value, positions: &[usize], output: Type) -> EvalResult<Value> {
    let transform = Swizzle {
        width: source.dimension().width(),
        positions,
    };

    Ok(build(
        output,
        transform_entries(source.components(), &transform)?,
    ))
}

/// Samples an array at the index into a plain value.
fn index_entry(source: &Value, index: &Value, output: Type) -> EvalResult<Value> {
    let index = match index.components() {
        Components::U8(components) => u32::from(components[0]),
        Components::U16(components) => u32::from(components[0]),
        Components::U32(components) => components[0],
        _ => unreachable!("the checker settles an unsigned index"),
    };
    let entries = source.entries();
    let out_of_range = || EvalFailure::IndexOutOfRange { index, entries };
    let entry = usize::try_from(index).map_err(|_| out_of_range())?;

    if entry >= entries {
        return Err(out_of_range());
    }

    let transform = Take {
        width: output.dimension.width(),
        entry,
    };

    Ok(build(
        output,
        transform_entries(source.components(), &transform)?,
    ))
}

/// Runs an `f32` function over lifted arguments.
fn f32_call(
    function: ElementwiseFunction,
    values: &[Value],
    entries: usize,
    width: usize,
) -> EvalResult<Vec<f32>> {
    let operands = values.iter().map(ValueExt::f32s).collect::<Vec<_>>();

    match function {
        ElementwiseFunction::R
        | ElementwiseFunction::Rg
        | ElementwiseFunction::Rgb
        | ElementwiseFunction::Rgba => {
            let mut output = Vec::with_capacity(entries * width);

            for entry in 0..entries {
                output.extend(operands.iter().map(|operand| *operand.component(entry, 0)));
            }

            Ok(output)
        }

        ElementwiseFunction::Abs => {
            componentwise(fixed(&operands), entries, width, |[a]| Ok(a.abs()))
        }
        ElementwiseFunction::Ceil => {
            componentwise(fixed(&operands), entries, width, |[a]| Ok(a.ceil()))
        }
        ElementwiseFunction::Floor => {
            componentwise(fixed(&operands), entries, width, |[a]| Ok(a.floor()))
        }
        ElementwiseFunction::Round => {
            componentwise(fixed(&operands), entries, width, |[a]| Ok(a.round()))
        }

        ElementwiseFunction::Length => per_entry(fixed(&operands), entries, |[vector]| {
            Ok(vec![magnitude(vector)])
        }),

        ElementwiseFunction::Normalize => per_entry(fixed(&operands), entries, |[vector]| {
            let magnitude = magnitude(vector);

            Ok(vector
                .iter()
                .map(|component| component / magnitude)
                .collect())
        }),

        ElementwiseFunction::Dot => {
            per_entry(fixed(&operands), entries, |[a, b]| Ok(vec![dot(a, b)]))
        }

        ElementwiseFunction::Distance => per_entry(fixed(&operands), entries, |[a, b]| {
            let difference = a.iter().zip(b).map(|(a, b)| a - b).collect::<Vec<_>>();

            Ok(vec![magnitude(&difference)])
        }),

        ElementwiseFunction::Cross => per_entry(fixed(&operands), entries, |[a, b]| {
            Ok(vec![
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ])
        }),

        ElementwiseFunction::OklabFromRgb => per_entry(fixed(&operands), entries, |[color]| {
            Ok(oklab_from_rgb(triple(color)).to_vec())
        }),
        ElementwiseFunction::OklchFromRgb => per_entry(fixed(&operands), entries, |[color]| {
            Ok(oklch_from_rgb(triple(color)).to_vec())
        }),
        ElementwiseFunction::RgbFromOklab => per_entry(fixed(&operands), entries, |[color]| {
            Ok(rgb_from_oklab(triple(color)).to_vec())
        }),
        ElementwiseFunction::RgbFromOklch => per_entry(fixed(&operands), entries, |[color]| {
            Ok(rgb_from_oklch(triple(color))?.to_vec())
        }),

        ElementwiseFunction::Pow => {
            componentwise(fixed(&operands), entries, width, |[a, b]| Ok(a.powf(*b)))
        }

        ElementwiseFunction::Clamp => {
            componentwise(fixed(&operands), entries, width, |[x, low, high]| {
                if low > high {
                    return Err(EvalFailure::InvertedBounds {
                        operation: "clamp".to_owned(),
                        low: *low,
                        high: *high,
                    });
                }

                Ok(x.clamp(*low, *high))
            })
        }

        ElementwiseFunction::Lerp => {
            componentwise(fixed(&operands), entries, width, |[a, b, t]| {
                Ok(a + (b - a) * t)
            })
        }

        ElementwiseFunction::Step => {
            componentwise(fixed(&operands), entries, width, |[edge, x]| {
                Ok(if x < edge { 0.0 } else { 1.0 })
            })
        }

        ElementwiseFunction::Smoothstep => {
            componentwise(fixed(&operands), entries, width, |[low, high, x]| {
                if low >= high {
                    return Err(EvalFailure::InvertedBounds {
                        operation: "smoothstep".to_owned(),
                        low: *low,
                        high: *high,
                    });
                }

                let t = ((x - low) / (high - low)).clamp(0.0, 1.0);

                Ok(t * t * (3.0 - 2.0 * t))
            })
        }

        ElementwiseFunction::Max | ElementwiseFunction::Min | ElementwiseFunction::Mod => {
            unreachable!("the type-keeping calls take their own path")
        }
    }
}

/// Runs a type-keeping binary function over lifted arguments.
fn keep_call(
    function: ElementwiseFunction,
    values: &[Value],
    scalar: Scalar,
    entries: usize,
    width: usize,
) -> EvalResult<Components> {
    let (first, second) = (&values[0], &values[1]);
    let operation = function.to_function().name();

    Ok(match scalar {
        Scalar::F32 => {
            let output = componentwise([first.f32s(), second.f32s()], entries, width, |[a, b]| {
                Ok(match function {
                    ElementwiseFunction::Max => a.max(*b),
                    ElementwiseFunction::Min => a.min(*b),
                    ElementwiseFunction::Mod => a - b * (a / b).floor(),
                    _ => unreachable!("the f32 calls take their own path"),
                })
            })?;

            Components::F32(finite(output, operation)?)
        }

        Scalar::U8 => Components::U8(unsigned_keep(
            function,
            [first.u8s(), second.u8s()],
            entries,
            width,
        )?),

        Scalar::U16 => Components::U16(unsigned_keep(
            function,
            [first.u16s(), second.u16s()],
            entries,
            width,
        )?),

        Scalar::U32 => Components::U32(unsigned_keep(
            function,
            [first.u32s(), second.u32s()],
            entries,
            width,
        )?),

        Scalar::Bool | Scalar::String => {
            unreachable!("the checker rejects a bool or string under min, max, and mod")
        }
    })
}

/// Runs a type-keeping binary function over unsigned operands.
fn unsigned_keep<T: Unsigned>(
    function: ElementwiseFunction,
    operands: [Operand<'_, T>; 2],
    entries: usize,
    width: usize,
) -> EvalResult<Vec<T>> {
    componentwise(operands, entries, width, |[a, b]| {
        let (a, b) = (*a, *b);

        Ok(match function {
            ElementwiseFunction::Max => a.max(b),
            ElementwiseFunction::Min => a.min(b),
            ElementwiseFunction::Mod => {
                if b.to_u64() == 0 {
                    return Err(EvalFailure::DivisionByZero);
                }

                T::from_u64(a.to_u64() % b.to_u64()).expect("a remainder fits its type")
            }
            _ => unreachable!("the f32 calls take their own path"),
        })
    })
}

/// Runs an arithmetic operator over unsigned operands through `u64`.
fn unsigned_binary<T: Unsigned>(
    operator: BinaryOperator,
    operands: [Operand<'_, T>; 2],
    entries: usize,
    width: usize,
) -> EvalResult<Vec<T>> {
    let overflow = || EvalFailure::Overflow {
        operation: operator.to_string(),
    };

    componentwise(operands, entries, width, |[a, b]| {
        let (a, b) = (a.to_u64(), b.to_u64());

        match operator {
            BinaryOperator::Add => T::from_u64(a + b).ok_or_else(overflow),
            BinaryOperator::Multiply => T::from_u64(a * b).ok_or_else(overflow),
            BinaryOperator::Subtract => a
                .checked_sub(b)
                .and_then(T::from_u64)
                .ok_or(EvalFailure::BelowZero { left: a, right: b }),
            BinaryOperator::Divide => {
                if b == 0 {
                    return Err(EvalFailure::DivisionByZero);
                }

                Ok(T::from_u64(a / b).expect("a quotient fits its type"))
            }
        }
    })
}

/// Compares two lifted values position by position.
fn compare_values(
    operator: ComparisonOperator,
    left: &Value,
    right: &Value,
    entries: usize,
    width: usize,
) -> EvalResult<Vec<bool>> {
    match left.scalar() {
        Scalar::F32 => componentwise([left.f32s(), right.f32s()], entries, width, |[a, b]| {
            Ok(compare(operator, a, b))
        }),
        Scalar::U8 => componentwise([left.u8s(), right.u8s()], entries, width, |[a, b]| {
            Ok(compare(operator, a, b))
        }),
        Scalar::U16 => componentwise([left.u16s(), right.u16s()], entries, width, |[a, b]| {
            Ok(compare(operator, a, b))
        }),
        Scalar::U32 => componentwise([left.u32s(), right.u32s()], entries, width, |[a, b]| {
            Ok(compare(operator, a, b))
        }),
        Scalar::String => componentwise(
            [left.strings(), right.strings()],
            entries,
            width,
            |[a, b]| Ok(compare(operator, a, b)),
        ),
        Scalar::Bool => componentwise([left.bools(), right.bools()], entries, width, |[a, b]| {
            Ok(compare(operator, a, b))
        }),
    }
}

/// One comparison.
fn compare<T: PartialOrd>(operator: ComparisonOperator, a: &T, b: &T) -> bool {
    match operator {
        ComparisonOperator::Equal => a == b,
        ComparisonOperator::Greater => a > b,
        ComparisonOperator::GreaterEqual => a >= b,
        ComparisonOperator::Less => a < b,
        ComparisonOperator::LessEqual => a <= b,
        ComparisonOperator::NotEqual => a != b,
    }
}

/// The literal as components.
fn number(value: NumberValue) -> Components {
    match value {
        NumberValue::F32(value) => Components::F32(vec![value]),
        NumberValue::U8(value) => Components::U8(vec![value]),
        NumberValue::U16(value) => Components::U16(vec![value]),
        NumberValue::U32(value) => Components::U32(vec![value]),
    }
}

/// A value of the settled type over the computed components.
fn build(output: Type, components: Components) -> Value {
    Value::new(output.domain, output.dimension, components)
        .expect("the evaluator fills every entry of the settled type")
}

/// The values, erroring where any is NaN or infinite.
fn finite(values: Vec<f32>, operation: &str) -> EvalResult<Vec<f32>> {
    if values.iter().all(|value| value.is_finite()) {
        Ok(values)
    } else {
        Err(EvalFailure::NonFinite {
            operation: operation.to_owned(),
        })
    }
}

/// The vector's length.
fn magnitude(vector: &[f32]) -> f32 {
    dot(vector, vector).sqrt()
}

/// The dot product.
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

/// A vec3 entry as an array.
fn triple(components: &[f32]) -> [f32; 3] {
    components.try_into().expect("the checker requires a vec3")
}

/// The operands as a fixed-arity array.
fn fixed<T: Copy, const N: usize>(operands: &[T]) -> [T; N] {
    operands.try_into().expect("the parser checks arity")
}

#[cfg(test)]
mod tests {
    use crate::{
        Components, Dimension, Domain, Error, EvalFailure, Scalar, Value, ValueEnvironment,
        evaluator::{
            assert_close, bools, buried, empty, evaluate, f32s, lamp, step, strings, u8s, u16s,
            u32s, wide_bools,
        },
    };

    fn value(text: &str) -> Value {
        match evaluate(text, &lamp()) {
            Ok(value) => value,
            Err(error) => panic!("{text} failed: {error}"),
        }
    }

    fn value_in(text: &str, environment: &ValueEnvironment) -> Value {
        match evaluate(text, environment) {
            Ok(value) => value,
            Err(error) => panic!("{text} failed: {error}"),
        }
    }

    fn close(text: &str, expected: &[f32]) {
        assert_close(&value(text), expected);
    }

    fn close_in(text: &str, environment: &ValueEnvironment, expected: &[f32]) {
        assert_close(&value_in(text, environment), expected);
    }

    fn failure(text: &str) -> EvalFailure {
        failure_in(text, &lamp())
    }

    fn failure_in(text: &str, environment: &ValueEnvironment) -> EvalFailure {
        match evaluate(text, environment) {
            Err(Error::Eval {
                binding: None,
                failure,
            }) => failure,
            other => panic!("{text} gave {other:?}"),
        }
    }

    fn non_finite(operation: &str) -> EvalFailure {
        EvalFailure::NonFinite {
            operation: operation.to_owned(),
        }
    }

    // Literals and names.

    #[test]
    fn literals_and_names_answer_their_values() {
        assert_eq!(value("0.5"), f32s(Domain::Plain, Dimension::Vec1, &[0.5]));
        assert_eq!(value("2u8"), u8s(Domain::Plain, Dimension::Vec1, &[2]));
        assert_eq!(
            value("count"),
            u32s(Domain::Swatch, Dimension::Vec1, &[3, 7])
        );
        assert_eq!(value("true"), bools(Domain::Plain, &[true]));
        assert_eq!(value("\"x\""), strings(Domain::Plain, &["x"]));
        assert_eq!(value("tag"), strings(Domain::Swatch, &["steel", "glass"]));
    }

    // Arithmetic.

    #[test]
    fn f32_arithmetic_pairs_entries_and_broadcasts() {
        close("roughnessFactor + 0.1", &[1.0, 0.5]);
        close("1 - roughnessFactor", &[0.1, 0.6]);
        close(
            "baseColorFactor * 2",
            &[1.0, 1.0, 1.0, 2.0, 2.0, 1.8, 1.2, 1.2],
        );
        close(
            "baseColorFactor.rg * baseColorFactor.ba",
            &[0.25, 0.5, 0.6, 0.54],
        );
        close(
            "baseColorFactor / roughnessFactor",
            &[
                0.5 / 0.9,
                0.5 / 0.9,
                0.5 / 0.9,
                1.0 / 0.9,
                2.5,
                2.25,
                1.5,
                1.5,
            ],
        );
        close("-roughnessFactor", &[-0.9, -0.4]);
        close("- -roughnessFactor", &[0.9, 0.4]);
        close("emissiveStrength / max(emissiveStrength)", &[0.0, 1.0]);
    }

    #[test]
    fn a_non_finite_f32_result_errors() {
        assert_eq!(failure("roughnessFactor / 0"), non_finite("/"));
        assert_eq!(failure("0 / emissiveStrength"), non_finite("/"));
        assert_eq!(
            failure("emissiveStrength / emissiveStrength"),
            non_finite("/")
        );
        assert_eq!(failure("pow(-1, 0.5)"), non_finite("pow"));
        assert_eq!(failure("normalize(rgb(0, 0, 0))"), non_finite("normalize"));
        assert_eq!(failure("mod(roughnessFactor, 0)"), non_finite("mod"));
        assert_eq!(failure("pow(10, 100)"), non_finite("pow"));
    }

    #[test]
    fn unsigned_arithmetic_is_exact_and_checked() {
        assert_eq!(
            value("count + 1"),
            u32s(Domain::Swatch, Dimension::Vec1, &[4, 8])
        );
        assert_eq!(
            value("count * 2"),
            u32s(Domain::Swatch, Dimension::Vec1, &[6, 14])
        );
        assert_eq!(
            value("count - 3"),
            u32s(Domain::Swatch, Dimension::Vec1, &[0, 4])
        );
        assert_eq!(
            value("count / 2"),
            u32s(Domain::Swatch, Dimension::Vec1, &[1, 3])
        );
        assert_eq!(
            value("mod(count, 2)"),
            u32s(Domain::Swatch, Dimension::Vec1, &[1, 1])
        );
        assert_eq!(
            value("wide / 2"),
            u8s(Domain::Voxel, Dimension::Vec2, &[100, 50, 25, 12])
        );
        assert_eq!(
            value("wide.y + wide.y"),
            u8s(Domain::Voxel, Dimension::Vec1, &[200, 50])
        );
        assert_eq!(
            value("10 - count"),
            u32s(Domain::Swatch, Dimension::Vec1, &[7, 3])
        );
        assert_eq!(
            failure("3 - count"),
            EvalFailure::BelowZero { left: 3, right: 7 }
        );
        assert_eq!(failure("count / 0"), EvalFailure::DivisionByZero);
        assert_eq!(failure("mod(count, 0)"), EvalFailure::DivisionByZero);
        assert_eq!(
            failure("wide.x + 100"),
            EvalFailure::Overflow {
                operation: "+".to_owned()
            }
        );
        assert_eq!(
            failure("wide.x * 2"),
            EvalFailure::Overflow {
                operation: "*".to_owned()
            }
        );
        assert_eq!(
            failure("wide.x - 250"),
            EvalFailure::BelowZero {
                left: 200,
                right: 250
            }
        );
    }

    #[test]
    fn mod_is_the_floored_remainder() {
        close("mod(roughnessFactor, 0.5)", &[0.4, 0.4]);
        close("mod(-0.25, 1)", &[0.75]);
        close("mod(2.5, 1)", &[0.5]);
        close("mod(emissiveStrength + 0.618, 1)", &[0.618, 0.618]);
    }

    // Domains.

    #[test]
    fn climbs_duplicate_entries_up_the_ladder() {
        close("voxel(roughnessFactor)", &[0.9, 0.4]);
        assert_eq!(
            value("face(count)"),
            u32s(
                Domain::Face,
                Dimension::Vec1,
                &[3, 3, 3, 3, 3, 7, 7, 7, 7, 7]
            )
        );
        assert_eq!(value("corner(emissiveStrength)").entries(), 40);
        close("face(0.5)", &[0.5; 10]);
        assert_eq!(
            value("swatch(2u32)"),
            u32s(Domain::Swatch, Dimension::Vec1, &[2, 2])
        );
        assert_eq!(value("swatch(roughnessFactor)"), value("roughnessFactor"));
        assert_eq!(
            value("face(tag)"),
            strings(
                Domain::Face,
                &["steel"; 5]
                    .into_iter()
                    .chain(["glass"; 5])
                    .collect::<Vec<_>>()
            )
        );
        assert_eq!(value("voxel(flag)"), bools(Domain::Voxel, &[false, true]));
        close("corner(unit)", &[1.0, 0.0, 0.0].repeat(40));
    }

    #[test]
    fn elementwise_operations_climb_the_lower_operand() {
        close(
            "roughnessFactor * faceValue",
            &[0.0, 0.9, 1.8, 2.7, 3.6, 2.0, 2.4, 2.8, 3.2, 3.6],
        );
        close(
            "computedOcclusion * emissiveStrength",
            &(0..40)
                .map(|corner| {
                    if corner < 20 {
                        0.0
                    } else {
                        corner as f32 / 10.0
                    }
                })
                .collect::<Vec<_>>(),
        );
        assert_eq!(
            value("voxelPosition.y + count"),
            u32s(Domain::Voxel, Dimension::Vec1, &[3, 8])
        );
        assert_eq!(
            value("flag && faceValue > 6").components(),
            &Components::Bool([false; 7].into_iter().chain([true; 3]).collect())
        );
    }

    #[test]
    fn a_climb_onto_a_merged_face_needs_agreeing_pieces() {
        let step = step();

        close_in("voxel(height)", &step, &[0.0, 0.0, 1.0]);
        close_in(
            "face(baseColorFactor)",
            &step,
            &[0.55, 0.5, 0.45, 1.0].repeat(10),
        );
        assert_eq!(
            failure_in("face(height)", &step),
            EvalFailure::ClimbDisagreement {
                target: Domain::Face,
                entry: 1
            }
        );
        assert_eq!(
            failure_in("corner(height) * 2", &step),
            EvalFailure::ClimbDisagreement {
                target: Domain::Face,
                entry: 1
            }
        );
    }

    // Reductions.

    #[test]
    fn the_plain_reductions_fold_the_whole_domain() {
        close("max(roughnessFactor)", &[0.9]);
        close("min(baseColorFactor)", &[0.5, 0.5, 0.5, 0.6]);
        close("max(baseColorFactor)", &[1.0, 0.9, 0.6, 1.0]);
        close("avg(baseColorFactor.rg)", &[0.75, 0.7]);
        close("sum(faceValue)", &[45.0]);
        close("avg(computedOcclusion)", &[19.5 / 40.0]);
        assert_eq!(
            value("sum(count)"),
            u32s(Domain::Plain, Dimension::Vec1, &[10])
        );
        assert_eq!(
            value("max(count)"),
            u32s(Domain::Plain, Dimension::Vec1, &[7])
        );
        assert_eq!(
            value("min(wide)"),
            u8s(Domain::Plain, Dimension::Vec2, &[50, 25])
        );
        close("avg(count)", &[5.0]);
        close("avg(wide)", &[125.0, 62.5]);
    }

    #[test]
    fn face_reductions_gather_four_corners() {
        close(
            "faceAvg(computedOcclusion)",
            &(0..10)
                .map(|face| (4 * face) as f32 / 40.0 + 1.5 / 40.0)
                .collect::<Vec<_>>(),
        );
        close(
            "faceSum(computedOcclusion)",
            &(0..10)
                .map(|face| (16 * face + 6) as f32 / 40.0)
                .collect::<Vec<_>>(),
        );
        close(
            "faceMin(computedOcclusion)",
            &(0..10)
                .map(|face| (4 * face) as f32 / 40.0)
                .collect::<Vec<_>>(),
        );
        close(
            "faceMax(computedOcclusion)",
            &(0..10)
                .map(|face| (4 * face + 3) as f32 / 40.0)
                .collect::<Vec<_>>(),
        );
    }

    #[test]
    fn voxel_and_swatch_reductions_read_through_the_pieces() {
        assert_eq!(
            value("voxelSum(face(1u32))"),
            u32s(Domain::Voxel, Dimension::Vec1, &[5, 5])
        );
        close("voxelAvg(faceValue)", &[2.0, 7.0]);
        close("voxelMax(faceValue)", &[4.0, 9.0]);
        close("voxelMin(computedOcclusion)", &[0.0, 0.5]);
        assert_eq!(
            value("swatchSum(face(1u32))"),
            u32s(Domain::Swatch, Dimension::Vec1, &[5, 5])
        );
        close("swatchAvg(faceValue)", &[2.0, 7.0]);
        assert_eq!(
            value("swatchMin(voxelPosition)"),
            u32s(Domain::Swatch, Dimension::Vec3, &[0, 0, 0, 0, 1, 0])
        );
        close("swatchAvg(voxelPosition)", &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        close(
            "swatchSum(computedOcclusion)",
            &[190.0 / 40.0, 590.0 / 40.0],
        );
        close(
            "swatchMax(faceAvg(computedOcclusion))",
            &[17.5 / 40.0, 37.5 / 40.0],
        );
        assert_eq!(
            value("voxelSum(face(count))"),
            u32s(Domain::Voxel, Dimension::Vec1, &[15, 35])
        );
        close("swatchMin(voxelMin(computedOcclusion))", &[0.0, 0.5]);
    }

    #[test]
    fn a_merged_face_counts_once_per_piece() {
        let step = step();

        assert_eq!(
            value_in("voxelSum(face(1u32))", &step),
            u32s(Domain::Voxel, Dimension::Vec1, &[4, 5, 5])
        );
        assert_eq!(
            value_in("swatchSum(face(1u32))", &step),
            u32s(Domain::Swatch, Dimension::Vec1, &[14])
        );
        close_in("voxelAvg(faceValue)", &step, &[1.75, 4.6, 4.4]);
        close_in("swatchAvg(faceValue)", &step, &[52.0 / 14.0]);
        close_in("swatchMin(faceValue)", &step, &[0.0]);
        close_in("swatchMax(faceValue)", &step, &[9.0]);
        close_in("voxelSum(computedOcclusion)[0]", &step, &[136.0]);
        close_in(
            "swatchAvg(voxelAvg(faceValue))",
            &step,
            &[(1.75 + 4.6 + 4.4) / 3.0],
        );
    }

    #[test]
    fn an_empty_destination_errors_under_min_max_and_avg_and_sums_to_zero() {
        let buried = buried();

        close_in("voxelSum(faceValue)", &buried, &[6.0, 0.0]);
        close_in("swatchSum(faceValue)", &buried, &[6.0, 0.0]);
        assert_eq!(
            value_in("voxelSum(face(1u32))", &buried),
            u32s(Domain::Voxel, Dimension::Vec1, &[3, 0])
        );
        assert_eq!(
            failure_in("voxelMin(faceValue)", &buried),
            EvalFailure::EmptyDestination {
                operation: "voxelMin".to_owned(),
                target: Domain::Voxel,
                entry: 1
            }
        );
        assert_eq!(
            failure_in("swatchAvg(faceValue)", &buried),
            EvalFailure::EmptyDestination {
                operation: "swatchAvg".to_owned(),
                target: Domain::Swatch,
                entry: 1
            }
        );
        assert_eq!(
            failure_in("swatchMax(corner(faceValue))", &buried),
            EvalFailure::EmptyDestination {
                operation: "swatchMax".to_owned(),
                target: Domain::Swatch,
                entry: 1
            }
        );

        let empty = empty();

        close_in("sum(faceValue)", &empty, &[0.0]);
        assert_eq!(value_in("face(1u32)", &empty).entries(), 0);
        assert_eq!(
            failure_in("max(faceValue)", &empty),
            EvalFailure::EmptyDestination {
                operation: "max".to_owned(),
                target: Domain::Plain,
                entry: 0
            }
        );
        assert_eq!(
            failure_in("avg(faceValue)", &empty),
            EvalFailure::EmptyDestination {
                operation: "avg".to_owned(),
                target: Domain::Plain,
                entry: 0
            }
        );
    }

    #[test]
    fn a_sum_that_overflows_its_type_errors() {
        assert_eq!(
            failure("sum(face(wide))"),
            EvalFailure::Overflow {
                operation: "sum".to_owned()
            }
        );
        assert_eq!(
            failure("voxelSum(face(wide))"),
            EvalFailure::Overflow {
                operation: "voxelSum".to_owned()
            }
        );
        assert_eq!(
            failure("voxelSum(face(wide.y))"),
            EvalFailure::Overflow {
                operation: "voxelSum".to_owned()
            }
        );
        assert_eq!(
            value("voxelSum(face(wide.y / 10))"),
            u8s(Domain::Voxel, Dimension::Vec1, &[50, 10])
        );
    }

    // Indexing and swizzles.

    #[test]
    fn indexing_samples_one_entry() {
        close("baseColorFactor[1]", &[1.0, 0.9, 0.6, 0.6]);
        close("baseColorFactor[1].rgb", &[1.0, 0.9, 0.6]);
        close("baseColorFactor.rgb[1]", &[1.0, 0.9, 0.6]);
        close("faceValue[9u16]", &[9.0]);
        assert_eq!(value("tag[1u8]"), strings(Domain::Plain, &["glass"]));
        assert_eq!(value("flag[0]"), bools(Domain::Plain, &[false]));
        assert_eq!(
            value("count[1]"),
            u32s(Domain::Plain, Dimension::Vec1, &[7])
        );
        assert_eq!(
            failure("count[2]"),
            EvalFailure::IndexOutOfRange {
                index: 2,
                entries: 2
            }
        );
        assert_eq!(
            failure("computedOcclusion[40]"),
            EvalFailure::IndexOutOfRange {
                index: 40,
                entries: 40
            }
        );
    }

    #[test]
    fn swizzles_pick_and_repeat_components() {
        close("baseColorFactor.a", &[1.0, 0.6]);
        close("baseColorFactor.bgr", &[0.5, 0.5, 0.5, 0.6, 0.9, 1.0]);
        close("roughnessFactor.rr", &[0.9, 0.9, 0.4, 0.4]);
        close("0.5.rrr", &[0.5, 0.5, 0.5]);
        close("unit.zyx", &[0.0, 0.0, 1.0]);
        assert_eq!(
            value("wide.yx"),
            u8s(Domain::Voxel, Dimension::Vec2, &[100, 200, 25, 50])
        );
        assert_eq!(
            value("voxelPosition.y"),
            u32s(Domain::Voxel, Dimension::Vec1, &[0, 1])
        );
        assert_eq!(
            value("(baseColorFactor > 0.55).b"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(
            value("(unit == 0).zyx"),
            wide_bools(Domain::Plain, Dimension::Vec3, &[true, true, false])
        );
    }

    // Booleans and strings.

    #[test]
    fn comparisons_answer_per_entry() {
        assert_eq!(
            value("roughnessFactor > 0.5"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(
            value("roughnessFactor >= 0.9"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(
            value("roughnessFactor < 0.5"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(
            value("roughnessFactor <= 0.4"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(
            value("emissiveStrength == 0"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(
            value("emissiveStrength != 0"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(value("count == 7"), bools(Domain::Swatch, &[false, true]));
        assert_eq!(value("wide.x > 100"), bools(Domain::Voxel, &[true, false]));
        assert_eq!(
            value("tag == \"glass\""),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(
            value("\"glass\" != tag"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(value("\"a\" == \"a\""), bools(Domain::Plain, &[true]));
        assert_eq!(
            value("faceValue < roughnessFactor"),
            bools(
                Domain::Face,
                &[
                    true, false, false, false, false, false, false, false, false, false
                ]
            )
        );
        assert_eq!(
            value("roughnessFactor == 0.9"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(value("flag == true"), bools(Domain::Swatch, &[false, true]));
        assert_eq!(
            value("(unit == 0) != true"),
            wide_bools(Domain::Plain, Dimension::Vec3, &[true, false, false])
        );
    }

    #[test]
    fn a_wide_comparison_answers_per_component() {
        assert_eq!(
            value("baseColorFactor.rgb > 0.55"),
            wide_bools(
                Domain::Swatch,
                Dimension::Vec3,
                &[false, false, false, true, true, true]
            )
        );
        assert_eq!(
            value("0.7 < baseColorFactor"),
            wide_bools(
                Domain::Swatch,
                Dimension::Vec4,
                &[false, false, false, true, true, true, false, false]
            )
        );
        assert_eq!(
            value("wide == wide.yx"),
            wide_bools(
                Domain::Voxel,
                Dimension::Vec2,
                &[false, false, false, false]
            )
        );
        assert_eq!(
            value("unit == 0"),
            wide_bools(Domain::Plain, Dimension::Vec3, &[false, true, true])
        );
    }

    #[test]
    fn folds_answer_per_entry_over_components() {
        assert_eq!(
            value("all(baseColorFactor.rgb > 0.4)"),
            bools(Domain::Swatch, &[true, true])
        );
        assert_eq!(
            value("all(baseColorFactor.rgb > 0.55)"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(
            value("any(baseColorFactor.rgb < 0.7)"),
            bools(Domain::Swatch, &[true, true])
        );
        assert_eq!(
            value("any(baseColorFactor == 0.5)"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(
            value("all(baseColorFactor == baseColorFactor)"),
            bools(Domain::Swatch, &[true, true])
        );
        assert_eq!(
            value("all(tag == \"steel\")"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(
            value("any(voxelPosition > 0)"),
            bools(Domain::Voxel, &[false, true])
        );
        assert_eq!(value("all(0.5 < unit)"), bools(Domain::Plain, &[false]));
        assert_eq!(value("any(flag)"), bools(Domain::Swatch, &[false, true]));
        assert_eq!(
            value("all(!(baseColorFactor.rgb > 0.55))"),
            bools(Domain::Swatch, &[true, false])
        );
        assert_eq!(value("any(true)"), bools(Domain::Plain, &[true]));
    }

    #[test]
    fn the_logical_operators_combine_bools() {
        assert_eq!(value("!flag"), bools(Domain::Swatch, &[true, false]));
        assert_eq!(value("flag && true"), bools(Domain::Swatch, &[false, true]));
        assert_eq!(
            value("flag || false"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(value("flag ^ true"), bools(Domain::Swatch, &[true, false]));
        assert_eq!(
            value("flag && count > 5"),
            bools(Domain::Swatch, &[false, true])
        );
        assert_eq!(value("!flag || flag"), bools(Domain::Swatch, &[true, true]));
        assert_eq!(value("!true && false"), bools(Domain::Plain, &[false]));
        assert_eq!(
            value("!(unit == 0)"),
            wide_bools(Domain::Plain, Dimension::Vec3, &[true, false, false])
        );
        assert_eq!(
            value("unit == 0 && true"),
            wide_bools(Domain::Plain, Dimension::Vec3, &[false, true, true])
        );
        assert_eq!(
            value("baseColorFactor.rgb > 0.55 || flag"),
            wide_bools(
                Domain::Swatch,
                Dimension::Vec3,
                &[false, false, false, true, true, true]
            )
        );
        assert_eq!(
            value("baseColorFactor.rgb > 0.55 ^ baseColorFactor.rgb > 0.8"),
            wide_bools(
                Domain::Swatch,
                Dimension::Vec3,
                &[false, false, false, false, false, true]
            )
        );
    }

    #[test]
    fn mix_picks_per_entry_by_the_bool() {
        close("mix(0f32, 1, flag)", &[0.0, 1.0]);
        close(
            "mix(baseColorFactor, 0.5.rrrr, flag)",
            &[0.5, 0.5, 0.5, 1.0, 0.5, 0.5, 0.5, 0.5],
        );
        assert_eq!(
            value("mix(count, 0, flag)"),
            u32s(Domain::Swatch, Dimension::Vec1, &[3, 0])
        );
        assert_eq!(
            value("mix(\"a\", tag, flag)"),
            strings(Domain::Swatch, &["a", "glass"])
        );
        assert_eq!(
            value("mix(false, flag, true)"),
            bools(Domain::Swatch, &[false, true])
        );
        close(
            "mix(0.5, faceValue, flag)",
            &[0.5, 0.5, 0.5, 0.5, 0.5, 5.0, 6.0, 7.0, 8.0, 9.0],
        );
        assert_eq!(
            value("mix(\"OPAQUE\", \"BLEND\", min(baseColorFactor.a) < 1)"),
            strings(Domain::Plain, &["BLEND"])
        );
        close(
            "mix(baseColorFactor, 0.rrrr, baseColorFactor > 0.55)",
            &[0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0],
        );
        assert_eq!(
            value("mix(wide, wide * 0, wide > 60)"),
            u8s(Domain::Voxel, Dimension::Vec2, &[0, 0, 50, 25])
        );
    }

    #[test]
    fn default_reads_the_name_or_the_fallback() {
        close("default(roughnessFactor, 1)", &[0.9, 0.4]);
        close("default(missing, 1)", &[1.0]);
        close("default(missing, rgb(0, 0, 0))", &[0.0, 0.0, 0.0]);
        close("swatch(default(missing, 1))", &[1.0, 1.0]);
        close(
            "default(roughnessFactor, faceValue)",
            &[0.9, 0.9, 0.9, 0.9, 0.9, 0.4, 0.4, 0.4, 0.4, 0.4],
        );
        assert_eq!(
            value("default(count, 1)"),
            u32s(Domain::Swatch, Dimension::Vec1, &[3, 7])
        );
        assert_eq!(
            value("default(missing, \"x\")"),
            strings(Domain::Plain, &["x"])
        );
        assert_eq!(
            value("default(tag, \"x\")"),
            strings(Domain::Swatch, &["steel", "glass"])
        );
    }

    // Functions.

    #[test]
    fn the_constructors_pack_vec1_parts() {
        close("r(roughnessFactor)", &[0.9, 0.4]);
        close("rg(roughnessFactor, metallicFactor)", &[0.9, 1.0, 0.4, 0.0]);
        close(
            "rgb(metallicFactor, roughnessFactor, 0)",
            &[1.0, 0.9, 0.0, 0.0, 0.4, 0.0],
        );
        close("rgba(1, 1, 1, 1)", &[1.0, 1.0, 1.0, 1.0]);
        close("rgb(0.5, faceValue, 1)[9]", &[0.5, 9.0, 1.0]);
        assert_eq!(
            value("rgb(flag, true, flag)"),
            wide_bools(
                Domain::Swatch,
                Dimension::Vec3,
                &[false, true, false, true, true, true]
            )
        );
        assert_eq!(
            value("rg(unit.x == 1, false)"),
            wide_bools(Domain::Plain, Dimension::Vec2, &[true, false])
        );
    }

    #[test]
    fn min_and_max_are_elementwise_in_their_binary_forms() {
        close("max(emissiveStrength, 0.001)", &[0.001, 4.0]);
        close(
            "min(baseColorFactor, 0.6)",
            &[0.5, 0.5, 0.5, 0.6, 0.6, 0.6, 0.6, 0.6],
        );
        close(
            "max(0.6, baseColorFactor)",
            &[0.6, 0.6, 0.6, 1.0, 1.0, 0.9, 0.6, 0.6],
        );
        assert_eq!(
            value("max(count, 5)"),
            u32s(Domain::Swatch, Dimension::Vec1, &[5, 7])
        );
        assert_eq!(
            value("min(wide, 60)"),
            u8s(Domain::Voxel, Dimension::Vec2, &[60, 60, 50, 25])
        );
        close("max(computedOcclusion, 0.2)[0]", &[0.2]);
    }

    #[test]
    fn abs_and_the_rounding_forms_are_componentwise() {
        close("abs(roughnessFactor - 1)", &[0.1, 0.6]);
        close("abs(rgb(-1, 2, -3))", &[1.0, 2.0, 3.0]);
        close("floor(roughnessFactor * 4)", &[3.0, 1.0]);
        close("ceil(roughnessFactor * 4)", &[4.0, 2.0]);
        close("round(roughnessFactor * 4)", &[4.0, 2.0]);
        close("round(2.5)", &[3.0]);
        close("round(-2.5)", &[-3.0]);
        close("round(0.4)", &[0.0]);
        close("floor(-0.5)", &[-1.0]);
        close("ceil(-0.5)", &[0.0]);
        close("round(roughnessFactor * 4) / 4", &[1.0, 0.5]);
    }

    #[test]
    fn the_vector_functions_fold_each_entry() {
        close("dot(unit, unit)", &[1.0]);
        close("dot(baseColorFactor.rgb, unit)", &[0.5, 1.0]);
        close("dot(roughnessFactor, metallicFactor)", &[0.9, 0.0]);
        close("length(rgb(3, 4, 0))", &[5.0]);
        close("length(-2)", &[2.0]);
        close(
            "length(baseColorFactor.rg)",
            &[0.5f32.hypot(0.5), 1.0f32.hypot(0.9)],
        );
        close("distance(unit, rgb(0, 0, 0))", &[1.0]);
        close(
            "distance(baseColorFactor.rgb, unit)",
            &[(0.25f32 + 0.25 + 0.25).sqrt(), (0.81f32 + 0.36).sqrt()],
        );
        close("normalize(rgb(3, 4, 0))", &[0.6, 0.8, 0.0]);
        close("normalize(unit * 5)", &[1.0, 0.0, 0.0]);
        close("cross(unit, rgb(0, 1, 0))", &[0.0, 0.0, 1.0]);
        close("cross(rgb(0, 1, 0), unit)", &[0.0, 0.0, -1.0]);
    }

    #[test]
    fn pow_lerp_step_clamp_and_smoothstep_follow_their_formulas() {
        close("pow(roughnessFactor, 2)", &[0.81, 0.16]);
        close("pow(rgb(4, 9, 16), 0.5)", &[2.0, 3.0, 4.0]);
        close("pow(2, rgb(1, 2, 3).x)", &[2.0]);
        close("lerp(0, 10, 0.25)", &[2.5]);
        close("lerp(0, 10, 1.5)", &[15.0]);
        close("lerp(1, computedOcclusion, 0.8)[0]", &[0.2]);
        close(
            "lerp(baseColorFactor.rgb, rgb(1, 1, 1), 0.5)",
            &[0.75, 0.75, 0.75, 1.0, 0.95, 0.8],
        );
        close("step(0.001, emissiveStrength)", &[0.0, 1.0]);
        close("step(0.5, baseColorFactor)", &[1.0; 8]);
        close("step(baseColorFactor.r, baseColorFactor.g)", &[1.0, 0.0]);
        close("clamp(emissiveStrength, 1, 3)", &[1.0, 3.0]);
        close(
            "clamp(baseColorFactor, 0.55, 0.95)",
            &[0.55, 0.55, 0.55, 0.95, 0.95, 0.9, 0.6, 0.6],
        );
        close("smoothstep(0, 1, 0.5)", &[0.5]);
        close("smoothstep(0, 1, roughnessFactor)", &[0.972, 0.352]);
        close("smoothstep(0, 1, 2)", &[1.0]);
        close("smoothstep(0, 1, -1)", &[0.0]);
        close("smoothstep(0.2, 0.8, computedOcclusion)[20]", &[0.5]);
    }

    #[test]
    fn inverted_bounds_error() {
        assert_eq!(
            failure("clamp(emissiveStrength, 1, 0.5)"),
            EvalFailure::InvertedBounds {
                operation: "clamp".to_owned(),
                low: 1.0,
                high: 0.5
            }
        );
        assert_eq!(
            failure("smoothstep(1, 1, 0.5)"),
            EvalFailure::InvertedBounds {
                operation: "smoothstep".to_owned(),
                low: 1.0,
                high: 1.0
            }
        );
        close("clamp(0.5, 1, 1)", &[1.0]);
    }

    #[test]
    fn exact_conversions_error_on_a_fraction_or_a_range_miss() {
        close("f32(count)", &[3.0, 7.0]);
        close("f32(wide)", &[200.0, 100.0, 50.0, 25.0]);
        close("f32(16777216u32)", &[16777216.0]);
        close("f32(voxelPosition.y) * 0.5", &[0.0, 0.5]);
        assert_eq!(
            value("u8(emissiveStrength)"),
            u8s(Domain::Swatch, Dimension::Vec1, &[0, 4])
        );
        assert_eq!(
            value("u8(count)"),
            u8s(Domain::Swatch, Dimension::Vec1, &[3, 7])
        );
        assert_eq!(
            value("u16(wide)"),
            u16s(Domain::Voxel, Dimension::Vec2, &[200, 100, 50, 25])
        );
        assert_eq!(
            value("u32(wide.x)"),
            u32s(Domain::Voxel, Dimension::Vec1, &[200, 50])
        );
        assert_eq!(
            value("u32(2.0)"),
            u32s(Domain::Plain, Dimension::Vec1, &[2])
        );
        assert_eq!(
            value("u8(255.0)"),
            u8s(Domain::Plain, Dimension::Vec1, &[255])
        );
        assert_eq!(
            failure("f32(16777217u32)"),
            EvalFailure::Inexact { value: 16777217 }
        );
        assert_eq!(
            failure("u8(roughnessFactor)"),
            EvalFailure::Fraction {
                value: 0.9,
                target: Scalar::U8
            }
        );
        assert_eq!(
            failure("u8(emissiveStrength * 100)"),
            EvalFailure::OutOfRange {
                value: 400.0,
                target: Scalar::U8
            }
        );
        assert_eq!(
            failure("u8(-1)"),
            EvalFailure::OutOfRange {
                value: -1.0,
                target: Scalar::U8
            }
        );
        assert_eq!(
            failure("u8(300u32)"),
            EvalFailure::OutOfRange {
                value: 300.0,
                target: Scalar::U8
            }
        );
        assert_eq!(
            failure("u16(70000u32)"),
            EvalFailure::OutOfRange {
                value: 70000.0,
                target: Scalar::U16
            }
        );
        assert_eq!(
            failure("u32(4294967296.0)"),
            EvalFailure::OutOfRange {
                value: 4294967296.0,
                target: Scalar::U32
            }
        );
    }

    #[test]
    fn the_rounding_conversions_round_by_their_mode_into_their_range() {
        assert_eq!(
            value("round_u8(roughnessFactor)"),
            u8s(Domain::Swatch, Dimension::Vec1, &[1, 0])
        );
        assert_eq!(
            value("ceil_u8(roughnessFactor)"),
            u8s(Domain::Swatch, Dimension::Vec1, &[1, 1])
        );
        assert_eq!(
            value("floor_u8(roughnessFactor)"),
            u8s(Domain::Swatch, Dimension::Vec1, &[0, 0])
        );
        assert_eq!(
            value("round_u16(computedOcclusion * 65535)[20]"),
            u16s(Domain::Plain, Dimension::Vec1, &[32768])
        );
        assert_eq!(
            value("round_u32(2.5)"),
            u32s(Domain::Plain, Dimension::Vec1, &[3])
        );
        assert_eq!(
            value("ceil_u16(0.5)"),
            u16s(Domain::Plain, Dimension::Vec1, &[1])
        );
        assert_eq!(
            value("floor_u32(2.9)"),
            u32s(Domain::Plain, Dimension::Vec1, &[2])
        );
        assert_eq!(
            value("ceil_u32(-0.5)"),
            u32s(Domain::Plain, Dimension::Vec1, &[0])
        );
        assert_eq!(
            value("round_u8(255.4)"),
            u8s(Domain::Plain, Dimension::Vec1, &[255])
        );
        assert_eq!(
            failure("round_u8(emissiveStrength * 100)"),
            EvalFailure::OutOfRange {
                value: 400.0,
                target: Scalar::U8
            }
        );
        assert_eq!(
            failure("floor_u32(-0.5)"),
            EvalFailure::OutOfRange {
                value: -1.0,
                target: Scalar::U32
            }
        );
        assert_eq!(
            failure("round_u8(255.5)"),
            EvalFailure::OutOfRange {
                value: 256.0,
                target: Scalar::U8
            }
        );
    }

    #[test]
    fn the_color_conversions_visit_oklab_and_oklch() {
        let white = value("oklabFromRgb(rgb(1, 1, 1))");
        let Components::F32(white) = white.components() else {
            panic!()
        };

        assert!((white[0] - 1.0).abs() < 1e-4, "{white:?}");
        assert!(white[1].abs() < 1e-4 && white[2].abs() < 1e-4, "{white:?}");
        close("oklabFromRgb(rgb(0, 0, 0))", &[0.0, 0.0, 0.0]);

        let red = value("oklabFromRgb(unit)");
        let Components::F32(red) = red.components() else {
            panic!()
        };

        assert!((red[0] - 0.628).abs() < 1e-3, "{red:?}");
        assert!((red[1] - 0.2249).abs() < 1e-3, "{red:?}");
        assert!((red[2] - 0.1258).abs() < 1e-3, "{red:?}");

        let round_trip = value("rgbFromOklab(oklabFromRgb(baseColorFactor.rgb))");

        assert_close(&round_trip, &[0.5, 0.5, 0.5, 1.0, 0.9, 0.6]);
        close("oklchFromRgb(rgb(0.5, 0.5, 0.5)).yz", &[0.0, 0.0]);
        close("oklchFromRgb(rgb(0, 0, 0))", &[0.0, 0.0, 0.0]);

        let red = value("oklchFromRgb(unit)");
        let Components::F32(red) = red.components() else {
            panic!()
        };

        assert!((red[1] - 0.2577).abs() < 1e-3, "{red:?}");
        assert!((red[2] - 29.23 / 360.0).abs() < 1e-3, "{red:?}");

        let round_trip = value("rgbFromOklch(oklchFromRgb(baseColorFactor.rgb))");

        assert_close(&round_trip, &[0.5, 0.5, 0.5, 1.0, 0.9, 0.6]);

        let blue = value("oklchFromRgb(rgb(0, 0, 1)).z");
        let Components::F32(blue) = blue.components() else {
            panic!()
        };

        assert!((blue[0] - 264.05 / 360.0).abs() < 1e-3, "{blue:?}");

        let shifted = value("rgbFromOklch(oklchFromRgb(unit) + rgb(0, 0, 0.5))");

        assert_eq!(shifted.dimension(), Dimension::Vec3);
        assert_eq!(
            failure("rgbFromOklch(rgb(0.5, 0.1, 1.5))"),
            EvalFailure::HueRange { hue: 1.5 }
        );
        assert_eq!(
            failure("rgbFromOklch(rgb(0.5, 0.1, -0.1))"),
            EvalFailure::HueRange { hue: -0.1 }
        );
        assert_eq!(
            failure("rgbFromOklch(rgb(0.5, -0.1, 0.5))"),
            EvalFailure::Chroma { chroma: -0.1 }
        );
        assert_eq!(
            value("rgbFromOklch(rgb(0.5, 0, 1))"),
            value("rgbFromOklch(rgb(0.5, 0, 0))")
        );
    }
}
