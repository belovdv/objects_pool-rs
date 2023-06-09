use std::marker::PhantomData;

use super::id::Id;
use super::Pool;
use super::Storable;

pub trait Variant<Container> {
    fn pack(self) -> Container;
    fn unpack(from: &Container) -> &Self;
}

/// A set of objects.
/// Keeps all objects while exists.
/// `Id`s expected to be used like references.
///
/// # Example
///
/// ```
/// use objects_pool::{Pool as _, Simple, Variadic, variadic};
///
/// variadic!(C: String, i32);
///
/// let mut pool: Variadic<C, Simple<C>> = Default::default();
///
/// // With feature `fn_overload` using `pool.insert(...)` is possible.
/// let id_abc = pool.insert_s("abc".to_string());
/// let id_123 = pool.insert_s(123);
///
/// let id_abc_any = pool.insert(C::String("abc".to_string()));
/// let id_123_any = pool.insert(C::i32(123));
///
/// let id_abc_copy = id_abc;
/// assert!(id_abc == id_abc_copy);
/// assert!(id_123_any != id_abc_any);
/// // id_abc != id_abc_any // These are different types.
///
/// assert!(*pool.get_s(id_123) == 123);
/// assert!(matches!(pool.get(id_abc_any), C::String(_)));
/// ```
///
/// # Caveats
///
/// `Id` can only be used with set which is gotten from.
///
/// Uses `usize::add(1)` as `Id` generator.
pub struct Variadic<Container: Storable<InnerPool>, InnerPool: Pool<Container>> {
    pool: InnerPool,
    _p: PhantomData<Container>,
}

// To be done: may be proc? No, there isn't clear reason for it.
// To be done: impl `From<Id<ty>>` for `Id<name>`.
#[macro_export]
macro_rules! variadic {
    ($name:ident: $($ty:ident),*) => {
        ::objects_pool::variadic!(!enum_simple: $name: $($ty),*);
        $(::objects_pool::variadic!(!variant_impl: $name: $ty);)*
    };
    ($name:ident: $($ty:ident),*; derive($($derive:ident),*)) => {
        ::objects_pool::variadic!(!enum_derive: $name: $($ty),*; $($derive),*);
        $(::objects_pool::variadic!(!variant_impl: $name: $ty);)*
    };
    (!enum_simple: $name:ident: $($ty:ident),*) => {
        #[allow(non_camel_case_types)]
        pub enum $name {
            $($ty($ty)),*
        }
    };
    (!enum_derive: $name:ident: $($ty:ident),*; $($derive:ident),*) => {
        #[allow(non_camel_case_types)]
        #[derive($($derive),*)]
        pub enum $name {
            $($ty($ty)),*
        }
    };
    (!variant_impl: $name:ident: $ty:ident) => {
        impl ::objects_pool::Variant<$name> for $ty {
            fn pack(self) -> $name {
                $name::$ty(self)
            }

            fn unpack(from: &$name) -> &Self {
                match from {
                    $name::$ty(s) => s,
                    _ => unreachable!(),
                }
            }
        }
    };
}

impl<C> Variant<C> for C {
    fn pack(self) -> C {
        self
    }

    fn unpack(from: &C) -> &Self {
        from
    }
}

impl<Container: Storable<InnerPool>, InnerPool: Pool<Container>> Variadic<Container, InnerPool> {
    pub fn get_s<Type: Variant<Container>>(&self, id: Id<Type>) -> &Type {
        let id = Id::new(id.id);
        Type::unpack(self.pool.get(id))
    }

    pub fn insert_s<Type: Variant<Container>>(&mut self, value: Type) -> Id<Type> {
        let packed = Type::pack(value);
        let id = self.pool.insert(packed);
        Id::new(id.id)
    }
}

impl<Container: Storable<InnerPool> + Storable<Self>, InnerPool: Pool<Container>> Pool<Container>
    for Variadic<Container, InnerPool>
{
}

impl<Container: Storable<InnerPool>, InnerPool: Pool<Container> + Default> Default
    for Variadic<Container, InnerPool>
{
    fn default() -> Self {
        let pool = Default::default();
        Self {
            pool,
            _p: PhantomData,
        }
    }
}

impl<Container: Storable<InnerPool> + Variant<Container>, InnerPool: Pool<Container>>
    Storable<Variadic<Container, InnerPool>> for Container
{
    fn store(self, pool: &mut Variadic<Container, InnerPool>) -> Id<Self> {
        pool.pool.insert(self)
    }

    fn access(pool: &Variadic<Container, InnerPool>, id: Id<Self>) -> &Self {
        pool.pool.get(Id::new(id.id))
    }
}

#[cfg(feature = "fn_overload")]
impl<Container: Storable<InnerPool>, InnerPool: Pool<Container>, Type: Variant<Container>>
    Storable<Variadic<Container, InnerPool>> for Type
{
    default fn store(self, pool: &mut Variadic<Container, InnerPool>) -> Id<Self> {
        pool.insert_s(self)
    }

    default fn access(pool: &Variadic<Container, InnerPool>, id: Id<Self>) -> &Self {
        pool.get_s(id)
    }
}
