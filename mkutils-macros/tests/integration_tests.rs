use mkutils_macros::{SetVariant, Toggle};

#[derive(Debug, PartialEq, SetVariant, Toggle)]
enum MyEnum {
    UnitOne,
    UnitTwo,
    UnitThree,
    String(String),
    Tuple(i32, i32),
}

#[test]
fn test_set_variant() {
    let mut val = MyEnum::UnitOne;

    val.set_unit_two();

    std::assert_eq!(val, MyEnum::UnitTwo);

    val.set_unit_three();

    std::assert_eq!(val, MyEnum::UnitThree);
}

#[test]
fn test_toggle() {
    std::assert_eq!(MyEnum::UnitOne.toggled(), MyEnum::UnitTwo);
    std::assert_eq!(MyEnum::UnitTwo.toggled(), MyEnum::UnitThree);
    std::assert_eq!(MyEnum::UnitThree.toggled(), MyEnum::UnitOne);
    std::assert_eq!(MyEnum::String(String::new()).toggled(), MyEnum::UnitOne);
    std::assert_eq!(MyEnum::Tuple(0, 0).toggled(), MyEnum::UnitOne);
}
