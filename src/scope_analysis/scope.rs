use std::marker::PhantomData;

use id_arena::Id;

use super::definition::_Definition;

pub struct _Scope<'a> {
    definitions: Vec<Id<_Definition<'a>>>,
    _phantom_data: PhantomData<&'a ()>,
}
