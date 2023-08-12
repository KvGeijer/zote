use enum_macros::TryFromPrimitive;

#[derive(TryFromPrimitive, Debug, PartialEq, Eq)]
enum Numbers {
    Zero,
    One,
    Two,
    Three,
    Four,
}

#[test]
fn to_variant() {
    assert_eq!(Numbers::try_from(0u8), Ok(Numbers::Zero));
    assert_eq!(Numbers::try_from(1u8), Ok(Numbers::One));
    assert_eq!(Numbers::try_from(2u8), Ok(Numbers::Two));
    assert_eq!(Numbers::try_from(3u8), Ok(Numbers::Three));
    assert_eq!(Numbers::try_from(4u8), Ok(Numbers::Four));
    assert_eq!(Numbers::try_from(5u8), Err(()));
}

#[test]
fn from_variant() {
    assert_eq!(Numbers::Zero as u8, 0u8);
    assert_eq!(Numbers::One as u8, 1u8);
    assert_eq!(Numbers::Two as u8, 2u8);
    assert_eq!(Numbers::Three as u8, 3u8);
    assert_eq!(Numbers::Four as u8, 4u8);
}
