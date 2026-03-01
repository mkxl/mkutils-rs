use mkutils_macros::{Constructor, SetVariant, Toggle};

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

#[derive(Constructor)]
struct CStruct {
    name: String,
    count: i32,
}

#[derive(Constructor)]
struct TupleStruct(bool, usize);

#[derive(Constructor)]
struct UnitStruct;

#[derive(Constructor)]
#[new(pub)]
struct PubConstructorStruct(u8);

#[test]
fn test_constructor_c_struct() {
    let val = CStruct::new(String::from("hello"), 42);

    std::assert_eq!(val.name, "hello");
    std::assert_eq!(val.count, 42);
}

#[test]
fn test_constructor_tuple_struct() {
    let val = TupleStruct::new(true, 7);

    std::assert_eq!(val.0, true);
    std::assert_eq!(val.1, 7);
}

#[test]
fn test_constructor_unit_struct() {
    let _val = UnitStruct::new();
}

#[test]
fn test_constructor_pub_visibility() {
    let val = PubConstructorStruct::new(255);

    std::assert_eq!(val.0, 255);
}
