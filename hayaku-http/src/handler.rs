use super::{Request, ResponseWriter};

pub trait Handler<T: Clone> {
    fn handler(&self, &Request, &mut ResponseWriter, &T);
}
