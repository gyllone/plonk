
use ark_ff::Field;

///
pub trait CustomTable<F: Field> {
    ///
    fn collect_rows() -> Vec<[F; 3]>;
}

///
pub trait CustomSet<F: Field> {
    ///
    type Element: Into<F> + Copy;
    ///
    fn collect_elements() -> Vec<Self::Element>;
    ///
    fn collect_rows() -> Vec<[F; 3]> {
        let elements = Self::collect_elements();
        let mut rows = Vec::with_capacity(elements.len());

        for e in elements {
            rows.push([e.into(), F::zero(), F::zero()]);
        }

        rows
    }
}

///
pub trait Custom1DMap<F: Field> {
    ///
    type X: Into<F> + Copy;
    ///
    type Y: Into<F> + Copy;
    ///
    fn lookup(x: Self::X) -> Self::Y;
    ///
    fn collect_x_axis() -> Vec<Self::X>;
    ///
    fn collect_rows() -> Vec<[F; 3]> {
        let x_axis = Self::collect_x_axis();
        let mut rows = Vec::with_capacity(x_axis.len());

        for x in x_axis {
            let y = Self::lookup(x);
            rows.push([x.into(), y.into(), F::zero()]);
        }

        rows
    }
}

///
pub trait Custom2DMap<F: Field> {
    ///
    type X: Into<F> + Copy;
    ///
    type Y: Into<F> + Copy;
    ///
    type Z: Into<F> + Copy;
    ///
    fn lookup(x: Self::X, y: Self::Y) -> Self::Z;
    ///
    fn collect_x_axis() -> Vec<Self::X>;
    ///
    fn collect_y_axis() -> Vec<Self::Y>;
    ///
    fn collect_rows() -> Vec<[F; 3]> {
        let x_axis = Self::collect_x_axis();
        let y_axis = Self::collect_y_axis();
        let mut rows = Vec::with_capacity(x_axis.len() * y_axis.len());

        for x in x_axis {
            for &y in y_axis.iter() {
                let z = Self::lookup(x, y);
                rows.push([x.into(), y.into(), z.into()]);
            }
        }

        rows
    }
}

///
#[macro_export]
macro_rules! impl_custom_table {
    ($table:ident, $inner:ident) => {
        impl<F: Field> CustomTable<F> for $table {
            fn collect_rows() -> Vec<[F; 3]> {
                <$table as $inner<F>>::collect_rows()
            }
        }
    };
    ($table:ident, $inner:ident, $id:ident, $tp:ty) => {
        impl<F: Field, const $id: $tp> CustomTable<F> for $table<$id> {
            fn collect_rows() -> Vec<[F; 3]> {
                <$table<$id> as $inner<F>>::collect_rows()
            }
        }
    };
}

///
#[macro_export]
macro_rules! impl_uint_range_table {
    ($table:ident) => {
        ///
        pub struct $table<const BITS: u32>;

        impl<F: Field, const BITS: u32> CustomSet<F> for $table<BITS> {
            type Element = u128;

            fn collect_elements() -> Vec<Self::Element> {
                (0..(1 << BITS)).collect()
            }
        }

        impl_custom_table!($table, CustomSet, BITS, u32);
    };
}

///
#[macro_export]
macro_rules! impl_uint_operation_table {
    ($table:ident, $tp:ty, |$x:ident| -> $body:block) => {
        ///
        pub struct $table;

        impl<F: Field> Custom1DMap<F> for $table {
            type X = $tp;
            type Y = $tp;

            fn lookup($x: Self::X) -> Self::Y $body

            fn collect_x_axis() -> Vec<Self::X> {
                (0..=<$tp>::MAX).collect()
            }
        }

        impl_custom_table!($table, Custom1DMap);
    };
    (@withvar $table:ident, $tp:ty, |$x:ident, $y:ident| -> $body:block) => {
        ///
        pub struct $table;

        impl<F: Field> Custom2DMap<F> for $table {
            type X = $tp;
            type Y = $tp;
            type Z = $tp;

            fn lookup($x: Self::X, $y: Self::Y) -> Self::Z $body

            fn collect_x_axis() -> Vec<Self::X> {
                (0..=<$tp>::MAX).collect()
            }

            fn collect_y_axis() -> Vec<Self::Y> {
                (0..=<$tp>::MAX).collect()
            }
        }

        impl_custom_table!($table, Custom2DMap);
    };
    (@withconst $table:ident, $tp:ty, |$a:ident, $b:ident| -> $body:block) => {
        ///
        pub struct $table<const OP: $tp>;

        impl<F: Field, const OP: $tp> Custom1DMap<F> for $table<OP> {
            type X = $tp;
            type Y = $tp;

            fn lookup(x: Self::X) -> Self::Y {
                let func = |$a: $tp, $b: $tp| $body;
                func(x, OP)
            }

            fn collect_x_axis() -> Vec<Self::X> {
                (0..=<$tp>::MAX).collect()
            }
        }

        impl_custom_table!($table, Custom1DMap, OP, $tp);
    };
}

impl_uint_range_table!(UintRangeTable);

impl_uint_operation_table!(U8NotTable, u8, |x| -> { !x });

impl_uint_operation_table!(U16NotTable, u16, |x| -> { !x });

impl_uint_operation_table!(@withvar U8OrTable, u8, |x, y| -> { x | y });
impl_uint_operation_table!(@withvar U8XorTable, u8, |x, y| -> { x ^ y });
impl_uint_operation_table!(@withvar U8AndTable, u8, |x, y| -> { x & y });
impl_uint_operation_table!(@withvar U8NotAndTable, u8, |x, y| -> { (!x) & y });

impl_uint_operation_table!(@withconst U8OrWithConstTable, u8, |x, y| -> { x | y });
impl_uint_operation_table!(@withconst U8XorWithConstTable, u8, |x, y| -> { x ^ y });
impl_uint_operation_table!(@withconst U8AndWithConstTable, u8, |x, y| -> { x & y });
impl_uint_operation_table!(@withconst U8NotAndWithConstTable, u8, |x, y| -> { (!x) & y });

impl_uint_operation_table!(@withconst U16OrWithConstTable, u16, |x, y| -> { x | y });
impl_uint_operation_table!(@withconst U16XorWithConstTable, u16, |x, y| -> { x ^ y });
impl_uint_operation_table!(@withconst U16AndWithConstTable, u16, |x, y| -> { x & y });
impl_uint_operation_table!(@withconst U16NotAndWithConstTable, u16, |x, y| -> { (!x) & y });