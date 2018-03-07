macro_rules! plumb {
    ($label:expr, $future:expr) => ({
        $future
        .map(|thing| {println!("{} map {:?}", $label, thing);})
        .map_err(|thing| {println!("{} err {:?}", $label, thing);})
    })
}
