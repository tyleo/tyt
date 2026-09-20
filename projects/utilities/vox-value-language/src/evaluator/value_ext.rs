use crate::{Components, Value, evaluator::Operand};

/// Reads a value's components as the type the checker settled.
pub(crate) trait ValueExt {
    /// The `f32` components.
    fn f32s(&self) -> Operand<'_, f32>;

    /// The `u8` components.
    fn u8s(&self) -> Operand<'_, u8>;

    /// The `u16` components.
    fn u16s(&self) -> Operand<'_, u16>;

    /// The `u32` components.
    fn u32s(&self) -> Operand<'_, u32>;

    /// The bool components.
    fn bools(&self) -> Operand<'_, bool>;

    /// The string components.
    fn strings(&self) -> Operand<'_, String>;
}

impl ValueExt for Value {
    fn f32s(&self) -> Operand<'_, f32> {
        let Components::F32(components) = self.components() else {
            unreachable!("the checker settled f32");
        };

        operand(components, self)
    }

    fn u8s(&self) -> Operand<'_, u8> {
        let Components::U8(components) = self.components() else {
            unreachable!("the checker settled u8");
        };

        operand(components, self)
    }

    fn u16s(&self) -> Operand<'_, u16> {
        let Components::U16(components) = self.components() else {
            unreachable!("the checker settled u16");
        };

        operand(components, self)
    }

    fn u32s(&self) -> Operand<'_, u32> {
        let Components::U32(components) = self.components() else {
            unreachable!("the checker settled u32");
        };

        operand(components, self)
    }

    fn bools(&self) -> Operand<'_, bool> {
        let Components::Bool(components) = self.components() else {
            unreachable!("the checker settled bool");
        };

        operand(components, self)
    }

    fn strings(&self) -> Operand<'_, String> {
        let Components::String(components) = self.components() else {
            unreachable!("the checker settled string");
        };

        operand(components, self)
    }
}

/// The components as an operand of the value's width.
fn operand<'a, T>(components: &'a [T], value: &Value) -> Operand<'a, T> {
    Operand {
        components,
        width: value.dimension().width(),
    }
}
