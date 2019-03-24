use core::fmt;

use crate::{Descriptor, Encoding};
use crate::multi::Encodings;
use super::never::Never;

/// An encoding for a union.
#[derive(Clone, Copy, Debug)]
pub struct Union<S, T> {
    name: S,
    members: T,
}

impl<S, T> Union<S, T> {
    /// Constructs an encoding for a union with the given name and
    /// members with the given encodings.
    pub const fn new(name: S, members: T) -> Union<S, T> {
        Union { name: name, members: members }
    }
}

impl<S, T> Union<S, T> where S: AsRef<str>, T: Encodings {
    fn name(&self) -> &str {
        self.name.as_ref()
    }
}

impl<S, T> Encoding for Union<S, T> where S: AsRef<str>, T: Encodings {
    type PointerTarget = Never;
    type ArrayItem = Never;
    type StructFields = Never;
    type UnionMembers = T;

    fn descriptor(&self) -> Descriptor<Never, Never, Never, T> {
        Descriptor::Union(self.name(), &self.members)
    }
}

impl<S, T> fmt::Display for Union<S, T> where S: AsRef<str>, T: Encodings {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.write(formatter)
    }
}

impl<S, T, E: ?Sized> PartialEq<E> for Union<S, T>
        where S: AsRef<str>, T: Encodings, E: Encoding {
    fn eq(&self, other: &E) -> bool {
        self.eq_encoding(other)
    }
}

#[cfg(test)]
mod tests {
    use std::string::ToString;
    use crate::encoding::Primitive;
    use crate::parse::StrEncoding;
    use super::*;

    #[test]
    fn test_union_display() {
        let f = (Primitive::Char, Primitive::Int);
        let s = Union::new("Onion", f);
        assert_eq!(s.to_string(), "(Onion=ci)");
    }

    #[test]
    fn test_eq_encoding() {
        let i = Primitive::Int;
        let c = Primitive::Char;

        let u = Union::new("Onion", (c, i));
        assert!(u == u);
        assert!(u != i);

        let u2 = StrEncoding::new("(Onion=ci)").unwrap();
        assert!(u == u2);
    }
}
