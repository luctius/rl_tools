macro_rules! replace_expr {
    ($_t:tt -> $sub:expr) => {
        $sub
    };
}

#[allow(unused_macros)]
macro_rules! replace_type {
    ($_t:tt -> $sub:ident) => {
        $sub
    };
}

/// Imagine macro parameters, but more like those Russian dolls.
///
/// Calls m!(A, B, C), m!(A, B), m!(B), and m!() for i.e. (m, A, B, C)
/// where m is any macro, for any number of parameters.
#[allow(unused_macros)]
macro_rules! smaller_tuples_too {
    ($m: ident, $ty: ident) => {
        $m!{$ty}
        $m!{}
    };
    ($m: ident, $ty: ident, $($tt: ident),*) => {
        $m!{$ty, $($tt),*}
        smaller_tuples_too!{$m, $($tt),*}
    };
}

#[allow(unused_macros)]
macro_rules! count_tt {
    () => { 0 };
    ($odd:tt $($a:tt $b:tt)*) => { (count_tt!($($a)*) << 1) | 1 };
    ($($a:tt $even:tt)*) => { count_tt!($($a)*) << 1 };
}

#[allow(unused_macros)]
macro_rules! coord_to_id_3d {
    ( $typename:ident@[$size_x:ident] -> ($x:ident,$y:ident) ) => {
        Id::new($typename::type_id(), ($y * $size_y) + x)
    };
}

#[allow(unused_macros)]
macro_rules! coord_to_id_3d {
    ( $typename:ident@[$size_x:ident,$size_y:ident] -> ($x:ident,$y:ident,$z:ident) ) => {
        Id::new($typename::type_id(), ($z * $size_x, $size_y) + ($y * $size_y) + x)
    };
}
/*
#[allow(unused_macros)]
macro_rules! coord_to_id_4d {
    ( $typename:ident@[$size_x:ident,$size_y:ident,$size_z:ident] -> ($x:ident,$y:ident,$z:ident,$a:ident) ) => {
        Id::new($typename::type_id(), ($a * $size_x * $size_y * $size_z) + ($z * $size_x * $size_y) + ($y * $size_y) +x)
    };
}
*/

#[allow(unused_macros)]
macro_rules! id_to_coord_2d {
    ($id:ident@$size_x:ident) => {{
        let idx = $id.to_idx();
        let y = idx / $size_x;
        let x = idx % $size_x;
        (x, y)
    }};
}

#[allow(unused_macros)]
macro_rules! id_to_coord_3d {
    ($id:ident@$size_x:ident,$size_y:ident) => {{
        let idx = $id.to_idx();
        let z = idx / ($size_x * $size_y);
        let idx = idx - (z * ($size_x * $size_y));
        let y = idx / $size_x;
        let x = idx % $size_x;
        (x, y)
    }};
}
