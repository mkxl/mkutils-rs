use mkutils_macros::{Inner, SetVariant, Toggle};

#[derive(Debug, PartialEq, SetVariant, Toggle)]
enum MyEnum {
    UnitOne,
    UnitTwo,
    UnitThree,
    String(String),
    Tuple(i32, i32),
}

#[derive(Inner)]
struct MyTupleStruct(bool, usize);

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

#[test]
fn test_inner() {
    let my_tuple_struct = MyTupleStruct(false, 0);
    let actual_pair = my_tuple_struct.inner();
    let expected_pair = (my_tuple_struct.0, my_tuple_struct.1);

    std::assert_eq!(actual_pair, expected_pair)
}
