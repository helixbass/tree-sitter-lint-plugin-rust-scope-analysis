use std::marker::PhantomData;

pub struct _Definition<'a> {
    _phantom_data: PhantomData<&'a ()>,
}
