use serde::{Deserialize, Serialize};
use serde_cbor;
use sled::{ConfigBuilder, DbResult, Error as SledError, Tree};
use statics;
use std::marker::PhantomData;
use tempdir::TempDir;

#[derive(Clone, Debug)]
pub struct Db<'a, F: 'a + Deserialize<'a> + Serialize> {
    phantom: PhantomData<&'a F>,
    tree: Tree,
}

pub fn ro<'a, F: Deserialize<'a> + Serialize>() -> DbResult<Db<'a, F>, ()> {
    let config = config().clone().read_only(true).build();
    Tree::start(config).map(|t| Db {
        phantom: PhantomData,
        tree: t,
    })
}

pub fn rw<'a, F: Deserialize<'a> + Serialize>() -> DbResult<Db<'a, F>, ()> {
    let config = config().clone().build();
    Tree::start(config).map(|t| Db {
        phantom: PhantomData,
        tree: t,
    })
}

impl<'a, F: Deserialize<'a> + Serialize> Db<'a, F> {
    pub fn get(&self, key: &str) -> DbResult<Option<F>, ()> {
        self.tree.get(key.as_bytes()).and_then(Self::decode_val)
    }

    fn decode_val(val: Option<Vec<u8>>) -> DbResult<Option<F>, ()> {
        val.map_or(Ok(None), |buf|
            serde_cbor::from_reader(buf.as_slice()).map_err(|err| {
                println!("Bad cbor: {:?}\n{:?}", buf, err);
                SledError::Corruption { at: 0 }
            }).and_then(|v| Ok(Some(v)))
        )
    }

    pub fn flush(&self) -> DbResult<(), ()> {
        self.tree.flush()
    }

    pub fn set(&self, key: &str, value: &F) -> DbResult<(), ()> {
        serde_cbor::to_vec(value).map_err(|err| {
            println!("Bad encoding: {:?}", err);
            SledError::Corruption { at: 0 }
        }).and_then(|buf| self.tree.set(
            key.to_owned().into_bytes(),
            buf
        ))
    }

    pub fn del(&self, key: &str) -> DbResult<Option<F>, ()> {
        self.tree.del(key.as_bytes()).and_then(Self::decode_val)
    }
}

fn config() -> &'static ConfigBuilder {
    lazy_static! {
        static ref DB_CONFIG: ConfigBuilder = {
            let db = ConfigBuilder::new().use_compression(true);

            if let Some(ref path) = statics::config().persistent {
                db.path(path)
            } else {
                let tmp = TempDir::new(&statics::key()).expect("Cannot create temporary db");
                db.path(tmp).temporary(true)
            }
        };
    }

    &DB_CONFIG
}
