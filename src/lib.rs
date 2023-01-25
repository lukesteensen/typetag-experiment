use once_cell::race::OnceBox;
use serde::Deserialize;
use std::collections::{btree_map::Entry, BTreeMap};

mod content;
mod de;
mod typetag;

// /// /// /// /// ///
// APP STUFF
// /// /// /// /// ///

trait TransformConfig: Sync {
    fn build(&self) -> usize;
}

type Description = ComponentDescription<DeserializeFn<dyn TransformConfig>>;

inventory::collect!(Description);

impl dyn TransformConfig {
    const fn register<T>(name: &'static str, deserializer: T) -> ComponentDescription<T> {
        ComponentDescription { name, deserializer }
    }
}

impl<'de> Deserialize<'de> for Box<dyn TransformConfig> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        static REGISTRY: OnceBox<private::Registry<dyn TransformConfig>> = OnceBox::new();
        let registry = REGISTRY.get_or_init(|| {
            let mut map = BTreeMap::new();
            let mut names = Vec::new();
            for registered in inventory::iter::<Description> {
                match map.entry(registered.name) {
                    Entry::Vacant(entry) => {
                        entry.insert(Option::Some(registered.deserializer));
                    }
                    Entry::Occupied(mut entry) => {
                        entry.insert(Option::None);
                    }
                }
                names.push(registered.name);
            }
            names.sort_unstable();
            Box::new(private::Registry { map, names })
        });

        typetag::deserialize(deserializer, "TransformConfig", "type", registry)
    }
}

#[derive(Deserialize)]
struct SamplerConfig {
    num: usize,
}

impl TransformConfig for SamplerConfig {
    fn build(&self) -> usize {
        self.num
    }
}

inventory::submit!(<dyn TransformConfig>::register(
    "sampler",
    (|deserializer| Ok(Box::new(erased_serde::deserialize::<SamplerConfig>(
        deserializer
    )?))) as private::DeserializeFn<dyn TransformConfig>
));

// ERROR
// expec struct `BTreeMap<&'static str, std::option::Option<for<'a, 'b> fn(&'a mut (dyn erased_serde::Deserializer<'b> + 'a)) -> Result<Box<for<'a, 'b> fn(&'a mut (dyn erased_serde::Deserializer<'b> + 'a)) -> Result<Box<dyn TransformConfig>, erased_serde::Error>>, _>>>`
// found struct `BTreeMap<&        str, std::option::Option<for<'a, 'b> fn(&'a mut (dyn erased_serde::Deserializer<'b> + 'a)) -> Result<Box<Box<dyn TransformConfig>>, _>>>`

trait SourceConfig {
    fn build(&self) -> usize;
}

// /// /// /// /// ///
// LIB STUFF
// /// /// /// /// ///

pub struct ComponentDescription<T> {
    name: &'static str,
    deserializer: T,
}

type DeserializeFn<T> = fn(&mut dyn erased_serde::Deserializer) -> erased_serde::Result<Box<T>>;

mod private {
    use std::collections::BTreeMap;

    pub type DeserializeFn<T> =
        fn(&mut dyn erased_serde::Deserializer) -> erased_serde::Result<Box<T>>;

    pub struct Registry<T: ?Sized> {
        pub map: BTreeMap<&'static str, Option<DeserializeFn<T>>>,
        pub names: Vec<&'static str>,
    }
}
