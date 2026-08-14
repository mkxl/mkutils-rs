use mkutils_macros::{ConstAssoc, Constructor, SetVariant, Toggle, empty, with};

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
struct TupleStruct(&'static str, usize);

#[derive(Constructor)]
struct UnitStruct;

#[derive(Constructor)]
#[constructor(pub)]
struct PubConstructorStruct(u8);

#[derive(Constructor)]
#[constructor(from_parts)]
struct NamedConstructorStruct(&'static str);

#[derive(Constructor)]
#[constructor(pub(crate) create)]
struct NamedPubConstructorStruct;

#[test]
fn test_constructor_c_struct() {
    let val = CStruct::new(String::from("hello"), 42);

    std::assert_eq!(val.name, "hello");
    std::assert_eq!(val.count, 42);
}

#[test]
fn test_constructor_tuple_struct() {
    let val = TupleStruct::new("true", 7);

    std::assert_eq!(val.0, "true");
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

#[test]
fn test_custom_constructor_name() {
    let val = NamedConstructorStruct::from_parts("hello");

    std::assert_eq!(val.0, "hello");
}

#[test]
fn test_custom_constructor_name_and_visibility() {
    let _val = NamedPubConstructorStruct::create();
}

#[derive(ConstAssoc)]
#[const_assoc(pub MAX_SIZE: usize = 1024)]
#[const_assoc(DEFAULT_NAME: &str = "unnamed")]
struct ConstAssocStruct;

#[test]
fn test_const_assoc() {
    std::assert_eq!(ConstAssocStruct::MAX_SIZE, 1024);
    std::assert_eq!(ConstAssocStruct::DEFAULT_NAME, "unnamed");
}

#[derive(ConstAssoc)]
#[const_assoc(pub(crate) SCALE_FACTOR: usize = 25)]
enum ConstAssocEnum {}

#[test]
fn test_const_assoc_enum() {
    std::assert_eq!(ConstAssocEnum::SCALE_FACTOR, 25);
}

#[derive(Default)]
struct WithStruct {
    values: Vec<usize>,
    type_size: usize,
}

impl WithStruct {
    #[with]
    fn set_values(&mut self, (first, second): (usize, usize)) -> &mut Self {
        self.values = std::vec![first, second];
        self
    }

    #[with]
    const fn record_type<T>(&mut self) {
        self.type_size = std::mem::size_of::<T>();
    }

    #[with]
    fn push(&mut self, value: usize) {
        self.values.push(value);
    }
}

#[test]
fn test_with() {
    let value = WithStruct::default()
        .with_values((2, 3))
        .with_type::<u64>()
        .with_push(5);

    std::assert_eq!(value.values, std::vec![2, 3, 5]);
    std::assert_eq!(value.type_size, std::mem::size_of::<u64>());
}

#[empty]
impl EmptyEnum {
    const NAME: &'static str = "enum";
}

#[empty(unit_struct)]
impl EmptyUnitStruct {
    const NAME: &'static str = "unit_struct";
}

#[empty(c_struct)]
impl EmptyCStruct {
    const NAME: &'static str = "c_struct";
}

#[empty(unit_struct)]
#[derive(Debug, Default, PartialEq)]
#[must_use]
impl AttributedEmptyUnitStruct {}

mod visible_empty_types {
    use super::empty;

    #[empty(pub unit_struct)]
    impl PublicUnitStruct {}

    #[empty(pub(crate) c_struct)]
    impl CrateCStruct {}
}

#[test]
fn test_empty() {
    fn assert_default<T: std::default::Default>() {}

    let consume_empty_enum: fn(EmptyEnum) -> ! = |value| match value {};
    std::hint::black_box(EmptyUnitStruct);
    std::hint::black_box(EmptyCStruct {});
    std::hint::black_box(visible_empty_types::PublicUnitStruct);
    std::hint::black_box(visible_empty_types::CrateCStruct {});

    std::assert_eq!(EmptyEnum::NAME, "enum");
    std::assert_eq!(EmptyUnitStruct::NAME, "unit_struct");
    std::assert_eq!(EmptyCStruct::NAME, "c_struct");
    assert_default::<AttributedEmptyUnitStruct>();
    std::assert_eq!(AttributedEmptyUnitStruct, AttributedEmptyUnitStruct);
    std::assert_eq!(
        std::format!("{AttributedEmptyUnitStruct:?}"),
        "AttributedEmptyUnitStruct"
    );
    std::hint::black_box(consume_empty_enum);
}
