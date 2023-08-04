use crate::game::primitives::*;
use enum_map::{enum_map, EnumArray, EnumMap};
use std::iter::Iterator;

pub trait Vectorize {
    fn vectorize_append(&self, vec: &mut Vec<f32>);
    fn vectorize(&self) -> Vec<f32>;
}

impl Vectorize for f32 {
    fn vectorize_append(&self, vec: &mut Vec<f32>) {
        vec.push(*self);
    }
    fn vectorize(&self) -> Vec<f32> {
        let mut res = Vec::new();
        self.vectorize_append(&mut res);
        res
    }
}

impl Vectorize for [f32] {
    fn vectorize_append(&self, vec: &mut Vec<f32>) {
        for v in self.iter() {
            v.vectorize_append(vec);
        }
    }
    fn vectorize(&self) -> Vec<f32> {
        let mut res = Vec::new();
        self.vectorize_append(&mut res);
        res
    }
}

impl<K: EnumArray<U>, U: Vectorize> Vectorize for &EnumMap<K, U> {
    fn vectorize_append(&self, vec: &mut Vec<f32>) {
        for (_, value) in self.iter() {
            value.vectorize_append(vec);
        }
    }
    fn vectorize(&self) -> Vec<f32> {
        let mut res = Vec::new();
        self.vectorize_append(&mut res);
        res
    }
}

impl<K: EnumArray<U>, U: Vectorize> Vectorize for EnumMap<K, U> {
    fn vectorize_append(&self, vec: &mut Vec<f32>) {
        for (_, value) in self {
            value.vectorize_append(vec);
        }
    }
    fn vectorize(&self) -> Vec<f32> {
        let mut res = Vec::new();
        self.vectorize_append(&mut res);
        res
    }
}

pub trait Unvectorize {
    fn unvectorize(iter: &mut dyn Iterator<Item = f32>) -> Self;
}

impl Unvectorize for f32 {
    fn unvectorize(iter: &mut dyn Iterator<Item = f32>) -> Self {
        iter.next().unwrap()
    }
}

impl<U: Unvectorize> Unvectorize for EnumMap<FaceKey, U> {
    fn unvectorize(iter: &mut dyn Iterator<Item = f32>) -> Self {
        enum_map! {
            FaceKey::A=>U::unvectorize(iter),
            FaceKey::B=>U::unvectorize(iter),
            FaceKey::C=>U::unvectorize(iter),
            FaceKey::D=>U::unvectorize(iter),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use enum_map::enum_map;
    use vectorize_derive::*;

    #[derive(Vectorize)]
    struct TestStruct {
        a: f32,
        b: f32,
        c: f32,
    }

    #[derive(Vectorize)]
    struct Parent {
        a: f32,
        b: TestStruct,
    }

    const TEST: TestStruct = TestStruct {
        a: 1.0,
        b: 2.0,
        c: 3.0,
    };

    #[test]
    fn test_basic() {
        assert_eq!(TEST.vectorize(), vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_nested() {
        let p = Parent { a: 4.0, b: TEST };

        assert_eq!(p.vectorize(), vec![4.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_enum_basic() {
        let thing = enum_map! {
            FaceKey::A => 1.0,
            FaceKey::B => 2.0,
            FaceKey::C => 3.0,
            FaceKey::D => 4.0,
        };

        assert_eq!(thing.vectorize(), vec![1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_enum_nested() {
        let thing = enum_map! {
            FaceKey::A => TEST,
            FaceKey::B => TEST,
            FaceKey::C => TEST,
            FaceKey::D => TEST,
        };

        assert_eq!(
            thing.vectorize(),
            vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0]
        );
    }
}
