use std::fmt::Debug;

pub trait Command: Send + Sync + Debug {
    fn command_type() -> &'static str;
}
