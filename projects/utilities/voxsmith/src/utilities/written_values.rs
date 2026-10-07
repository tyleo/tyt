use crate::{
    Result,
    utilities::{check_material_range, value_pool_kind_name},
};
use branded_id::U32Id;
use meshdoc::material::{COLOR_RANGE, MaterialPropertyKind, scalar_range};
use std::result::Result as StdResult;
use vox_value_language::{Components, Dimension, Domain, Value};
use voxcore::{
    BVoxValuePool, BVoxValuePoolValue, Result as VoxResult, VoxExt, VoxMain, VoxValueColumn,
    VoxValuePool, VoxValuePoolValues,
};

/// A `VoxMain::retain_*_value` appending one value to a value pool.
type RetainValue<T, V> =
    fn(&mut VoxMain<T>, U32Id<BVoxValuePool>, V) -> VoxResult<U32Id<BVoxValuePoolValue>>;

/// A write's value for each material of a palette, typed by the value pool
/// kind that holds it.
#[derive(Clone, Debug, PartialEq)]
pub enum WrittenValues {
    /// Values for a `bool` value pool.
    Bool(Vec<bool>),

    /// Values for a `float` value pool.
    Float(Vec<f64>),

    /// Values for an `int` value pool.
    Int(Vec<i64>),

    /// Values for a `string` value pool.
    String(Vec<String>),

    /// Values for a `vec-2-float` value pool.
    Vec2Float(Vec<[f64; 2]>),

    /// Values for a `vec-2-int` value pool.
    Vec2Int(Vec<[i64; 2]>),

    /// Values for a `vec-3-float` value pool.
    Vec3Float(Vec<[f64; 3]>),

    /// Values for a `vec-3-int` value pool.
    Vec3Int(Vec<[i64; 3]>),

    /// Values for a `vec-4-float` value pool.
    Vec4Float(Vec<[f64; 4]>),

    /// Values for a `vec-4-int` value pool.
    Vec4Int(Vec<[i64; 4]>),
}

impl WrittenValues {
    /// The values `value` lands on `material_count` materials. A plain value
    /// lands on every material. A swatch array lands one entry per material.
    /// Errors on another domain or a bool wider than vec1, with a reason that
    /// follows the write's element.
    pub fn of(value: Value, material_count: usize) -> StdResult<Self, String> {
        let domain = value.domain();
        let dimension = value.dimension();

        let entries = match domain {
            Domain::Plain => material_count,

            Domain::Swatch => value.entries(),

            Domain::Voxel | Domain::Face | Domain::Corner => {
                return Err(format!(
                    "evaluates to a {domain} value, and a write takes a plain or swatch value"
                ));
            }
        };

        let width = dimension.width();

        let values = match value.into_components() {
            Components::Bool(flags) if dimension == Dimension::Vec1 => {
                WrittenValues::Bool(cycled(flags, entries))
            }

            Components::Bool(_) => {
                return Err(format!(
                    "evaluates to bool {dimension}, which no value pool kind holds"
                ));
            }

            Components::F64(components) => floats(cycled(components, entries * width), dimension),

            Components::String(texts) => WrittenValues::String(cycled(texts, entries)),

            Components::U8(components) => ints(
                cycled(components, entries * width)
                    .into_iter()
                    .map(i64::from)
                    .collect(),
                dimension,
            ),

            Components::U16(components) => ints(
                cycled(components, entries * width)
                    .into_iter()
                    .map(i64::from)
                    .collect(),
                dimension,
            ),

            Components::U32(components) => ints(
                cycled(components, entries * width)
                    .into_iter()
                    .map(i64::from)
                    .collect(),
                dimension,
            ),
        };

        Ok(values)
    }

    /// An empty value pool of the values' kind.
    pub fn empty_value_pool(&self) -> VoxValuePool {
        let empty = "an empty value pool holds no value outside its domain";

        match self {
            WrittenValues::Bool(_) => VoxValuePool::boolean(Vec::new()),
            WrittenValues::Float(_) => VoxValuePool::float(Vec::new()).expect(empty),
            WrittenValues::Int(_) => VoxValuePool::int(Vec::new()).expect(empty),
            WrittenValues::String(_) => VoxValuePool::string(Vec::new()),
            WrittenValues::Vec2Float(_) => VoxValuePool::vec_2_float(Vec::new()).expect(empty),
            WrittenValues::Vec2Int(_) => VoxValuePool::vec_2_int(Vec::new()).expect(empty),
            WrittenValues::Vec3Float(_) => VoxValuePool::vec_3_float(Vec::new()).expect(empty),
            WrittenValues::Vec3Int(_) => VoxValuePool::vec_3_int(Vec::new()).expect(empty),
            WrittenValues::Vec4Float(_) => VoxValuePool::vec_4_float(Vec::new()).expect(empty),
            WrittenValues::Vec4Int(_) => VoxValuePool::vec_4_int(Vec::new()).expect(empty),
        }
    }

    /// Errors unless `value_pool` holds the values' kind, with a reason that
    /// follows the write's element.
    pub fn check_fits(&self, value_pool: &VoxValuePool) -> StdResult<(), String> {
        let fits = matches!(
            (self, value_pool.values()),
            (WrittenValues::Bool(_), VoxValuePoolValues::Bool(_))
                | (WrittenValues::Float(_), VoxValuePoolValues::Float(_))
                | (WrittenValues::Int(_), VoxValuePoolValues::Int(_))
                | (WrittenValues::String(_), VoxValuePoolValues::String(_))
                | (
                    WrittenValues::Vec2Float(_),
                    VoxValuePoolValues::Vec2Float(_)
                )
                | (WrittenValues::Vec2Int(_), VoxValuePoolValues::Vec2Int(_))
                | (
                    WrittenValues::Vec3Float(_),
                    VoxValuePoolValues::Vec3Float(_)
                )
                | (WrittenValues::Vec3Int(_), VoxValuePoolValues::Vec3Int(_))
                | (
                    WrittenValues::Vec4Float(_),
                    VoxValuePoolValues::Vec4Float(_)
                )
                | (WrittenValues::Vec4Int(_), VoxValuePoolValues::Vec4Int(_))
        );

        if fits {
            return Ok(());
        }

        Err(format!(
            "evaluates to {} values, which its {} value pool cannot hold",
            value_pool_kind_name(self.empty_value_pool().values()),
            value_pool_kind_name(value_pool.values()),
        ))
    }

    /// Errors unless every value lies in the range of the vocabulary property
    /// `property`. A name outside the vocabulary has no range and passes. So
    /// does a kind the name does not read.
    pub fn check_range(&self, property: &str) -> Result<()> {
        let Some(kind) = MaterialPropertyKind::of(property) else {
            return Ok(());
        };

        match (kind, self) {
            (
                MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba,
                WrittenValues::Vec3Float(colors),
            ) => check_colors(property, colors),

            (
                MaterialPropertyKind::ColorRgb | MaterialPropertyKind::ColorRgba,
                WrittenValues::Vec4Float(colors),
            ) => check_colors(property, colors),

            (MaterialPropertyKind::Scalar, WrittenValues::Float(numbers)) => {
                let range =
                    scalar_range(property).expect("every scalar vocabulary property has a range");

                for &number in numbers {
                    check_material_range(property, number, range)?;
                }

                Ok(())
            }

            (MaterialPropertyKind::Scalar, WrittenValues::Int(numbers)) => {
                let range =
                    scalar_range(property).expect("every scalar vocabulary property has a range");

                for &number in numbers {
                    check_material_range(property, number as f64, range)?;
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }

    /// Lands each value in `value_pool_id` and returns the value ids in
    /// material order. An equal value the value pool already holds is reused,
    /// and any other is appended. Expects `value_pool_id` to hold the values'
    /// kind.
    pub fn land<T: VoxExt>(
        self,
        main: &mut VoxMain<T>,
        value_pool_id: U32Id<BVoxValuePool>,
    ) -> Vec<U32Id<BVoxValuePoolValue>> {
        match self {
            WrittenValues::Bool(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::boolean_values,
                VoxMain::retain_boolean_value,
            ),

            WrittenValues::Float(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::float_values,
                VoxMain::retain_float_value,
            ),

            WrittenValues::Int(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::int_values,
                VoxMain::retain_int_value,
            ),

            WrittenValues::String(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::string_values,
                VoxMain::retain_string_value,
            ),

            WrittenValues::Vec2Float(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::vec_2_float_values,
                VoxMain::retain_vec_2_float_value,
            ),

            WrittenValues::Vec2Int(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::vec_2_int_values,
                VoxMain::retain_vec_2_int_value,
            ),

            WrittenValues::Vec3Float(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::vec_3_float_values,
                VoxMain::retain_vec_3_float_value,
            ),

            WrittenValues::Vec3Int(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::vec_3_int_values,
                VoxMain::retain_vec_3_int_value,
            ),

            WrittenValues::Vec4Float(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::vec_4_float_values,
                VoxMain::retain_vec_4_float_value,
            ),

            WrittenValues::Vec4Int(values) => land_each(
                main,
                value_pool_id,
                values,
                VoxValuePool::vec_4_int_values,
                VoxMain::retain_vec_4_int_value,
            ),
        }
    }
}

/// `components` repeated until it holds `count`. A plain value repeats once
/// per material. A swatch array already holds `count`.
fn cycled<T: Clone>(components: Vec<T>, count: usize) -> Vec<T> {
    components.iter().cycle().take(count).cloned().collect()
}

/// The float values of `components`, `dimension` components per value.
fn floats(components: Vec<f64>, dimension: Dimension) -> WrittenValues {
    match dimension {
        Dimension::Vec1 => WrittenValues::Float(components),
        Dimension::Vec2 => WrittenValues::Vec2Float(arrays(&components)),
        Dimension::Vec3 => WrittenValues::Vec3Float(arrays(&components)),
        Dimension::Vec4 => WrittenValues::Vec4Float(arrays(&components)),
    }
}

/// The int values of `components`, `dimension` components per value.
fn ints(components: Vec<i64>, dimension: Dimension) -> WrittenValues {
    match dimension {
        Dimension::Vec1 => WrittenValues::Int(components),
        Dimension::Vec2 => WrittenValues::Vec2Int(arrays(&components)),
        Dimension::Vec3 => WrittenValues::Vec3Int(arrays(&components)),
        Dimension::Vec4 => WrittenValues::Vec4Int(arrays(&components)),
    }
}

/// `components` grouped into arrays of `N`.
fn arrays<T: Copy, const N: usize>(components: &[T]) -> Vec<[T; N]> {
    components.as_chunks::<N>().0.to_vec()
}

/// Errors unless every component of each color lies in the color range.
fn check_colors<const N: usize>(property: &str, colors: &[[f64; N]]) -> Result<()> {
    for &component in colors.iter().flatten() {
        check_material_range(property, component, COLOR_RANGE)?;
    }

    Ok(())
}

/// Lands each of `values` in `value_pool_id`, reusing an equal value `column`
/// reads and appending any other through `retain`.
fn land_each<T: VoxExt, V: PartialEq>(
    main: &mut VoxMain<T>,
    value_pool_id: U32Id<BVoxValuePool>,
    values: Vec<V>,
    column: for<'a> fn(&'a VoxValuePool) -> Option<VoxValueColumn<'a, V>>,
    retain: RetainValue<T, V>,
) -> Vec<U32Id<BVoxValuePoolValue>> {
    let mut value_ids = Vec::with_capacity(values.len());

    for value in values {
        let value_pool = main
            .value_pool(value_pool_id)
            .expect("a written property draws from a live value pool");

        let held_values = column(value_pool).expect("a written value fits its value pool");

        let held_id = held_values
            .iter()
            .find(|&(_, held)| *held == value)
            .map(|(value_id, _)| value_id);

        let value_id = match held_id {
            Some(value_id) => value_id,

            None => retain(main, value_pool_id, value)
                .expect("an evaluated value is finite and within the u32 range"),
        };

        value_ids.push(value_id);
    }

    value_ids
}

#[cfg(test)]
mod tests {
    use crate::utilities::WrittenValues;
    use vox_value_language::{Components, Dimension, Domain, Value};
    use voxcore::{
        VoxMain, VoxValuePool,
        material::{BASE_COLOR, ROUGHNESS},
    };

    fn value(domain: Domain, dimension: Dimension, components: Components) -> Value {
        Value::new(domain, dimension, components).unwrap()
    }

    #[test]
    fn a_plain_value_lands_on_every_material() {
        let plain = value(Domain::Plain, Dimension::Vec2, Components::U8(vec![1, 2]));

        let values = WrittenValues::of(plain, 3).unwrap();

        assert_eq!(values, WrittenValues::Vec2Int(vec![[1, 2]; 3]));
    }

    #[test]
    fn a_swatch_array_lands_one_entry_per_material() {
        let swatch = value(
            Domain::Swatch,
            Dimension::Vec1,
            Components::F64(vec![0.25, 0.5]),
        );

        let values = WrittenValues::of(swatch, 2).unwrap();

        assert_eq!(values, WrittenValues::Float(vec![0.25, 0.5]));
    }

    #[test]
    fn a_voxel_value_errors() {
        let voxel = value(Domain::Voxel, Dimension::Vec1, Components::F64(vec![]));

        let reason = WrittenValues::of(voxel, 2).unwrap_err();

        assert!(reason.contains("voxel value"), "{reason}");
    }

    #[test]
    fn a_bool_wider_than_vec1_errors() {
        let flags = value(
            Domain::Plain,
            Dimension::Vec2,
            Components::Bool(vec![true, false]),
        );

        let reason = WrittenValues::of(flags, 1).unwrap_err();

        assert!(reason.contains("bool vec2"), "{reason}");
    }

    #[test]
    fn a_kind_other_than_the_pool_errors() {
        let values = WrittenValues::Int(vec![1]);
        let value_pool = VoxValuePool::float(vec![0.5]).unwrap();

        assert_eq!(
            values.check_fits(&value_pool),
            Err("evaluates to int values, which its float value pool cannot hold".to_owned())
        );
    }

    #[test]
    fn a_vocabulary_property_checks_its_range() {
        assert!(
            WrittenValues::Float(vec![0.0, 1.0])
                .check_range(ROUGHNESS)
                .is_ok()
        );

        let error = WrittenValues::Float(vec![1.2])
            .check_range(ROUGHNESS)
            .unwrap_err()
            .to_string();
        assert!(error.contains("1.2"), "{error}");

        let error = WrittenValues::Vec4Float(vec![[0.0, 0.0, 1.5, 1.0]])
            .check_range(BASE_COLOR)
            .unwrap_err()
            .to_string();
        assert!(error.contains(BASE_COLOR), "{error}");

        assert!(WrittenValues::Float(vec![7.0]).check_range("wear").is_ok());
    }

    #[test]
    fn landing_reuses_an_equal_value_and_appends_the_rest() {
        let mut main: VoxMain = VoxMain::default();
        let value_pool_id = main.retain_value_pool(VoxValuePool::float(Vec::new()).unwrap());
        let half_id = main.retain_float_value(value_pool_id, 0.5).unwrap();

        let value_ids = WrittenValues::Float(vec![0.25, 0.5, 0.25]).land(&mut main, value_pool_id);

        assert_eq!(value_ids[1], half_id);
        assert_eq!(value_ids[0], value_ids[2]);
        assert_ne!(value_ids[0], half_id);
        assert_eq!(main.value_pool(value_pool_id).unwrap().len(), 2);
    }
}
