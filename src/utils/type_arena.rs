use std::{
    collections::{HashMap, HashSet},
    iter::empty,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeArenaError {
    TypeNotFound,
    DuplicateTypeKey,
}

pub type TypeArenaResult<T> = Result<T, TypeArenaError>;

/**
 * In the `TypeArena` we store `TypeModel`s. They can have relations with other types in the
 * arena. The eventual goal is to transform `Union` and `Intersection` types to something rust
 * understands.
 */
#[derive(Debug, Default)]
pub struct TypeArena {
    type_models: HashMap<usize, TypeModel<usize>>,
    last_key: usize,
}
impl TypeArena {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_type_model(&self, type_key: usize) -> TypeArenaResult<&TypeModel<usize>> {
        self.type_models
            .get(&type_key)
            .ok_or(TypeArenaError::TypeNotFound)
    }

    pub fn register_type_model(&mut self, type_model: TypeModel<usize>) -> TypeArenaResult<usize> {
        let type_key = self.get_next_key();

        if self.type_models.insert(type_key, type_model).is_some() {
            return Err(TypeArenaError::DuplicateTypeKey);
        }

        Ok(type_key)
    }

    pub fn register_type_union(
        &mut self,
        left_type_key: usize,
        right_type_key: usize,
    ) -> TypeArenaResult<usize> {
        let left_type_model = self.get_type_model(left_type_key)?;
        let right_type_model = self.get_type_model(right_type_key)?;

        let left_type_enum = left_type_model.get_type_enum();
        let right_type_enum = right_type_model.get_type_enum();

        /*
         * if both types are the same return left
         */
        if left_type_enum == right_type_enum {
            return Ok(left_type_key);
        }

        /*
         * If one of the types is never, return the other
         *
         */
        if *left_type_enum == TypeEnum::Never {
            return Ok(right_type_key);
        }
        if *right_type_enum == TypeEnum::Never {
            return Ok(left_type_key);
        }

        /*
         * If one of the types is any, return that type (any)
         *
         */
        if *left_type_enum == TypeEnum::Any {
            return Ok(left_type_key);
        }
        if *right_type_enum == TypeEnum::Any {
            return Ok(right_type_key);
        }

        if let TypeEnum::Union(left_union_type) = left_type_enum {
            if let TypeEnum::Union(right_union_type) = right_type_enum {
                /*
                 * if both are unions then concatenate them
                 */
                let union_type_enum = TypeEnum::Union(UnionType::new(
                    empty()
                        .chain(left_union_type.get_types())
                        .chain(right_union_type.get_types())
                        .cloned()
                        .collect(),
                ));

                let union_type_model = union_type_enum.into();
                return self.register_type_model(union_type_model);
            } else {
                /*
                 * if one of them is a union then create a new union
                 */
                let union_type_enum = TypeEnum::Union(UnionType::new(
                    empty()
                        .chain(left_union_type.get_types())
                        .cloned()
                        .chain([right_type_key])
                        .collect(),
                ));

                let union_type_model = union_type_enum.into();
                return self.register_type_model(union_type_model);
            }
        } else if let TypeEnum::Union(right_union_type) = right_type_enum {
            /*
             * if one of them is a union then create a new union
             */
            let union_type_enum = TypeEnum::Union(UnionType::new(
                empty()
                    .chain(right_union_type.get_types())
                    .cloned()
                    .chain([left_type_key])
                    .collect(),
            ));

            let union_type_model = union_type_enum.into();
            return self.register_type_model(union_type_model);
        }

        /*
         * return union if no match
         */
        let union_type_enum = TypeEnum::Union([left_type_key, right_type_key].into());

        let union_type_model = union_type_enum.into();
        self.register_type_model(union_type_model)
    }

    pub fn register_type_intersection(
        &mut self,
        left_type_key: usize,
        right_type_key: usize,
    ) -> TypeArenaResult<usize> {
        let left_type_model = self.get_type_model(left_type_key)?;
        let right_type_model = self.get_type_model(right_type_key)?;

        let left_type_enum = left_type_model.get_type_enum();
        let right_type_enum = right_type_model.get_type_enum();

        /*
         * if both types are the same return left
         */
        if left_type_enum == right_type_enum {
            return Ok(left_type_key);
        }

        /*
         * If one of the types is never, return that type (never)
         *
         */
        if *left_type_enum == TypeEnum::Never {
            return Ok(left_type_key);
        }
        if *right_type_enum == TypeEnum::Never {
            return Ok(right_type_key);
        }

        /*
         * If one of the types is any, return the other type
         *
         */
        if *left_type_enum == TypeEnum::Any {
            return Ok(right_type_key);
        }
        if *right_type_enum == TypeEnum::Any {
            return Ok(left_type_key);
        }

        if let TypeEnum::Object(left_object_type) = left_type_enum {
            let left_object_type = left_object_type.clone();

            if let TypeEnum::Object(right_object_type) = right_type_enum {
                let right_object_type = right_object_type.clone();

                let mut intersection_properties: HashMap<_, _> = Default::default();

                let left_properties = left_object_type.get_properties();
                let right_properties = right_object_type.get_properties();
                let property_names: HashSet<_> = empty()
                    .chain(left_properties.keys())
                    .chain(right_properties.keys())
                    .collect();

                for property_name in property_names {
                    let left_property_type_key = left_properties.get(property_name);
                    let right_property_type_key = right_properties.get(property_name);

                    let property_name = property_name.clone();

                    if let Some(left_property_type_key) = left_property_type_key {
                        let left_property_type_key = *left_property_type_key;

                        if let Some(right_property_type_key) = right_property_type_key {
                            let right_property_type_key = *right_property_type_key;

                            /*
                             * both property type keys are set
                             */
                            let intersection_property_type_key = self.register_type_intersection(
                                left_property_type_key,
                                right_property_type_key,
                            )?;
                            assert!(intersection_properties
                                .insert(property_name, intersection_property_type_key)
                                .is_none())
                        } else {
                            /*
                             * only left property type key is set
                             */
                            assert!(intersection_properties
                                .insert(property_name, left_property_type_key)
                                .is_none())
                        }
                    } else if let Some(right_property_type_key) = right_property_type_key {
                        let right_property_type_key = *right_property_type_key;

                        /*
                         * only right property type key is set
                         */
                        assert!(intersection_properties
                            .insert(property_name, right_property_type_key)
                            .is_none(),)
                    } else {
                        /*
                         * both left and right property keys cannot be none
                         */
                        unreachable!()
                    }
                }

                let intersection_type_enum =
                    TypeEnum::Object(ObjectType::new(intersection_properties));

                let intersection_type_model = intersection_type_enum.into();
                return self.register_type_model(intersection_type_model);
            }
        }

        /*
         * if one of the types is a union, this will create another union
         */
        if let TypeEnum::Union(left_union_type) = left_type_enum {
            let left_union_type = left_union_type.clone();

            let mut types = Vec::new();
            for type_key in left_union_type.get_types() {
                types.push(self.register_type_intersection(*type_key, right_type_key)?);
            }

            let intersection_type_enum = TypeEnum::Union(types.into());

            let intersection_type_model = intersection_type_enum.into();
            return self.register_type_model(intersection_type_model);
        }
        if let TypeEnum::Union(right_union_type) = right_type_enum {
            let right_union_type = right_union_type.clone();

            let mut types = Vec::new();
            for type_key in right_union_type.get_types() {
                types.push(self.register_type_intersection(*type_key, left_type_key)?);
            }

            let intersection_type_enum = TypeEnum::Union(types.into());

            let intersection_type_model = intersection_type_enum.into();
            return self.register_type_model(intersection_type_model);
        }

        /*
         * return never if no match
         */
        let intersection_type_enum = TypeEnum::Never;

        let intersection_type_model = intersection_type_enum.into();
        self.register_type_model(intersection_type_model)
    }

    fn get_next_key(&mut self) -> usize {
        self.last_key += 1;

        self.last_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeEnum<K> {
    Never,
    Any,
    Boolean,
    Integer,
    String,
    Object(ObjectType<K>),
    Union(UnionType<K>),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectType<K> {
    properties: HashMap<String, K>,
}
impl<K> ObjectType<K> {
    pub fn new(properties: HashMap<String, K>) -> Self {
        Self { properties }
    }

    pub fn get_properties(&self) -> &HashMap<String, K> {
        &self.properties
    }
}
impl<K, T> From<T> for ObjectType<K>
where
    T: IntoIterator<Item = (String, K)>,
{
    fn from(value: T) -> Self {
        Self::new(value.into_iter().collect())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnionType<K> {
    types: Vec<K>,
}
impl<K> UnionType<K> {
    pub fn new(types: Vec<K>) -> Self {
        assert!(types.len() >= 2);

        Self { types }
    }

    pub fn get_types(&self) -> &Vec<K> {
        &self.types
    }
}
impl<K, T> From<T> for UnionType<K>
where
    T: IntoIterator<Item = K>,
{
    fn from(value: T) -> Self {
        Self::new(value.into_iter().collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeModel<K> {
    name: Option<String>,
    type_enum: TypeEnum<K>,
}
impl<K> TypeModel<K> {
    pub fn new(name: Option<String>, type_enum: TypeEnum<K>) -> Self {
        Self { name, type_enum }
    }

    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn get_type_enum(&self) -> &TypeEnum<K> {
        &self.type_enum
    }
}
impl<K> From<TypeEnum<K>> for TypeModel<K> {
    fn from(type_enum: TypeEnum<K>) -> Self {
        Self::new(None, type_enum)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_union() {
        let mut arena = TypeArena::new();

        let property_type_a_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_a: ObjectType<_> = [("a".into(), property_type_a_key)].into();
        let object_type_a_key = arena
            .register_type_model(TypeEnum::Object(object_type_a).into())
            .unwrap();

        let property_type_b_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_b: ObjectType<_> = [("b".into(), property_type_b_key)].into();
        let object_type_b_key = arena
            .register_type_model(TypeEnum::Object(object_type_b).into())
            .unwrap();

        let union_type_ab_key = arena
            .register_type_union(object_type_a_key, object_type_b_key)
            .unwrap();
        let union_type_ab_enum = arena
            .get_type_model(union_type_ab_key)
            .unwrap()
            .get_type_enum();

        assert_eq!(
            union_type_ab_enum,
            &TypeEnum::Union([object_type_a_key, object_type_b_key].into())
        );

        let property_type_c_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_c: ObjectType<_> = [("c".into(), property_type_c_key)].into();
        let object_type_c_key = arena
            .register_type_model(TypeEnum::Object(object_type_c).into())
            .unwrap();

        let union_type_abc_key = arena
            .register_type_union(union_type_ab_key, object_type_c_key)
            .unwrap();
        let union_type_abc_enum = arena
            .get_type_model(union_type_abc_key)
            .unwrap()
            .get_type_enum();

        assert_eq!(
            union_type_abc_enum,
            &TypeEnum::Union([object_type_a_key, object_type_b_key, object_type_c_key].into())
        );

        let property_type_d_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_d: ObjectType<_> = [("c".into(), property_type_d_key)].into();
        let object_type_d_key = arena
            .register_type_model(TypeEnum::Object(object_type_d).into())
            .unwrap();

        let property_type_e_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_e: ObjectType<_> = [("c".into(), property_type_e_key)].into();
        let object_type_e_key = arena
            .register_type_model(TypeEnum::Object(object_type_e).into())
            .unwrap();

        let union_type_de_key = arena
            .register_type_union(object_type_d_key, object_type_e_key)
            .unwrap();
        let union_type_de_enum = arena
            .get_type_model(union_type_de_key)
            .unwrap()
            .get_type_enum();

        assert_eq!(
            union_type_de_enum,
            &TypeEnum::Union([object_type_d_key, object_type_e_key].into())
        );

        let union_type_abcde_key = arena
            .register_type_union(union_type_abc_key, union_type_de_key)
            .unwrap();
        let union_type_abcde_enum = arena
            .get_type_model(union_type_abcde_key)
            .unwrap()
            .get_type_enum();

        assert_eq!(
            union_type_abcde_enum,
            &TypeEnum::Union(
                [
                    object_type_a_key,
                    object_type_b_key,
                    object_type_c_key,
                    object_type_d_key,
                    object_type_e_key
                ]
                .into()
            )
        );
    }

    #[test]
    fn test_complicated() {
        let mut arena = TypeArena::new();

        let property_type_a_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_a: ObjectType<_> = [("a".into(), property_type_a_key)].into();
        let object_type_a_key = arena
            .register_type_model(TypeEnum::Object(object_type_a).into())
            .unwrap();

        let property_type_b_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_b: ObjectType<_> = [("b".into(), property_type_b_key)].into();
        let object_type_b_key = arena
            .register_type_model(TypeEnum::Object(object_type_b).into())
            .unwrap();

        let property_type_c_key = arena.register_type_model(TypeEnum::String.into()).unwrap();
        let object_type_c: ObjectType<_> = [("c".into(), property_type_c_key)].into();
        let object_type_c_key = arena
            .register_type_model(TypeEnum::Object(object_type_c).into())
            .unwrap();

        let union_type_key = arena
            .register_type_union(object_type_a_key, object_type_b_key)
            .unwrap();

        let intersection_type_key = arena
            .register_type_intersection(union_type_key, object_type_c_key)
            .unwrap();
        let intersection_type_enum = arena
            .get_type_model(intersection_type_key)
            .unwrap()
            .get_type_enum();

        if let TypeEnum::Union(intersection_object_type) = intersection_type_enum {
            let intersection_types = intersection_object_type.get_types();

            assert_eq!(intersection_types.len(), 2);

            let object_type_ac_key = *intersection_types.first().unwrap();
            let object_type_ac_enum = arena
                .get_type_model(object_type_ac_key)
                .unwrap()
                .get_type_enum();

            let object_type_bc_key = *intersection_types.get(1).unwrap();
            let object_type_bc_enum = arena
                .get_type_model(object_type_bc_key)
                .unwrap()
                .get_type_enum();

            assert_eq!(
                [object_type_ac_enum, object_type_bc_enum],
                [
                    &TypeEnum::Object(
                        [
                            ("a".into(), property_type_a_key),
                            ("c".into(), property_type_c_key)
                        ]
                        .into()
                    ),
                    &TypeEnum::Object(
                        [
                            ("b".into(), property_type_b_key),
                            ("c".into(), property_type_c_key)
                        ]
                        .into()
                    ),
                ],
            );
        } else {
            panic!("unexpected {:?}", intersection_type_enum);
        }
    }
}