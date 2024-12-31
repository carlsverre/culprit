use core::any::Any;
use core::any::TypeId;

pub struct Request<'a> {
    pub(crate) tid: TypeId,
    pub(crate) slot: Option<&'a dyn Any>,
}

impl<'a> Request<'a> {
    pub(crate) fn provide<T: 'static>(&mut self, val: &'a T) -> &mut Self {
        if self.slot.is_none() && TypeId::of::<T>() == self.tid {
            self.slot = Some(val as &'a dyn Any)
        }
        self
    }
}

pub trait Provider {
    fn provide<'a>(&'a self, request: &mut Request<'a>);
}

pub fn request<T>(provider: &(impl Provider + ?Sized)) -> Option<&T>
where
    T: 'static,
{
    let tid = TypeId::of::<T>();
    let mut request = Request { tid, slot: None };
    provider.provide(&mut request);
    request.slot.and_then(|s| s.downcast_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SimpleProvider {
        i: i32,
        s: String,
    }

    impl Provider for SimpleProvider {
        fn provide<'a>(&'a self, request: &mut Request<'a>) {
            request.provide(&self.i).provide(&self.s);
        }
    }

    #[test]
    fn test_request() {
        let provider = SimpleProvider {
            i: 42,
            s: "hello".to_string(),
        };

        assert_eq!(42, *request::<i32>(&provider).unwrap());
        assert_eq!("hello", *request::<String>(&provider).unwrap());
        assert!(request::<u128>(&provider).is_none());
    }
}
