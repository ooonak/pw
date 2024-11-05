mod data_types;

fn main() {
    let machine = data_types::machine::load();

    let buffer = data_types::machine::serialize_machine(&machine);

    println!("Serialized, buffer contains: {:?}", buffer);

    let deserialized = data_types::machine::deserialize_machine(&buffer);
    println!("Deserialized: {:?}", deserialized);
}
